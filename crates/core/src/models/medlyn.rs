//! Medlyn et al. (2011) optimal stomatal
//! conductance, gs = gs0 + 1.6·(1 + g1/√D)·A/Cs, with D floored at 1e-5 kPa.

use crate::numbers;
use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};

const D_MIN: f64 = 1e-5;

pub fn predict(p: &Params, x: &Columns) -> Vec<f64> {
    let (a, cs, d) = (col(x, "A"), col(x, "Cs"), col(x, "D"));
    let at = |v: &[f64], i: usize| v.get(i).copied().unwrap_or(f64::NAN);
    let (gs0, g1) = (get(p, "gs0"), get(p, "g1"));
    (0..a.len())
        .map(|i| {
            let di = numbers::max(at(d, i), D_MIN);
            gs0 + 1.6 * (1.0 + g1 / di.sqrt()) * (a[i] / at(cs, i))
        })
        .collect()
}

pub struct Medlyn {
    params: Vec<ParamSpec>,
}

impl Medlyn {
    pub fn new() -> Self {
        Medlyn {
            params: vec![
                param("gs0", 0.01, 0.0, 1.0).describe("residual conductance (mol m⁻² s⁻¹)"),
                param("g1", 3.0, 0.01, 30.0).describe("slope, sensitivity to VPD (√kPa)"),
            ],
        }
    }
}

impl Default for Medlyn {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for Medlyn {
    fn name(&self) -> &str {
        "medlyn"
    }
    fn inputs(&self) -> &[&'static str] {
        &["A", "Cs", "D"]
    }
    fn params(&self) -> &[ParamSpec] {
        &self.params
    }
    fn predict(&self, p: &Params, x: &Columns) -> Vec<f64> {
        predict(p, x)
    }
    fn latex(&self) -> Option<String> {
        Some(r"g_s = g_{s,0} + 1.6\left(1 + \frac{g_1}{\sqrt{D}}\right)\frac{A}{C_s}".to_string())
    }
}
