//! A light-response curve, `A = Amax * Q / (Q + Kq) - Rd`, with exact derivatives:
//!
//! ```text
//! d/dAmax:  Q / (Q + Kq)
//! d/dKq:    -((Amax * Q) / ((Q + Kq) ^ 2))
//! d/dRd:    -1
//! d/dQ:     ((1 / (Q + Kq)) - (((1 / (Q + Kq)) ^ 2) * Q)) * Amax
//! other:    0
//! ```

use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};

pub fn predict(p: &Params, x: &Columns) -> Vec<f64> {
    let (amax, kq, rd) = (get(p, "Amax"), get(p, "Kq"), get(p, "Rd"));
    col(x, "Q")
        .iter()
        .map(|&q| ((amax * q) / (q + kq)) - rd)
        .collect()
}

/// The derivatives above, evaluated at every point (rows = points).
pub fn jacobian(p: &Params, x: &Columns, wrt: &[String]) -> Vec<Vec<f64>> {
    let (amax, kq) = (get(p, "Amax"), get(p, "Kq"));
    col(x, "Q")
        .iter()
        .map(|&q| {
            wrt.iter()
                .map(|name| match name.as_str() {
                    "Amax" => q / (q + kq),
                    "Kq" => -((amax * q) / (q + kq).powf(2.0)),
                    "Rd" => -1.0,
                    "Q" => ((1.0 / (q + kq)) - ((1.0 / (q + kq)).powf(2.0) * q)) * amax,
                    _ => 0.0,
                })
                .collect()
        })
        .collect()
}

pub struct LightResponse {
    params: Vec<ParamSpec>,
}

impl LightResponse {
    pub fn new() -> Self {
        LightResponse {
            params: vec![
                param("Amax", 25.0, 1.0, 200.0),
                param("Kq", 300.0, 5.0, 2000.0),
                param("Rd", 1.5, 0.0, 10.0),
            ],
        }
    }
}

impl Default for LightResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for LightResponse {
    fn name(&self) -> &str {
        "lightResponse"
    }
    fn inputs(&self) -> &[&'static str] {
        &["Q"]
    }
    fn params(&self) -> &[ParamSpec] {
        &self.params
    }
    fn predict(&self, p: &Params, x: &Columns) -> Vec<f64> {
        predict(p, x)
    }
    fn has_jacobian(&self) -> bool {
        true
    }
    fn jacobian(&self, p: &Params, x: &Columns, wrt: &[String]) -> Option<Vec<Vec<f64>>> {
        Some(jacobian(p, x, wrt))
    }
    fn latex(&self) -> Option<String> {
        Some(r"\frac{ Amax\cdot Q}{\left( Q+ Kq\right)}- Rd".to_string())
    }
}
