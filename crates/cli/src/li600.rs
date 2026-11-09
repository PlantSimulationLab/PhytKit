//! `pk li600 correct` and `pk li600 estimate`.

use crate::args::{CliError, ParsedArgs, Result};
use crate::io::{load_file, write_out};
use crate::{object, Output};
use phytkit_core::numbers::{self, json as num};
use serde_json::{json, Value};
use phytkit_core::li600::{correct_li600, corrected_columns_csv, corrected_full_csv, parse_li600, Li600Options};
use phytkit_core::model::Params;
use phytkit_core::poro::{parse_param_text, parse_poro_file, poro_columns_csv, poro_full_csv, solve_poro_a, PoroModel, PoroOptions, DEFAULT_MIN_GSW, PORO_MODELS};

const LI600_HINT: &str = "pk li600 reads LI-600 porometer exports. For gas-exchange curves (A–Ci), use pk fit photosynthesis.";
const PARAMS_HINT: &str = "Use the JSON from pk fit, a CSV with a column per parameter, a Parameter/Value table, or lines of name = value.";

fn mean(a: &[f64]) -> f64 {
    if a.is_empty() { f64::NAN } else { a.iter().fold(0.0, |s, v| s + v) / a.len() as f64 }
}

pub fn correct(args: &ParsedArgs) -> Result<Output> {
    let path = &args.positional[0];
    let sidedness = args.num("sidedness", 1.0)?;
    let c = args.num("thermal-conductance", 0.007)?;
    let file = load_file(path)?;
    let parsed = parse_li600(&file.text).map_err(|e| CliError::data(e).hint(LI600_HINT))?;
    let corrected = correct_li600(&parsed.rows, Li600Options { sidedness, thermal_conductance: c });
    let valid: Vec<_> = corrected.iter().filter(|r| r.converged && r.overestimate.is_finite()).collect();
    let out = args.str("out")?;
    if let Some(o) = &out {
        write_out(o, &corrected_full_csv(&parsed, &corrected))?;
    }
    let format = args.str("format")?.unwrap_or_else(|| "json".into());
    if format == "csv" {
        return Ok(Output::Raw(corrected_columns_csv(&corrected)));
    }
    let overestimates: Vec<f64> = valid.iter().map(|r| r.overestimate).collect();
    let measured: Vec<f64> = valid.iter().map(|r| r.measured).collect();
    let mut o = object(json!({
        "file": file.name,
        "converted": file.converted,
        "n": parsed.n,
        "solved": valid.len(),
        "settings": { "sidedness": sidedness, "thermalConductance": c },
        "summary": {
            "meanOverestimatePct": num(numbers::round_to(mean(&overestimates), 3)),
            "maxOverestimatePct": num(numbers::round_to(numbers::max_of(&overestimates), 3)),
            "maxMeasuredGsw": num(numbers::round_to(numbers::max_of(&measured), 6))
        },
        "warnings": parsed.warnings,
        "out": out
    }));
    if args.bool("rows", false) {
        let rows: Vec<Value> = corrected
            .iter()
            .map(|r| {
                json!({
                    "gsw_measured": num(numbers::round_to(r.measured, 6)),
                    "gsw_corrected": num(numbers::round_to(r.corrected, 6)),
                    "overestimate_pct": num(numbers::round_to(r.overestimate, 3)),
                    "E_corrected_mmol": num(numbers::round_to(r.e * 1e3, 6)),
                    "Ta_chamb_corrected": num(numbers::round_to(r.ta_chamb, 4)),
                    "T_out_corrected": num(numbers::round_to(r.tout, 4)),
                    "converged": r.converged
                })
            })
            .collect();
        o.insert("rows".into(), Value::Array(rows));
    }
    Ok(Output::Json(o))
}

/// Parameters = model defaults, then the --params file, then --set k=v.
fn resolve_params(args: &ParsedArgs, model_name: &str) -> Result<(&'static PoroModel, Params, Vec<String>, Option<String>)> {
    let model = PORO_MODELS.iter().find(|m| m.name == model_name).ok_or_else(|| {
        CliError::usage("unknown_model", format!("Unknown model \"{model_name}\" for pk li600 estimate.")).hint(format!("Models: {}.", PORO_MODELS.iter().map(|m| m.name).collect::<Vec<_>>().join(", ")))
    })?;
    let mut params = model.params();
    let known = model.param_names();
    let mut ignored = Vec::new();
    let pfile = args.str("params")?;
    if let Some(pf) = &pfile {
        let text = load_file(pf)?.text;
        let found = parse_param_text(&text).map_err(|e| CliError::data(format!("--params {pf}: {e}")).hint(PARAMS_HINT))?;
        for (k, v) in &found {
            if known.contains(&k.as_str()) {
                params.insert(k.clone(), *v);
            } else {
                ignored.push(k.clone());
            }
        }
        if found.is_empty() {
            return Err(CliError::data(format!("No parameters found in {pf}.")).hint(PARAMS_HINT));
        }
    }
    for (k, v) in args.kv("set")? {
        if !known.contains(&k.as_str()) {
            return Err(crate::options::unknown_parameter("--set", &k, model.name, &known));
        }
        params.insert(k, v);
    }
    Ok((model, params, ignored, pfile))
}

pub fn estimate(args: &ParsedArgs) -> Result<Output> {
    let path = &args.positional[0];
    let model_name = args.str("model")?.unwrap_or_else(|| "fvcb".into());
    let (model, params, ignored, source) = resolve_params(args, &model_name)?;
    let ca = args.num("ca", 420.0)?;
    let absorptance = args.num("absorptance", 0.85)?;
    let min_gsw = args.num("min-gsw", DEFAULT_MIN_GSW)?;
    let file = load_file(path)?;
    let parsed = parse_poro_file(&file.text).map_err(|e| CliError::data(e).hint(LI600_HINT))?;
    let results = solve_poro_a(model, &params, &parsed.rows, &PoroOptions { ca, absorptance, min_gsw });
    let ok: Vec<_> = results.iter().filter(|r| r.ok).collect();
    let out = args.str("out")?;
    if let Some(o) = &out {
        write_out(o, &poro_full_csv(&parsed, &results))?;
    }
    let format = args.str("format")?.unwrap_or_else(|| "json".into());
    if format == "csv" {
        return Ok(Output::Raw(poro_columns_csv(&results)));
    }
    let gm = model.gm(&params);
    let a: Vec<f64> = ok.iter().map(|r| r.a).collect();
    let ci: Vec<f64> = ok.iter().map(|r| r.ci).collect();
    let param_json: serde_json::Map<String, Value> = params.iter().map(|(k, v)| (k.clone(), num(*v))).collect();
    let mut o = object(json!({
        "file": file.name,
        "converted": file.converted,
        "n": parsed.n,
        "solved": ok.len(),
        "usedCorrectedGsw": parsed.used_corrected,
        "model": model.name,
        "params": param_json,
        "paramsSource": source,
        "ignoredParams": ignored,
        "settings": { "Ca": ca, "absorptance": absorptance, "minGsw": min_gsw, "gm": gm, "series": gm.is_some() },
        "clamped": results.iter().filter(|r| r.clamped).count(),
        "summary": {
            "meanA": num(numbers::round_to(mean(&a), 4)),
            "maxA": num(numbers::round_to(numbers::max_of(&a), 4)),
            "meanCi": num(numbers::round_to(mean(&ci), 2)),
            "meanCiCa": num(numbers::round_to(mean(&ci) / ca, 4))
        },
        "warnings": parsed.warnings,
        "out": out
    }));
    if args.bool("rows", false) {
        let rows: Vec<Value> = results
            .iter()
            .map(|r| {
                json!({
                    "A": num(numbers::round_to(r.a, 4)),
                    "Ci": num(numbers::round_to(r.ci, 3)),
                    "Cc": num(numbers::round_to(r.cc, 3)),
                    "gsw": num(numbers::round_to(r.gsw, 6)),
                    "Qabs": num(numbers::round_to(r.qabs, 2)),
                    "Tleaf": num(r.tleaf),
                    "ETR": num(r.etr),
                    "ok": r.ok,
                    "clamped": r.clamped
                })
            })
            .collect();
        o.insert("rows".into(), Value::Array(rows));
    }
    Ok(Output::Json(o))
}
