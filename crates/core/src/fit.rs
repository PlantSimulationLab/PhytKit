//! Bounded Levenberg-Marquardt with Nielsen damping, in scaled
//! coordinates.

use crate::linalg::solve_linear;
use crate::model::{Columns, Model, Params, PenaltyOptions};
use crate::numbers;
use std::collections::{HashMap, HashSet};

#[derive(Default, Clone)]
pub struct FitOptions {
    pub fit_params: Option<Vec<String>>,
    pub init: Params,
    pub max_iter: Option<usize>,
    pub tol: Option<f64>,
    pub finite_diff: bool,
    pub curves: Option<Vec<Vec<usize>>>,
    pub constraints: Option<Vec<String>>,
    pub constraint_weights: Option<HashMap<String, f64>>,
}

#[derive(Clone, Debug)]
pub struct FitResult {
    pub params: Params,
    pub fitted: Vec<String>,
    pub predicted: Vec<f64>,
    pub rmse: f64,
    pub r2: f64,
    pub iterations: usize,
    pub converged: bool,
}

fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    numbers::min(numbers::max(x, lo), hi)
}
fn norm(v: &[f64]) -> f64 {
    v.iter().fold(0.0, |s, x| s + x * x).sqrt()
}

pub fn fit(model: &dyn Model, x: &Columns, y: &[f64], opts: &FitOptions) -> FitResult {
    let n_data = y.len();
    let enabled: Option<HashSet<String>> = opts
        .constraints
        .as_ref()
        .map(|c| c.iter().cloned().collect());
    let use_penalties = model.has_penalties();

    let mut full: Params = model
        .params()
        .iter()
        .map(|p| (p.name.to_string(), p.default))
        .collect();
    for (k, v) in &opts.init {
        full.insert(k.clone(), *v);
    }
    let free: Vec<String> = match &opts.fit_params {
        Some(f) => f.clone(),
        None => model
            .params()
            .iter()
            .filter(|p| !p.fixed)
            .map(|p| p.name.to_string())
            .collect(),
    };
    let m = free.len();
    let spec = |nm: &str| model.params().iter().find(|p| p.name == nm);
    let lb: Vec<f64> = free
        .iter()
        .map(|nm| spec(nm).and_then(|p| p.lb).unwrap_or(f64::NEG_INFINITY))
        .collect();
    let ub: Vec<f64> = free
        .iter()
        .map(|nm| spec(nm).and_then(|p| p.ub).unwrap_or(f64::INFINITY))
        .collect();
    let theta0: Vec<f64> = (0..m)
        .map(|k| {
            clamp(
                full.get(&free[k]).copied().unwrap_or(f64::NAN),
                lb[k],
                ub[k],
            )
        })
        .collect();
    let scale: Vec<f64> = (0..m)
        .map(|k| {
            let v = theta0[k].abs();
            if v > 1e-8 {
                v
            } else if lb[k].is_finite() && ub[k].is_finite() && ub[k] > lb[k] {
                ub[k] - lb[k]
            } else {
                1.0
            }
        })
        .collect();
    let lb_u: Vec<f64> = (0..m).map(|k| lb[k] / scale[k]).collect();
    let ub_u: Vec<f64> = (0..m).map(|k| ub[k] / scale[k]).collect();
    let mut u: Vec<f64> = (0..m).map(|k| theta0[k] / scale[k]).collect();

    let mut xfit = x.clone();
    xfit.insert("Aobs".to_string(), y.to_vec());

    let params_from = |uu: &[f64]| -> Params {
        let mut p = full.clone();
        for k in 0..m {
            p.insert(free[k].clone(), uu[k] * scale[k]);
        }
        p
    };
    let popts = PenaltyOptions {
        curves: opts.curves.as_deref(),
        enabled: enabled.as_ref(),
        weights: opts.constraint_weights.as_ref(),
    };
    // Every residual (penalties included) is divided by the RMS of the data. That leaves the
    // minimizer unchanged but makes the fitter's absolute tolerances independent of the units
    // of y, so data of magnitude 1e-6 fit as well as data of magnitude 1.
    let y_rms = (y.iter().fold(0.0, |s, v| s + v * v) / n_data.max(1) as f64).sqrt();
    let inv_ys = if y_rms.is_finite() && y_rms > 0.0 {
        1.0 / y_rms
    } else {
        1.0
    };
    let residual = |uu: &[f64]| -> Vec<f64> {
        let p = params_from(uu);
        let pred = model.predict(&p, &xfit);
        let mut r: Vec<f64> = (0..n_data)
            .map(|i| (pred.get(i).copied().unwrap_or(f64::NAN) - y[i]) * inv_ys)
            .collect();
        if use_penalties {
            if let Some(pen) = model.penalties(&p, &xfit, &popts) {
                r.extend(pen.into_iter().map(|v| v * inv_ys));
            }
        }
        r
    };
    let sse = |r: &[f64]| r.iter().fold(0.0, |s, v| s + v * v);

    let use_analytic = !opts.finite_diff && model.has_jacobian() && !use_penalties;
    let jacobian = |uu: &[f64], r: &[f64]| -> Vec<Vec<f64>> {
        let n_res = r.len();
        if use_analytic {
            if let Some(jt) = model.jacobian(&params_from(uu), x, &free) {
                let ok = (0..n_data).all(|i| (0..m).all(|j| jt[i][j].is_finite()));
                if ok {
                    return jt
                        .iter()
                        .map(|row| {
                            row.iter()
                                .enumerate()
                                .map(|(k, v)| v * scale[k] * inv_ys)
                                .collect()
                        })
                        .collect();
                }
            }
        }
        let mut jm = vec![vec![0.0; m]; n_res];
        for j in 0..m {
            let mut u2 = uu.to_vec();
            let h = numbers::max(1e-8, 1e-6 * uu[j].abs());
            u2[j] += h;
            let r2 = residual(&u2);
            for i in 0..n_res {
                jm[i][j] = (r2.get(i).copied().unwrap_or(f64::NAN) - r[i]) / h;
            }
        }
        jm
    };
    let normal_eq = |jm: &[Vec<f64>], r: &[f64]| -> (Vec<Vec<f64>>, Vec<f64>) {
        let mut a = vec![vec![0.0; m]; m];
        let mut g = vec![0.0; m];
        for (i, ji) in jm.iter().enumerate() {
            let ri = r[i];
            for p in 0..m {
                g[p] += ji[p] * ri;
                for q in p..m {
                    a[p][q] += ji[p] * ji[q];
                }
            }
        }
        for p in 0..m {
            for q in 0..p {
                a[p][q] = a[q][p];
            }
        }
        (a, g)
    };

    let max_iter = opts.max_iter.unwrap_or(200);
    let eps = opts.tol.unwrap_or(1e-10);
    let tau = 1e-3;
    // Bound tolerance in scaled units, for deciding that a parameter sits on a bound.
    let on_lb = |uu: &[f64], k: usize| uu[k] <= lb_u[k] + 1e-12 * numbers::max(1.0, lb_u[k].abs());
    let on_ub = |uu: &[f64], k: usize| uu[k] >= ub_u[k] - 1e-12 * numbers::max(1.0, ub_u[k].abs());
    // A parameter is active (held on its bound for this step) when it sits on a bound and
    // the descent direction -g points out of the box. The projected gradient is zero there.
    let active_set = |uu: &[f64], gg: &[f64]| -> Vec<bool> {
        (0..m)
            .map(|k| (on_lb(uu, k) && gg[k] > 0.0) || (on_ub(uu, k) && gg[k] < 0.0))
            .collect()
    };
    let proj_grad_small = |uu: &[f64], gg: &[f64]| -> bool {
        let act = active_set(uu, gg);
        (0..m).all(|k| act[k] || gg[k].abs() < eps)
    };

    let mut r = residual(&u);
    let mut cost = sse(&r);
    let (mut a, mut g) = normal_eq(&jacobian(&u, &r), &r);
    let mut mu = tau * (0..m).fold(0.0, |s, k| numbers::max(s, a[k][k]));
    let mut nu = 2.0f64;
    let mut iter = 0;
    let mut converged = cost.is_finite() && (cost == 0.0 || proj_grad_small(&u, &g));
    // Set when the loop ends without meeting a convergence test (all trial steps rejected,
    // damping overflow, singular system). Converged then depends on the projected gradient
    // being small by a looser standard, so a fit that genuinely stalled says so.
    let mut stalled = false;

    while iter < max_iter && !converged && cost.is_finite() {
        iter += 1;
        let act = active_set(&u, &g);
        let inactive: Vec<usize> = (0..m).filter(|&k| !act[k]).collect();
        if inactive.is_empty() {
            // Every free parameter is pinned on a bound with the gradient pointing out:
            // this is a constrained minimum.
            converged = true;
            break;
        }
        let mut accepted = false;
        for _ in 0..30 {
            let alm: Vec<Vec<f64>> = inactive
                .iter()
                .map(|&p| {
                    inactive
                        .iter()
                        .map(|&q| if p == q { a[p][q] + mu } else { a[p][q] })
                        .collect()
                })
                .collect();
            let neg_g: Vec<f64> = inactive.iter().map(|&k| -g[k]).collect();
            let delta = match solve_linear(&alm, &neg_g) {
                Some(d) => {
                    let mut full_d = vec![0.0; m];
                    for (i, &k) in inactive.iter().enumerate() {
                        full_d[k] = d[i];
                    }
                    full_d
                }
                None => {
                    mu *= nu;
                    nu *= 2.0;
                    if !mu.is_finite() {
                        stalled = true;
                        break;
                    }
                    continue;
                }
            };
            if norm(&delta) <= eps * (norm(&u) + eps) {
                converged = true;
                break;
            }
            let cand: Vec<f64> = (0..m)
                .map(|k| clamp(u[k] + delta[k], lb_u[k], ub_u[k]))
                .collect();
            // Predicted reduction of the linear model for the step actually taken (after
            // clipping to the bounds): L(0) - L(s) = -2 g.s - s'As.
            let s: Vec<f64> = (0..m).map(|k| cand[k] - u[k]).collect();
            let mut pred_red = 0.0;
            for p in 0..m {
                pred_red -= 2.0 * g[p] * s[p];
                for q in 0..m {
                    pred_red -= s[p] * a[p][q] * s[q];
                }
            }
            let rc = residual(&cand);
            let cc = sse(&rc);
            let rho = if pred_red > 0.0 {
                (cost - cc) / pred_red
            } else {
                -1.0
            };
            if rho > 0.0 {
                u = cand;
                r = rc;
                cost = cc;
                let (a2, g2) = normal_eq(&jacobian(&u, &r), &r);
                a = a2;
                g = g2;
                mu *= numbers::max(1.0 / 3.0, 1.0 - (2.0 * rho - 1.0).powf(3.0));
                nu = 2.0;
                accepted = true;
                if cost == 0.0 || proj_grad_small(&u, &g) {
                    converged = true;
                }
                break;
            }
            mu *= nu;
            nu *= 2.0;
            if !mu.is_finite() {
                stalled = true;
                break;
            }
        }
        if stalled || (!accepted && !converged) {
            stalled = true;
            break;
        }
    }
    if stalled {
        // No step improves the fit. That is a minimum (to floating-point precision) only if
        // the projected gradient is small relative to the residual and the Jacobian:
        // |g_k| <= 1e-4 * |r| * |J_k| for every free, inactive parameter.
        let jm = jacobian(&u, &r);
        let act = active_set(&u, &g);
        let rn = norm(&r);
        converged = cost.is_finite()
            && (0..m).all(|k| {
                let jn = jm.iter().fold(0.0, |s, row| s + row[k] * row[k]).sqrt();
                act[k] || g[k].abs() <= 1e-4 * rn * jn
            });
    }
    if !cost.is_finite() {
        converged = false;
    }

    let params = params_from(&u);
    // The objective may use the measured A (FvCB's Cc = Ci - Aobs/gm), but the reported
    // predictions and fit statistics are the model's own, as `predict` gives them.
    let predicted = model.predict(&params, x);
    let mut ybar = 0.0;
    for v in y {
        ybar += v;
    }
    ybar /= n_data as f64;
    let (mut sstot, mut data_sse) = (0.0, 0.0);
    for i in 0..n_data {
        sstot += (y[i] - ybar) * (y[i] - ybar);
        let d = predicted.get(i).copied().unwrap_or(f64::NAN) - y[i];
        data_sse += d * d;
    }
    FitResult {
        params,
        fitted: free,
        predicted,
        rmse: (data_sse / n_data as f64).sqrt(),
        r2: if sstot > 0.0 {
            1.0 - data_sse / sstot
        } else {
            f64::NAN
        },
        iterations: iter,
        converged,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{defaults, Columns};
    use crate::models::model_by_name;

    fn data(model: &str, input: &str, xs: Vec<f64>, truth: &[(&str, f64)]) -> (Columns, Vec<f64>) {
        let m = model_by_name(model).unwrap();
        let mut p = defaults(m);
        for (k, v) in truth {
            p.insert(k.to_string(), *v);
        }
        let mut x = Columns::new();
        x.insert(input.to_string(), xs);
        let y = m.predict(&p, &x);
        (x, y)
    }

    #[test]
    fn small_magnitude_responses_fit_like_any_other() {
        let ps: Vec<f64> = (1..=16).map(|i| -0.3 * i as f64).collect();
        let truth = [("Kmax", 1.2e-6), ("P50", -2.5), ("S", 40.0)];
        let (x, y) = data("vulnerability", "P", ps, &truth);
        let mut init = Params::new();
        init.insert("Kmax".into(), 1e-6);
        let opts = FitOptions {
            init,
            ..Default::default()
        };
        let r = fit(model_by_name("vulnerability").unwrap(), &x, &y, &opts);
        assert!(r.converged && r.iterations > 0);
        assert!((r.params["P50"] + 2.5).abs() < 1e-6, "{:?}", r.params);
        assert!((r.params["S"] - 40.0).abs() < 1e-4, "{:?}", r.params);
    }

    #[test]
    fn a_parameter_on_its_bound_doesnt_stall_the_others() {
        // θr = -0.05 is below its bound of 0, so the best fit holds θr at 0.
        let psi: Vec<f64> = (0..25).map(|i| -0.001 * 1.6f64.powi(i)).collect();
        let truth = [("θr", -0.05), ("θs", 0.42), ("α", 2.0), ("n", 1.3)];
        let (x, y) = data("vanGenuchten", "psi", psi, &truth);
        let m = model_by_name("vanGenuchten").unwrap();
        let free = fit(m, &x, &y, &FitOptions::default());
        let mut init = Params::new();
        init.insert("θr".into(), 0.0);
        let pinned = fit(
            m,
            &x,
            &y,
            &FitOptions {
                fit_params: Some(vec!["θs".into(), "α".into(), "n".into()]),
                init,
                ..Default::default()
            },
        );
        assert!(free.converged && pinned.converged);
        assert_eq!(free.params["θr"], 0.0);
        for k in ["θs", "α", "n"] {
            let (a, b) = (free.params[k], pinned.params[k]);
            assert!((a - b).abs() <= 1e-5 * b.abs(), "{k}: free {a}, pinned {b}");
        }
    }

    #[test]
    fn non_finite_data_is_never_converged() {
        let psi: Vec<f64> = (0..10).map(|i| -0.01 * (i + 1) as f64).collect();
        let (x, mut y) = data("vanGenuchten", "psi", psi, &[]);
        y[3] = f64::NAN;
        let r = fit(
            model_by_name("vanGenuchten").unwrap(),
            &x,
            &y,
            &FitOptions::default(),
        );
        assert!(!r.converged);
    }
}
