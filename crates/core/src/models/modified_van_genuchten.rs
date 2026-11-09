//! Modified van Genuchten
//! retention curve (Troch 1993), m = 1 + 1/n₁.

use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};

pub fn predict(p: &Params, x: &Columns) -> Vec<f64> {
    let psi = col(x, "psi");
    let tr = get(p, "θr");
    let ts = get(p, "θs");
    let a1 = get(p, "α1");
    let n1 = get(p, "n1");
    let m1 = 1.0 + 1.0 / n1;
    psi.iter()
        .map(|&v| {
            let se = (1.0 + (a1 * v.abs()).powf(n1)).powf(-m1);
            tr + (ts - tr) * se
        })
        .collect()
}

pub struct ModifiedVanGenuchten {
    params: Vec<ParamSpec>,
}

impl ModifiedVanGenuchten {
    pub fn new() -> Self {
        ModifiedVanGenuchten {
            params: vec![
                param("θr", 0.05, 0.0, 0.5).describe("residual volumetric water content"),
                param("θs", 0.45, 0.1, 1.0).describe("saturated volumetric water content"),
                param("α1", 1.0, 1e-4, 1e5).describe("MVG retention scale parameter (ψ⁻¹)"),
                param("n1", 1.5, 0.2, 12.0).describe("MVG retention shape parameter"),
                param("α2", 1.0, 1e-4, 1e5).fixed().describe("MVG-RHC conductivity scale parameter (ψ⁻¹)"),
                param("n2", 1.5, 0.2, 12.0).fixed().describe("MVG-RHC conductivity shape parameter"),
            ],
        }
    }
}

impl Default for ModifiedVanGenuchten {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for ModifiedVanGenuchten {
    fn name(&self) -> &str {
        "modifiedVanGenuchten"
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
        Some(r"\theta = \theta_r + (\theta_s-\theta_r)\left[1+(\alpha_1|\psi|)^{n_1}\right]^{-(1+1/n_1)}".to_string())
    }
}
