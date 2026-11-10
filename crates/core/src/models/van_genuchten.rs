//! Van Genuchten (1980) retention curve.

use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};

pub fn predict(p: &Params, x: &Columns) -> Vec<f64> {
    let psi = col(x, "psi");
    let tr = get(p, "θr");
    let ts = get(p, "θs");
    let a = get(p, "α");
    let n = get(p, "n");
    let m = 1.0 - 1.0 / n;
    psi.iter()
        .map(|&v| {
            let h = v.abs();
            let se = (1.0 + (a * h).powf(n)).powf(-m);
            tr + (ts - tr) * se
        })
        .collect()
}

pub struct VanGenuchten {
    params: Vec<ParamSpec>,
}

impl VanGenuchten {
    pub fn new() -> Self {
        VanGenuchten {
            params: vec![
                param("θr", 0.05, 0.0, 0.5)
                    .units("m³ m⁻³")
                    .describe("residual volumetric water content"),
                param("θs", 0.45, 0.1, 1.0)
                    .units("m³ m⁻³")
                    .describe("saturated volumetric water content"),
                param("α", 1.0, 1e-4, 1e5)
                    .units("MPa⁻¹")
                    .describe("inverse air-entry pressure (ψ⁻¹)"),
                param("n", 1.5, 1.01, 12.0)
                    .units("dimensionless")
                    .describe("pore-size distribution parameter"),
            ],
        }
    }
}

impl Default for VanGenuchten {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for VanGenuchten {
    fn name(&self) -> &str {
        "vanGenuchten"
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
        Some(r"\theta = \theta_r + (\theta_s-\theta_r)\left[1+(\alpha|\psi|)^{n}\right]^{-m},\; m=1-\tfrac{1}{n}".to_string())
    }
}
