//! Pressure-volume curve
//! (Sack, John & Buckley 2018; Tyree & Hammel 1972).

use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};
use crate::numbers;
use indexmap::IndexMap;

pub fn predict(p: &Params, x: &Columns) -> Vec<f64> {
    let w = col(x, "w");
    let pi_o = get(p, "πo");
    let wtlp = get(p, "wtlp");
    let eps = get(p, "ε");
    w.iter()
        .map(|&wi| {
            let rel = numbers::max(0.0, (wi - wtlp) / (1.0 - wtlp));
            let turgor = pi_o * rel.powf(eps);
            let osmotic = -pi_o / wi;
            turgor + osmotic
        })
        .collect()
}

pub struct Pvcurve {
    params: Vec<ParamSpec>,
}

impl Pvcurve {
    pub fn new() -> Self {
        Pvcurve {
            params: vec![
                param("πo", 2.0, 0.1, 5.0).describe("osmotic pressure at full turgor (MPa)"),
                param("wtlp", 0.85, 0.5, 0.99)
                    .describe("relative water content at turgor loss point"),
                param("ε", 1.0, 1.0, 3.0).describe("wall-elasticity exponent on turgor"),
            ],
        }
    }
}

impl Default for Pvcurve {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for Pvcurve {
    fn name(&self) -> &str {
        "pvcurve"
    }
    fn inputs(&self) -> &[&'static str] {
        &["w"]
    }
    fn params(&self) -> &[ParamSpec] {
        &self.params
    }
    fn predict(&self, p: &Params, x: &Columns) -> Vec<f64> {
        predict(p, x)
    }
    /// Turgor is zero at w = wtlp, so Ψ there is purely osmotic: −πo / wtlp.
    fn derived(&self, p: &Params) -> Option<IndexMap<String, f64>> {
        let mut d = IndexMap::new();
        d.insert("psi_tlp".to_string(), -get(p, "πo") / get(p, "wtlp"));
        Some(d)
    }
    fn latex(&self) -> Option<String> {
        Some(r"\Psi = \pi_o\left[\max\!\left(0,\tfrac{w-w_{tlp}}{1-w_{tlp}}\right)\right]^{\varepsilon} - \dfrac{\pi_o}{w}".to_string())
    }
}
