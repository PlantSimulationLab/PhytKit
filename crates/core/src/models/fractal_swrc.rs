//! Fractal retention curve with adsorption
//! and capillarity (Li et al. 2025), with θ(ψ) from numerically inverting s(Se).

use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};
use std::f64::consts::PI;

/// 4λ/π with λ = 0.0728 N/m.
const PREF: f64 = (4.0 * 0.0728) / PI;

/// Matric suction s (MPa, ≥ 0) at an effective saturation Se.
pub fn fractal_suction(p: &Params, se: f64) -> f64 {
    let dd = get(p, "D");
    let rmax = get(p, "r_max");
    let d = get(p, "δ");
    let cc = (1.0 - d) * (1.0 - d);
    let e = (1.0 - dd) / (2.0 - dd);
    let base = 1.0 - (1.0 - se) / cc;
    if base <= 0.0 {
        return f64::INFINITY;
    }
    let b = 1.0 - base.powf(e);
    (PREF * ((2.0 - dd) / (1.0 - dd)) * (1.0 - d) * b) / se / rmax
}

/// Effective saturation Se for a potential ψ, by inverting s(Se).
pub fn fractal_saturation(p: &Params, psi: f64) -> f64 {
    let s = psi.abs();
    let d = get(p, "δ");
    let se_min = 1.0 - (1.0 - d) * (1.0 - d);
    if !(s > 0.0) {
        return 1.0;
    }
    let mut lo = se_min + 1e-9;
    let mut hi = 1.0;
    if fractal_suction(p, lo) < s {
        return lo;
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if fractal_suction(p, mid) > s {
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
    psi.iter().map(|&v| tr + (ts - tr) * fractal_saturation(p, v)).collect()
}

pub struct FractalSwrc {
    params: Vec<ParamSpec>,
}

impl FractalSwrc {
    pub fn new() -> Self {
        FractalSwrc {
            params: vec![
                param("θr", 0.05, 0.0, 0.5).describe("residual volumetric water content"),
                param("θs", 0.45, 0.1, 1.0).describe("saturated volumetric water content"),
                param("D", 1.5, 1.001, 1.999).describe("fractal dimension of the pore-size distribution"),
                param("r_max", 5.0, 1e-3, 1e4).describe("maximum equivalent pore radius (µm)"),
                param("δ", 0.02, 0.0, 0.3).describe("relative adsorptive water-film thickness"),
                param("L", 0.5, -5.0, 5.0).fixed().describe("Mualem tortuosity exponent (conductivity)"),
            ],
        }
    }
}

impl Default for FractalSwrc {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for FractalSwrc {
    fn name(&self) -> &str {
        "fractalSWRC"
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
        Some(
            r"s = \dfrac{4\lambda}{\pi}\dfrac{2-D}{1-D}\dfrac{1-\delta}{r_{max}}\dfrac{1-\left(1-\tfrac{1-S_e}{(1-\delta)^2}\right)^{\frac{1-D}{2-D}}}{S_e}"
                .to_string(),
        )
    }
}
