//! Brooks & Corey (1964) retention curve.

use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};

pub fn predict(p: &Params, x: &Columns) -> Vec<f64> {
    let psi = col(x, "psi");
    let tr = get(p, "θr");
    let ts = get(p, "θs");
    let pb = get(p, "ψb");
    let lam = get(p, "λ");
    psi.iter()
        .map(|&v| {
            let h = v.abs();
            let se = if h <= pb { 1.0 } else { (pb / h).powf(lam) };
            tr + (ts - tr) * se
        })
        .collect()
}

pub struct BrooksCorey {
    params: Vec<ParamSpec>,
}

impl BrooksCorey {
    pub fn new() -> Self {
        BrooksCorey {
            params: vec![
                param("θr", 0.05, 0.0, 0.5)
                    .units("m³ m⁻³")
                    .describe("residual volumetric water content"),
                param("θs", 0.45, 0.1, 1.0)
                    .units("m³ m⁻³")
                    .describe("saturated volumetric water content"),
                param("ψb", 0.03, 1e-5, 1e4)
                    .units("MPa")
                    .describe("air-entry (bubbling) potential (ψ units)"),
                param("λ", 0.5, 0.05, 10.0)
                    .units("dimensionless")
                    .describe("pore-size distribution index"),
                param("L", 0.5, -5.0, 5.0)
                    .units("dimensionless")
                    .fixed()
                    .describe("Mualem tortuosity exponent (conductivity)"),
            ],
        }
    }
}

impl Default for BrooksCorey {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for BrooksCorey {
    fn name(&self) -> &str {
        "brooksCorey"
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
        Some(r"S_e = \left(\dfrac{\psi_b}{|\psi|}\right)^{\lambda}\ (|\psi|>\psi_b),\quad \theta=\theta_r+(\theta_s-\theta_r)S_e".to_string())
    }
}
