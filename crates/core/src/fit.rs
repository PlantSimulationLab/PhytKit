//! Bounded Levenberg-Marquardt with Nielsen damping, in scaled
//! coordinates.

use crate::numbers;
use crate::linalg::solve_linear;
use crate::model::{Columns, Model, Params, PenaltyOptions};
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
fn max_abs(v: &[f64]) -> f64 {
    v.iter().fold(0.0, |s, x| numbers::max(s, x.abs()))
}

pub fn fit(model: &dyn Model, x: &Columns, y: &[f64], opts: &FitOptions) -> FitResult {
    let n_data = y.len();
    let enabled: Option<HashSet<String>> = opts.constraints.as_ref().map(|c| c.iter().cloned().collect());
    let use_penalties = model.has_penalties();

    let mut full: Params = model.params().iter().map(|p| (p.name.to_string(), p.default)).collect();
    for (k, v) in &opts.init {
        full.insert(k.clone(), *v);
    }
    let free: Vec<String> = match &opts.fit_params {
        Some(f) => f.clone(),
        None => model.params().iter().filter(|p| !p.fixed).map(|p| p.name.to_string()).collect(),
    };
    let m = free.len();
    let spec = |nm: &str| model.params().iter().find(|p| p.name == nm);
    let lb: Vec<f64> = free.iter().map(|nm| spec(nm).and_then(|p| p.lb).unwrap_or(f64::NEG_INFINITY)).collect();
    let ub: Vec<f64> = free.iter().map(|nm| spec(nm).and_then(|p| p.ub).unwrap_or(f64::INFINITY)).collect();
    let theta0: Vec<f64> = (0..m).map(|k| clamp(full.get(&free[k]).copied().unwrap_or(f64::NAN), lb[k], ub[k])).collect();
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
    let popts = PenaltyOptions { curves: opts.curves.as_deref(), enabled: enabled.as_ref(), weights: opts.constraint_weights.as_ref() };
    let residual = |uu: &[f64]| -> Vec<f64> {
        let p = params_from(uu);
        let pred = model.predict(&p, &xfit);
        let mut r: Vec<f64> = (0..n_data).map(|i| pred.get(i).copied().unwrap_or(f64::NAN) - y[i]).collect();
        if use_penalties {
            if let Some(pen) = model.penalties(&p, &xfit, &popts) {
                r.extend(pen);
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
                    return jt.iter().map(|row| row.iter().enumerate().map(|(k, v)| v * scale[k]).collect()).collect();
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

    let mut r = residual(&u);
    let mut cost = sse(&r);
    let (mut a, mut g) = normal_eq(&jacobian(&u, &r), &r);
    let mut mu = tau * (0..m).fold(0.0, |s, k| numbers::max(s, a[k][k]));
    let mut nu = 2.0f64;
    let mut iter = 0;
    let mut converged = max_abs(&g) < eps;

    while iter < max_iter && !converged {
        iter += 1;
        let mut accepted = false;
        for _ in 0..30 {
            let alm: Vec<Vec<f64>> = a.iter().enumerate().map(|(p, row)| row.iter().enumerate().map(|(q, v)| if p == q { v + mu } else { *v }).collect()).collect();
            let neg_g: Vec<f64> = g.iter().map(|v| -v).collect();
            let delta = match solve_linear(&alm, &neg_g) {
                Some(d) => d,
                None => {
                    mu *= nu;
                    nu *= 2.0;
                    if !mu.is_finite() {
                        converged = true;
                        break;
                    }
                    continue;
                }
            };
            if norm(&delta) <= eps * (norm(&u) + eps) {
                converged = true;
                break;
            }
            let cand: Vec<f64> = (0..m).map(|k| clamp(u[k] + delta[k], lb_u[k], ub_u[k])).collect();
            let rc = residual(&cand);
            let cc = sse(&rc);
            let mut pred_red = 0.0;
            for k in 0..m {
                pred_red += delta[k] * (mu * delta[k] - g[k]);
            }
            let rho = if pred_red > 0.0 { (cost - cc) / pred_red } else { -1.0 };
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
                if max_abs(&g) < eps {
                    converged = true;
                }
                break;
            }
            mu *= nu;
            nu *= 2.0;
            if !mu.is_finite() {
                converged = true;
                break;
            }
        }
        if !accepted && !converged {
            converged = true;
            break;
        }
    }

    let params = params_from(&u);
    let predicted = model.predict(&params, &xfit);
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
        r2: if sstot > 0.0 { 1.0 - data_sse / sstot } else { f64::NAN },
        iterations: iter,
        converged,
    }
}
