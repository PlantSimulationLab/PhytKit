//! Empirical photosynthesis model.
//!
//! Inputs: Ci (umol/mol), Q (absorbed PPFD), T (leaf temperature in Kelvin).
//! The dual-number Jacobian is exported but not attached to the
//! model, so `fit` uses finite differences.

use crate::dual::{d_add, d_add_s, d_const, d_div, d_mul, d_mul_s, d_s_div, d_s_sub, d_sub, d_var, Dual};
use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};
use crate::tresp::R_GAS;
use std::collections::HashMap;

#[inline]
fn at(v: &[f64], i: usize) -> f64 {
    v.get(i).copied().unwrap_or(f64::NAN)
}

pub use crate::tresp::tresp;
use crate::dual::{d_exp, d_log};

/// Dual-number version of `tresp` (T is a constant).
fn tresp_dual(t: f64, dha: &Dual, dhd: &Dual, topt: &Dual) -> Dual {
    let rinv = 1.0 / R_GAS;
    let c = d_log(&d_add_s(&d_div(dhd, dha), -1.0));
    let a1 = d_mul_s(dhd, rinv);
    let num = d_add_s(&d_exp(&d_sub(&d_mul(&a1, &d_add_s(&d_s_div(1.0, topt), -1.0 / 298.0)), &c)), 1.0);
    let den = d_add_s(&d_exp(&d_sub(&d_mul(&a1, &d_add_s(&d_s_div(1.0, topt), -1.0 / t)), &c)), 1.0);
    let front = d_exp(&d_mul_s(dha, rinv * (1.0 / 298.0 - 1.0 / t)));
    d_mul(&front, &d_div(&num, &den))
}

/// Analytic Jacobian via forward-mode dual numbers.
pub fn empirical_a_jacobian(p_in: &Params, x: &Columns, wrt: &[String]) -> Vec<Vec<f64>> {
    let mut p = p_in.clone();
    if get(&p, "Amax_Topt") < 200.0 {
        let v = get(&p, "Amax_Topt") + 273.15;
        p.insert("Amax_Topt".to_string(), v);
    }
    let m = wrt.len();
    // `new Map(wrt.map(...))`: a repeated name keeps its last index.
    let mut idx: HashMap<&str, usize> = HashMap::new();
    for (i, nm) in wrt.iter().enumerate() {
        idx.insert(nm.as_str(), i);
    }
    let par = |name: &str| -> Dual {
        match idx.get(name) {
            None => d_const(get(&p, name), m),
            Some(&i) => d_var(get(&p, name), i, m),
        }
    };
    let is_v = |name: &str| idx.contains_key(name);

    let (amax25, rd25, gamma_s25) = (par("Amax25"), par("Rd25"), par("GammaS25"));
    let (kco25, kq25) = (par("Kco25"), par("Kq25"));
    let (amax_dha, amax_dhd, amax_topt) = (par("Amax_dHa"), par("Amax_dHd"), par("Amax_Topt"));
    let (rd_dha, gamma_s_dha, kq_dha, kco_dha) = (par("Rd_dHa"), par("GammaS_dHa"), par("Kq_dHa"), par("Kco_dHa"));
    let gm = par("gm");
    let use_gm = gm.v > 0.0 && gm.v < 99.0;

    let amax_var = is_v("Amax_dHa") || is_v("Amax_dHd") || is_v("Amax_Topt");
    let (kco_var, kq_var, gam_var, rd_var) = (is_v("Kco_dHa"), is_v("Kq_dHa"), is_v("GammaS_dHa"), is_v("Rd_dHa"));
    let (d500, d1000) = (d_const(500.0, m), d_const(1000.0, m));

    let (ci, q, t) = (col(x, "Ci"), col(x, "Q"), col(x, "T"));
    let n = ci.len();
    let mut jac = Vec::with_capacity(n);
    for k in 0..n {
        let (ti, qi, cik) = (at(t, k), at(q, k), ci[k]);
        let a_t = if amax_var {
            d_mul(&amax25, &tresp_dual(ti, &amax_dha, &amax_dhd, &amax_topt))
        } else {
            d_mul_s(&amax25, tresp(ti, get(&p, "Amax_dHa"), get(&p, "Amax_dHd"), get(&p, "Amax_Topt")))
        };
        let kco = if kco_var {
            d_mul(&kco25, &tresp_dual(ti, &kco_dha, &d500, &d1000))
        } else {
            d_mul_s(&kco25, tresp(ti, get(&p, "Kco_dHa"), 500.0, 1000.0))
        };
        let kq = if kq_var {
            d_mul(&kq25, &tresp_dual(ti, &kq_dha, &d500, &d1000))
        } else {
            d_mul_s(&kq25, tresp(ti, get(&p, "Kq_dHa"), 500.0, 1000.0))
        };
        let gam = if gam_var {
            d_mul(&gamma_s25, &tresp_dual(ti, &gamma_s_dha, &d500, &d1000))
        } else {
            d_mul_s(&gamma_s25, tresp(ti, get(&p, "GammaS_dHa"), 500.0, 1000.0))
        };
        let rd = if rd_var {
            d_mul(&rd25, &tresp_dual(ti, &rd_dha, &d500, &d1000))
        } else {
            d_mul_s(&rd25, tresp(ti, get(&p, "Rd_dHa"), 500.0, 1000.0))
        };
        let a_qt = d_mul(&a_t, &d_s_div(qi, &d_add_s(&kq, qi)));

        let cc = if use_gm {
            let mut cc = d_s_sub(cik, &d_s_div(20.0, &gm));
            for _ in 0..20 {
                let a = d_sub(&d_mul(&a_qt, &d_div(&d_sub(&cc, &gam), &d_add(&cc, &kco))), &rd);
                let ccn = d_s_sub(cik, &d_div(&a, &gm));
                if (ccn.v - cc.v).abs() < 1e-6 {
                    break;
                }
                cc = ccn;
            }
            cc
        } else {
            d_const(cik, m)
        };
        let a_final = d_sub(&d_mul(&a_qt, &d_div(&d_sub(&cc, &gam), &d_add(&cc, &kco))), &rd);
        jac.push(a_final.d);
    }
    jac
}

/// Net assimilation at every point (`empiricalA.predict`).
pub fn predict(p_in: &Params, x: &Columns) -> Vec<f64> {
    let mut p = p_in.clone();
    if get(&p, "Amax_Topt") < 200.0 {
        let v = get(&p, "Amax_Topt") + 273.15;
        p.insert("Amax_Topt".to_string(), v);
    }
    let (ci, q, t) = (col(x, "Ci"), col(x, "Q"), col(x, "T"));
    let n = ci.len();
    let mut out = vec![0.0; n];
    let gm = get(&p, "gm");
    let use_gm = gm > 0.0 && gm < 99.0;
    let (amax25, amax_dha, amax_dhd, amax_topt) = (get(&p, "Amax25"), get(&p, "Amax_dHa"), get(&p, "Amax_dHd"), get(&p, "Amax_Topt"));
    let (kco25, kco_dha) = (get(&p, "Kco25"), get(&p, "Kco_dHa"));
    let (kq25, kq_dha) = (get(&p, "Kq25"), get(&p, "Kq_dHa"));
    let (gs25, gs_dha) = (get(&p, "GammaS25"), get(&p, "GammaS_dHa"));
    let (rd25, rd_dha) = (get(&p, "Rd25"), get(&p, "Rd_dHa"));

    for i in 0..n {
        let ti = at(t, i);
        let qi = at(q, i);
        let a_t = amax25 * tresp(ti, amax_dha, amax_dhd, amax_topt);
        let kco = kco25 * tresp(ti, kco_dha, 500.0, 1000.0);
        let kq = kq25 * tresp(ti, kq_dha, 500.0, 1000.0);
        let gamma = gs25 * tresp(ti, gs_dha, 500.0, 1000.0);
        let rd = rd25 * tresp(ti, rd_dha, 500.0, 1000.0);
        let a_qt = (a_t * qi) / (qi + kq);

        let mut cc = ci[i];
        if use_gm {
            let mut lo = -kco + 1e-3;
            let mut hi = ci[i] + 2000.0;
            let mut c = if ci[i] > lo { ci[i] } else { 0.5 * (lo + hi) };
            for _ in 0..40 {
                let denom = c + kco;
                let a = a_qt * ((c - gamma) / denom) - rd;
                let fc = c - ci[i] + a / gm;
                if fc > 0.0 {
                    hi = c;
                } else {
                    lo = c;
                }
                let fp = 1.0 + (a_qt * (gamma + kco)) / (denom * denom) / gm;
                let mut cn = c - fc / fp;
                if !(cn > lo && cn < hi) {
                    cn = 0.5 * (lo + hi);
                }
                if (cn - c).abs() < 1e-9 * (1.0 + c.abs()) {
                    c = cn;
                    break;
                }
                c = cn;
            }
            cc = c;
        }
        out[i] = a_qt * ((cc - gamma) / (cc + kco)) - rd;
    }
    out
}

pub struct EmpiricalA {
    params: Vec<ParamSpec>,
}

impl EmpiricalA {
    pub fn new() -> Self {
        EmpiricalA {
            params: vec![
                param("Amax25", 25.0, 1.0, 200.0).describe("max assimilation at 25C"),
                param("Rd25", 2.5, 0.0, 10.0).describe("dark respiration at 25C"),
                param("GammaS25", 42.75, 10.0, 120.0).describe("CO2 compensation point at 25C"),
                param("Kco25", 700.0, 50.0, 3000.0).describe("CO2 half-saturation at 25C"),
                param("Kq25", 50.0, 5.0, 2000.0).describe("light half-saturation at 25C"),
                param("Amax_dHa", 50.0, 1.0, 200.0).describe("Amax activation energy"),
                param("Amax_dHd", 200.0, 100.0, 500.0).describe("Amax deactivation energy"),
                param("Amax_Topt", 308.12, 288.0, 330.0).describe("Amax optimum temp (K)"),
                param("Rd_dHa", 46.39, 1.0, 200.0).describe("Rd activation energy"),
                param("GammaS_dHa", 37.83, 1.0, 200.0),
                param("Kq_dHa", 70.0, 1.0, 200.0),
                param("Kco_dHa", 79.43, 1.0, 200.0),
                param("gm", 0.25, 0.01, 99.0).describe("mesophyll conductance"),
            ],
        }
    }
}

impl Default for EmpiricalA {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for EmpiricalA {
    fn name(&self) -> &str {
        "empiricalA"
    }
    fn inputs(&self) -> &[&'static str] {
        &["Ci", "Q", "T"]
    }
    fn params(&self) -> &[ParamSpec] {
        &self.params
    }
    fn predict(&self, p: &Params, x: &Columns) -> Vec<f64> {
        predict(p, x)
    }
    fn latex(&self) -> Option<String> {
        Some(r"A = A_{max}(T)\,\frac{Q}{Q + K_q(T)}\cdot\frac{C_i - \Gamma^*(T)}{C_i + K_{co}(T)} - R_d(T)".to_string())
    }
}
