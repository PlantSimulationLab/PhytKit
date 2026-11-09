//! Reference values for every built-in model: predictions, Jacobians, penalties and
//! derived values on edge cases, plus full fits, compared with values stored in
//! `tests/data/model_reference.json`. This guards the models against unintended changes.

use indexmap::IndexMap;
use phytkit_core::fit::{fit, FitOptions};
use phytkit_core::numbers;
use phytkit_core::model::{defaults, Columns, Model, Params, PenaltyOptions};
use phytkit_core::models::{self, builtin_models, model_by_name};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/model_reference.json");

// Tolerances. Floating-point results can differ in the last bit between platforms, so
// values must agree closely rather than exactly. A fit that takes the same number of
// iterations must reach the same parameters, or an RMSE equal to FIT_PARAM_TOL; one that
// takes a different path (possible where the objective is flat, as with empiricalA) must
// reach an equally good fit.
const PRED_TOL: f64 = 1e-9; // relative to the largest |value| in the output vector
const ABS_TOL: f64 = 1e-14; // absolute, for outputs that are all near zero (rounding noise)
const FIT_PARAM_TOL: f64 = 1e-5; // relative, fitted parameters, when both took the same path
const FIT_RMSE_TOL: f64 = 1e-3; // relative RMSE difference allowed when the paths differ

fn num(v: &Value) -> f64 {
    match v {
        Value::String(s) => numbers::parse(s),
        Value::Number(n) => n.as_f64().unwrap(),
        _ => panic!("not a number: {v}"),
    }
}
fn nums(v: &Value) -> Vec<f64> {
    v.as_array().unwrap().iter().map(num).collect()
}
fn matrix(v: &Value) -> Vec<Vec<f64>> {
    v.as_array().unwrap().iter().map(nums).collect()
}
fn params(v: &Value) -> Params {
    v.as_object().unwrap().iter().map(|(k, x)| (k.clone(), num(x))).collect()
}
fn columns(v: &Value) -> Columns {
    v.as_object().unwrap().iter().map(|(k, x)| (k.clone(), nums(x))).collect()
}
fn strings(v: &Value) -> Vec<String> {
    v.as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_string()).collect()
}
fn opt_str(v: &Value) -> Option<&str> {
    v.as_str()
}
fn opt_num(v: &Value) -> Option<f64> {
    if v.is_null() { None } else { Some(num(v)) }
}

/// Max relative differences over a family of outputs.
#[derive(Default)]
struct Stat {
    strict: bool,
    values: usize,
    exact: usize,
    max_rel: f64,
    max_scaled: f64,
    worst: String,
    failures: Vec<String>,
}

impl Stat {
    fn same(a: f64, b: f64) -> bool {
        (a.is_nan() && b.is_nan()) || a == b
    }
    /// Compare expected values `want` with computed values `got`.
    fn vec(&mut self, what: &str, want: &[f64], got: &[f64]) {
        self.vec_floor(what, want, got, 0.0)
    }
    /// As `vec`, but differences are measured against at least `floor`, for outputs whose
    /// own magnitude says nothing (a Jacobian that is all rounding noise around zero).
    fn vec_floor(&mut self, what: &str, want: &[f64], got: &[f64], floor: f64) {
        if want.len() != got.len() {
            self.failures.push(format!("{what}: length {} (expected) vs {} (computed)", want.len(), got.len()));
            return;
        }
        let scale = want.iter().filter(|v| v.is_finite()).fold(floor, |s, v| s.max(v.abs()));
        for (i, (&a, &b)) in want.iter().zip(got).enumerate() {
            self.values += 1;
            if Self::same(a, b) {
                self.exact += 1;
                continue;
            }
            if !a.is_finite() || !b.is_finite() {
                self.failures.push(format!("{what}[{i}]: want {a} vs got {b}"));
                continue;
            }
            let d = (a - b).abs();
            let rel = d / a.abs().max(b.abs());
            let scaled = if scale > 0.0 { d / scale } else { rel };
            if rel > self.max_rel {
                self.max_rel = rel;
                self.worst = format!("{what}[{i}]: want {a:e} got {b:e}");
            }
            self.max_scaled = self.max_scaled.max(scaled);
            if self.strict && scaled > PRED_TOL && d > ABS_TOL {
                self.failures.push(format!("{what}[{i}]: want {a:e} vs got {b:e} (scaled diff {scaled:e})"));
            }
        }
    }
    fn line(&self, name: &str) -> String {
        if self.values == 0 {
            return format!("{name:>12}: -");
        }
        format!(
            "{name:>12}: {:5} values, {:5.1}% bit-identical, max rel {:.2e}, max rel-to-vector-max {:.2e}",
            self.values,
            100.0 * self.exact as f64 / self.values as f64,
            self.max_rel,
            self.max_scaled
        )
    }
}

fn check_metadata(m: &dyn Model, j: &Value, errs: &mut Vec<String>) {
    let name = j["name"].as_str().unwrap();
    let mut e = |s: String| errs.push(format!("{name}: {s}"));
    if m.inputs().iter().map(|s| s.to_string()).collect::<Vec<_>>() != strings(&j["inputs"]) {
        e(format!("inputs {:?}", m.inputs()));
    }
    if m.latex().as_deref() != opt_str(&j["latex"]) {
        e(format!("latex {:?} vs {:?}", m.latex(), j["latex"]));
    }
    let ps = j["params"].as_array().unwrap();
    if ps.len() != m.params().len() {
        e(format!("{} params vs {}", m.params().len(), ps.len()));
    }
    for (s, t) in m.params().iter().zip(ps) {
        let ok = s.name == t["name"].as_str().unwrap()
            && s.default.to_bits() == num(&t["default"]).to_bits()
            && s.lb.map(f64::to_bits) == opt_num(&t["lb"]).map(f64::to_bits)
            && s.ub.map(f64::to_bits) == opt_num(&t["ub"]).map(f64::to_bits)
            && s.fixed == t["fixed"].as_bool().unwrap()
            && s.description == opt_str(&t["description"]);
        if !ok {
            e(format!("param {s:?} vs {t}"));
        }
    }
    let cs = j["constraints"].as_array().unwrap();
    if cs.len() != m.constraints().len() {
        e(format!("{} constraints vs {}", m.constraints().len(), cs.len()));
    }
    for (c, t) in m.constraints().iter().zip(cs) {
        let ok = c.key == t["key"].as_str().unwrap()
            && c.label == t["label"].as_str().unwrap()
            && c.weight.to_bits() == num(&t["weight"]).to_bits()
            && c.description == opt_str(&t["description"])
            && c.per_curve == t["perCurve"].as_bool().unwrap();
        if !ok {
            e(format!("constraint {c:?} vs {t}"));
        }
    }
    if m.has_jacobian() != j["hasJacobian"].as_bool().unwrap() {
        e("hasJacobian".into());
    }
    if m.has_penalties() != j["hasPenalties"].as_bool().unwrap() {
        e("hasPenalties".into());
    }
    if m.derived(&defaults(m)).is_some() != j["hasDerived"].as_bool().unwrap() {
        e("hasDerived".into());
    }
}

#[test]
fn models_match_reference_values() {
    if std::env::var_os("UPDATE_MODEL_REFERENCE").is_some() {
        update_reference();
        return;
    }
    let text = std::fs::read_to_string(FIXTURE).expect("missing tests/data/model_reference.json");
    let root: Value = serde_json::from_str(&text).unwrap();
    let mut errs: Vec<String> = Vec::new();
    let mut report: Vec<String> = Vec::new();
    let strict = true;
    let new_stat = || Stat { strict, ..Default::default() };

    // Registry order (FvCB is not in the reference file).
    let rust_order: Vec<&str> = builtin_models().iter().map(|m| m.name()).collect();
    assert_eq!(
        rust_order,
        ["empiricalA", "fvcb", "fvcbC4", "vulnerability", "pvcurve", "vanGenuchten", "modifiedVanGenuchten", "raats", "brooksCorey", "fractalSWRC", "bta", "medlyn", "lightResponse"]
    );
    let ts_order: Vec<&str> = root["models"].as_array().unwrap().iter().map(|m| m["name"].as_str().unwrap()).collect();
    assert_eq!(ts_order, rust_order.iter().copied().filter(|n| *n != "fvcb").collect::<Vec<_>>());

    for jm in root["models"].as_array().unwrap() {
        let name = jm["name"].as_str().unwrap();
        let m = model_by_name(name).unwrap_or_else(|| panic!("model {name} not registered"));
        check_metadata(m, jm, &mut errs);

        let (mut sp, mut sj, mut spen, mut sd) = (new_stat(), new_stat(), new_stat(), new_stat());
        let cases = jm["cases"].as_array().unwrap();
        for c in cases {
            let label = c["label"].as_str().unwrap();
            let p = params(&c["p"]);
            let x = columns(&c["X"]);
            let pred_want = nums(&c["predict"]);
            sp.vec(&format!("{name} {label} predict"), &pred_want, &m.predict(&p, &x));
            // A derivative is only meaningful against the size of the prediction it describes.
            let pred_scale = pred_want.iter().filter(|v| v.is_finite()).fold(0.0f64, |s, v| s.max(v.abs()));
            if let Some(js) = c.get("jacobian") {
                for jc in js.as_array().unwrap() {
                    let wrt = strings(&jc["wrt"]);
                    let want = matrix(&jc["J"]);
                    let got = if name == "empiricalA" {
                        models::empirical_a_jacobian(&p, &x, &wrt)
                    } else {
                        m.jacobian(&p, &x, &wrt).expect("jacobian")
                    };
                    if want.len() != got.len() {
                        sj.failures.push(format!("{name} {label} jacobian rows"));
                        continue;
                    }
                    let flat_w: Vec<f64> = want.concat();
                    let flat_g: Vec<f64> = got.concat();
                    // Compare the whole matrix against its largest entry. Per-column scaling turns
                    // rounding noise in a column that is numerically zero (e.g. 1e-30) into a
                    // spurious 100% error.
                    sj.vec_floor(&format!("{name} {label} jacobian (row-major)"), &flat_w, &flat_g, pred_scale);
                    if flat_w.len() != flat_g.len() {
                        sj.failures.push(format!("{name} {label} jacobian shape"));
                    }
                }
            }
            if let Some(pens) = c.get("penalties") {
                for pc in pens.as_array().unwrap() {
                    let o = &pc["opts"];
                    let curves: Option<Vec<Vec<usize>>> = if o["curves"].is_null() {
                        None
                    } else {
                        Some(o["curves"].as_array().unwrap().iter().map(|g| nums(g).iter().map(|&v| v as usize).collect()).collect())
                    };
                    let enabled: Option<HashSet<String>> = o.get("enabled").map(|e| strings(e).into_iter().collect());
                    let weights: Option<HashMap<String, f64>> = o.get("weights").map(|w| w.as_object().unwrap().iter().map(|(k, v)| (k.clone(), num(v))).collect());
                    let po = PenaltyOptions { curves: curves.as_deref(), enabled: enabled.as_ref(), weights: weights.as_ref() };
                    let got = m.penalties(&p, &x, &po).expect("penalties");
                    spen.vec(&format!("{name} {label} penalties {o}"), &nums(&pc["out"]), &got);
                }
            }
            if let Some(d) = c.get("derived") {
                let want: IndexMap<String, f64> = d.as_object().unwrap().iter().map(|(k, v)| (k.clone(), num(v))).collect();
                let got = m.derived(&p).expect("derived");
                if want.keys().collect::<Vec<_>>() != got.keys().collect::<Vec<_>>() {
                    errs.push(format!("{name} {label}: derived keys {:?}", got.keys()));
                }
                sd.vec(&format!("{name} {label} derived"), &want.values().copied().collect::<Vec<_>>(), &got.values().copied().collect::<Vec<_>>());
            }
        }

        // Fits
        let fits = jm["fits"].as_array().unwrap();
        let (mut same_iter, mut same_conv, mut max_prel, mut max_rmse_rel) = (0, 0, 0.0f64, 0.0f64);
        let mut fit_lines = Vec::new();
        for f in fits {
            let label = f["label"].as_str().unwrap();
            let x = columns(&f["X"]);
            let y = nums(&f["y"]);
            let o = &f["opts"];
            let opts = FitOptions {
                fit_params: o.get("fitParams").map(strings),
                init: o.get("init").map(params).unwrap_or_default(),
                finite_diff: o.get("finiteDiff").and_then(|v| v.as_bool()).unwrap_or(false),
                curves: o.get("curves").map(|c| c.as_array().unwrap().iter().map(|g| nums(g).iter().map(|&v| v as usize).collect()).collect()),
                constraints: o.get("constraints").map(strings),
                constraint_weights: o.get("constraintWeights").map(|w| w.as_object().unwrap().iter().map(|(k, v)| (k.clone(), num(v))).collect()),
                ..Default::default()
            };
            let r = fit(m, &x, &y, &opts);
            let t = &f["result"];
            let (ti, tc) = (num(&t["iterations"]) as usize, t["converged"].as_bool().unwrap());
            if ti == r.iterations {
                same_iter += 1;
            }
            if tc == r.converged {
                same_conv += 1;
            }
            let tp = params(&t["params"]);
            let mut prel = 0.0f64;
            for (k, v) in &tp {
                let rv = r.params.get(k).copied().unwrap_or(f64::NAN);
                if !Stat::same(*v, rv) {
                    prel = prel.max((v - rv).abs() / v.abs().max(rv.abs()));
                }
            }
            if !prel.is_finite() {
                prel = f64::INFINITY;
            }
            let trmse = num(&t["rmse"]);
            let rmse_rel = if Stat::same(trmse, r.rmse) { 0.0 } else { (trmse - r.rmse).abs() / trmse.abs().max(r.rmse.abs()) };
            max_prel = max_prel.max(prel);
            max_rmse_rel = max_rmse_rel.max(rmse_rel);
            let same_path = ti == r.iterations;
            fit_lines.push(format!(
                "      fit {label:<22} iters want {ti:>3} got {:>3}  converged want {tc} got {}  rmse want {trmse:.6e} got {:.6e}  max param rel diff {prel:.2e}{}",
                r.iterations,
                r.converged,
                r.rmse,
                if same_path { "" } else { "  (different path)" }
            ));
            // Same iteration count usually means the same path, but along a flat direction the
            // parameters can drift apart even then (empiricalA on Windows/MSVC libm): accept that
            // only if the fit is equally good to the strict tolerance.
            let ok = tc == r.converged
                && (prel <= FIT_PARAM_TOL || rmse_rel <= if same_path { FIT_PARAM_TOL } else { FIT_RMSE_TOL });
            if strict && !ok {
                errs.push(format!("{name} fit {label}: iters {ti}/{} converged {tc}/{} rmse rel {rmse_rel:e} param rel {prel:e}", r.iterations, r.converged));
            }
        }

        report.push(format!("== {name} ({} cases, {} fits)", cases.len(), fits.len()));
        report.push(sp.line("predict"));
        report.push(sj.line("jacobian"));
        report.push(spen.line("penalties"));
        report.push(sd.line("derived"));
        if !sp.worst.is_empty() {
            report.push(format!("      worst predict: {}", sp.worst));
        }
        if !sj.worst.is_empty() {
            report.push(format!("      worst jacobian: {}", sj.worst));
        }
        report.push(format!(
            "     fits: {same_iter}/{} same iterations, {same_conv}/{} same convergence, max param rel diff {max_prel:.2e}, max rmse rel diff {max_rmse_rel:.2e}",
            fits.len(),
            fits.len()
        ));
        report.extend(fit_lines);
        for s in [sp, sj, spen, sd] {
            errs.extend(s.failures);
        }
    }

    // Helpers
    let h = &root["helpers"];
    let mut sh = new_stat();
    let (mut tw, mut rg) = (Vec::new(), Vec::new());
    for r in h["regIncBeta"].as_array().unwrap() {
        let v = nums(r);
        tw.push(v[3]);
        rg.push(models::reg_inc_beta(v[0], v[1], v[2]));
    }
    // compare element-wise (each value is its own scale)
    for (i, (a, b)) in tw.iter().zip(&rg).enumerate() {
        sh.vec(&format!("regIncBeta case {i}"), &[*a], &[*b]);
    }
    report.push(sh.line("regIncBeta"));
    errs.extend(std::mem::take(&mut sh.failures));
    type Helper = fn(&Params, f64) -> f64;
    for (key, f) in [
        ("raatsSaturation", models::raats_saturation as Helper),
        ("fractalSaturation", models::fractal_saturation as Helper),
        ("fractalSuction", models::fractal_suction as Helper),
    ] {
        let mut s = new_stat();
        for (i, c) in h[key].as_array().unwrap().iter().enumerate() {
            s.vec(&format!("{key} case {i}"), &[num(&c["out"])], &[f(&params(&c["p"]), num(&c["x"]))]);
        }
        report.push(s.line(key));
        errs.extend(s.failures);
    }

    println!("{}", report.join("\n"));
    if !errs.is_empty() {
        for e in errs.iter().take(60) {
            println!("FAIL {e}");
        }
        panic!("{} model reference failures", errs.len());
    }
}

/// A number as stored in the reference file: JSON numbers, with non-finite values as text.
fn enc(x: f64) -> Value {
    if x.is_finite() {
        serde_json::Number::from_f64(x).map(Value::Number).unwrap()
    } else {
        Value::String(numbers::text(x))
    }
}
fn enc_all(v: &[f64]) -> Value {
    Value::Array(v.iter().map(|x| enc(*x)).collect())
}

/// `UPDATE_MODEL_REFERENCE=1 cargo test --release --test model_reference` rewrites the
/// stored values from the current code: use it after changing a model on purpose, then
/// review the difference before committing it. Inputs and options are kept as they are.
fn update_reference() {
    let mut root: Value = serde_json::from_str(&std::fs::read_to_string(FIXTURE).expect("missing tests/data/model_reference.json")).unwrap();
    for jm in root["models"].as_array_mut().unwrap() {
        let name = jm["name"].as_str().unwrap().to_string();
        let m = model_by_name(&name).expect("model");
        for c in jm["cases"].as_array_mut().unwrap() {
            let (p, x) = (params(&c["p"]), columns(&c["X"]));
            c["predict"] = enc_all(&m.predict(&p, &x));
            if let Some(js) = c.get_mut("jacobian") {
                for jc in js.as_array_mut().unwrap() {
                    let wrt = strings(&jc["wrt"]);
                    let j = if name == "empiricalA" { models::empirical_a_jacobian(&p, &x, &wrt) } else { m.jacobian(&p, &x, &wrt).expect("jacobian") };
                    jc["J"] = Value::Array(j.iter().map(|r| enc_all(r)).collect());
                }
            }
            if let Some(pens) = c.get_mut("penalties") {
                for pc in pens.as_array_mut().unwrap() {
                    let o = pc["opts"].clone();
                    let curves: Option<Vec<Vec<usize>>> = if o["curves"].is_null() {
                        None
                    } else {
                        Some(o["curves"].as_array().unwrap().iter().map(|g| nums(g).iter().map(|&v| v as usize).collect()).collect())
                    };
                    let enabled: Option<HashSet<String>> = o.get("enabled").map(|e| strings(e).into_iter().collect());
                    let weights: Option<HashMap<String, f64>> = o.get("weights").map(|w| w.as_object().unwrap().iter().map(|(k, v)| (k.clone(), num(v))).collect());
                    let po = PenaltyOptions { curves: curves.as_deref(), enabled: enabled.as_ref(), weights: weights.as_ref() };
                    pc["out"] = enc_all(&m.penalties(&p, &x, &po).expect("penalties"));
                }
            }
            if c.get("derived").is_some() {
                let d = m.derived(&p).expect("derived");
                c["derived"] = Value::Object(d.into_iter().map(|(k, v)| (k, enc(v))).collect());
            }
        }
        for f in jm["fits"].as_array_mut().unwrap() {
            let (x, y, o) = (columns(&f["X"]), nums(&f["y"]), f["opts"].clone());
            let opts = FitOptions {
                fit_params: o.get("fitParams").map(strings),
                init: o.get("init").map(params).unwrap_or_default(),
                finite_diff: o.get("finiteDiff").and_then(|v| v.as_bool()).unwrap_or(false),
                curves: o.get("curves").map(|c| c.as_array().unwrap().iter().map(|g| nums(g).iter().map(|&v| v as usize).collect()).collect()),
                constraints: o.get("constraints").map(strings),
                constraint_weights: o.get("constraintWeights").map(|w| w.as_object().unwrap().iter().map(|(k, v)| (k.clone(), num(v))).collect()),
                ..Default::default()
            };
            let r = fit(m, &x, &y, &opts);
            let res = &mut f["result"];
            res["params"] = Value::Object(r.params.iter().map(|(k, v)| (k.clone(), enc(*v))).collect());
            res["rmse"] = enc(r.rmse);
            res["iterations"] = Value::from(r.iterations);
            res["converged"] = Value::Bool(r.converged);
        }
    }
    let h = &mut root["helpers"];
    for r in h["regIncBeta"].as_array_mut().unwrap() {
        let v = nums(r);
        r[3] = enc(models::reg_inc_beta(v[0], v[1], v[2]));
    }
    type Helper = fn(&Params, f64) -> f64;
    for (key, f) in [
        ("raatsSaturation", models::raats_saturation as Helper),
        ("fractalSaturation", models::fractal_saturation as Helper),
        ("fractalSuction", models::fractal_suction as Helper),
    ] {
        for c in h[key].as_array_mut().unwrap() {
            c["out"] = enc(f(&params(&c["p"]), num(&c["x"])));
        }
    }
    std::fs::write(FIXTURE, serde_json::to_string(&root).unwrap()).unwrap();
    println!("rewrote {FIXTURE}");
}
