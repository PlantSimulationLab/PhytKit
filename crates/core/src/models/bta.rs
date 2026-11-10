//! Buckley, Turnbull & Adams (2012) stomatal
//! conductance model, gs = Em·(Q + i0) / (k + b·Q + (Q + i0)·Ds).

use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};

pub fn predict(p: &Params, x: &Columns) -> Vec<f64> {
    let q = col(x, "Q");
    let ds = col(x, "Ds");
    let (em, i0, k, b) = (get(p, "Em"), get(p, "i0"), get(p, "k"), get(p, "b"));
    q.iter()
        .enumerate()
        .map(|(i, &qi)| {
            let di = ds.get(i).copied().unwrap_or(f64::NAN);
            let denom = k + b * qi + (qi + i0) * di;
            if denom > 0.0 {
                (em * (qi + i0)) / denom
            } else {
                f64::NAN
            }
        })
        .collect()
}

pub struct Bta {
    params: Vec<ParamSpec>,
}

impl Bta {
    pub fn new() -> Self {
        Bta {
            params: vec![
                param("Em", 0.2, 0.001, 20.0)
                    .units("mmol m⁻² s⁻¹")
                    .describe(
                        "maximum transpiration rate (gs in mol m⁻² s⁻¹ times Ds in mmol mol⁻¹)",
                    ),
                param("i0", 20.0, 0.0, 500.0)
                    .units("µmol m⁻² s⁻¹")
                    .describe("light compensation offset (µmol m⁻² s⁻¹)"),
                param("k", 100000.0, 1.0, 1e7)
                    .units("µmol m⁻² s⁻¹ mmol mol⁻¹")
                    .describe("denominator constant"),
                param("b", 100.0, 0.0, 20000.0)
                    .units("mmol mol⁻¹")
                    .describe("VPD/light sensitivity"),
            ],
        }
    }
}

impl Default for Bta {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for Bta {
    fn name(&self) -> &str {
        "bta"
    }
    fn inputs(&self) -> &[&'static str] {
        &["Q", "Ds"]
    }
    fn params(&self) -> &[ParamSpec] {
        &self.params
    }
    fn predict(&self, p: &Params, x: &Columns) -> Vec<f64> {
        predict(p, x)
    }
    fn latex(&self) -> Option<String> {
        Some(r"g_s = \frac{E_m\,(Q + i_0)}{k + b\,Q + (Q + i_0)\,D_s}".to_string())
    }
}
