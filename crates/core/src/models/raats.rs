//! Raats (1992) superclass retention curve
//! (Heinen & Bakker 2016), with θ(ψ) from numerically inverting Eq. 1.

use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};
use std::f64::consts::PI;

#[allow(clippy::excessive_precision)] // Lanczos coefficients
fn ln_gamma(z: f64) -> f64 {
    const C: [f64; 8] = [
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    if z < 0.5 {
        // The reflection formula for arguments below 0.5.
        return (PI / (PI * z).sin()).ln() - ln_gamma(1.0 - z);
    }
    let z = z - 1.0;
    let mut x = 0.99999999999980993;
    for (i, c) in C.iter().enumerate() {
        x += c / (z + i as f64 + 1.0);
    }
    let t = z + C.len() as f64 - 0.5;
    0.5 * (2.0 * PI).ln() + (z + 0.5) * t.ln() - t + x.ln()
}

/// Continued-fraction expansion (Numerical Recipes) for the incomplete beta.
fn betacf(a: f64, b: f64, x: f64) -> f64 {
    const FPMIN: f64 = 1e-300;
    let mut c = 1.0;
    let mut d = 1.0 - ((a + b) * x) / (a + 1.0);
    if d.abs() < FPMIN {
        d = FPMIN;
    }
    d = 1.0 / d;
    let mut h = d;
    for m_loop in 1..=200 {
        let m = m_loop as f64;
        let m2 = 2.0 * m;
        let mut aa = (m * (b - m) * x) / ((a + m2 - 1.0) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        h *= d * c;
        aa = (-(a + m) * (a + b + m) * x) / ((a + m2) * (a + m2 + 1.0));
        d = 1.0 + aa * d;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-12 {
            break;
        }
    }
    h
}

/// Regularized incomplete beta function I_x(a, b).
pub fn reg_inc_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let front =
        (a * x.ln() + b * (1.0 - x).ln() - (ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b))).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        (front * betacf(a, b, x)) / a
    } else {
        1.0 - (front * betacf(b, a, 1.0 - x)) / b
    }
}

/// Effective saturation S for a potential ψ, by inverting Raats Eq. 1.
pub fn raats_saturation(p: &Params, psi: f64) -> f64 {
    let h = psi.abs();
    if !(h > 0.0) {
        return 1.0;
    }
    let bs = get(p, "b*");
    let a = get(p, "α");
    let b = get(p, "β");
    let g = get(p, "γ");
    let head_of = |s: f64| bs * ((1.0 - s.powf(a)) / s.powf(b)).powf(g);
    let mut lo = 1e-9;
    let mut hi = 1.0;
    if head_of(lo) < h {
        return lo;
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if head_of(mid) > h {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

pub fn predict(p: &Params, x: &Columns) -> Vec<f64> {
    let psi = col(x, "psi");
    let tr = get(p, "θr");
    let ts = get(p, "θs");
    psi.iter()
        .map(|&v| tr + (ts - tr) * raats_saturation(p, v))
        .collect()
}

pub struct Raats {
    params: Vec<ParamSpec>,
}

impl Raats {
    pub fn new() -> Self {
        Raats {
            params: vec![
                param("θr", 0.05, 0.0, 0.5)
                    .units("m³ m⁻³")
                    .describe("residual volumetric water content"),
                param("θs", 0.45, 0.1, 1.0)
                    .units("m³ m⁻³")
                    .describe("saturated volumetric water content"),
                param("b*", 0.05, 1e-5, 1e5)
                    .units("MPa")
                    .describe("characteristic pressure head (ψ units)"),
                param("α", 5.0, 0.01, 10.0)
                    .units("dimensionless")
                    .describe("Raats shape parameter α"),
                param("β", 5.0, 0.1, 40.0)
                    .units("dimensionless")
                    .describe("Raats shape parameter β (dry end)"),
                param("γ", 0.8, 0.01, 0.999)
                    .units("dimensionless")
                    .describe("Raats shape parameter γ (wet end, 0–1)"),
                param("λ", 0.5, -5.0, 5.0)
                    .units("dimensionless")
                    .fixed()
                    .describe("Mualem tortuosity exponent (conductivity)"),
            ],
        }
    }
}

impl Default for Raats {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for Raats {
    fn name(&self) -> &str {
        "raats"
    }
    fn inputs(&self) -> &[&'static str] {
        &["psi"]
    }
    fn params(&self) -> &[ParamSpec] {
        &self.params
    }
    fn predict(&self, p: &Params, x: &Columns) -> Vec<f64> {
        predict(p, x)
    }
    fn latex(&self) -> Option<String> {
        Some(r"|\psi| = b^{*}\left[\dfrac{1-S^{\alpha}}{S^{\beta}}\right]^{\gamma},\quad S=\dfrac{\theta-\theta_r}{\theta_s-\theta_r}".to_string())
    }
}
