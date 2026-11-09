//! The model interface.
//!
//! Parameter values and data columns are insertion-ordered maps, because their order
//! shows up in the JSON output.

use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

/// Named parameter values (`ParamValues`).
pub type Params = IndexMap<String, f64>;
/// Named data columns of equal length (`DataColumns`).
pub type Columns = IndexMap<String, Vec<f64>>;

#[derive(Clone, Debug)]
pub struct ParamSpec {
    pub name: &'static str,
    pub default: f64,
    pub lb: Option<f64>,
    pub ub: Option<f64>,
    pub fixed: bool,
    pub description: Option<&'static str>,
}

impl ParamSpec {
    pub const fn new(name: &'static str, default: f64, lb: Option<f64>, ub: Option<f64>) -> Self {
        ParamSpec {
            name,
            default,
            lb,
            ub,
            fixed: false,
            description: None,
        }
    }
    pub const fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }
    pub const fn describe(mut self, d: &'static str) -> Self {
        self.description = Some(d);
        self
    }
}

/// Shorthand for a parameter with both bounds.
pub const fn param(name: &'static str, default: f64, lb: f64, ub: f64) -> ParamSpec {
    ParamSpec::new(name, default, Some(lb), Some(ub))
}

#[derive(Clone, Debug)]
pub struct ConstraintSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub weight: f64,
    pub description: Option<&'static str>,
    pub per_curve: bool,
}

/// Which penalty terms are on, and their weights. `enabled: None` means all on.
pub struct PenaltyOptions<'a> {
    pub curves: Option<&'a [Vec<usize>]>,
    pub enabled: Option<&'a HashSet<String>>,
    pub weights: Option<&'a HashMap<String, f64>>,
}

/// A fittable model. Only `predict` is required.
pub trait Model: Send + Sync {
    fn name(&self) -> &str;
    fn inputs(&self) -> &[&'static str];
    fn params(&self) -> &[ParamSpec];
    fn predict(&self, p: &Params, x: &Columns) -> Vec<f64>;

    /// Analytic Jacobian (rows = observations, columns = `wrt`), when the model has one.
    fn jacobian(&self, _p: &Params, _x: &Columns, _wrt: &[String]) -> Option<Vec<Vec<f64>>> {
        None
    }
    fn has_jacobian(&self) -> bool {
        false
    }
    /// Penalty residuals appended to the objective; `None` if the model has no penalties.
    fn penalties(&self, _p: &Params, _x: &Columns, _o: &PenaltyOptions) -> Option<Vec<f64>> {
        None
    }
    fn has_penalties(&self) -> bool {
        false
    }
    fn constraints(&self) -> &[ConstraintSpec] {
        &[]
    }
    fn latex(&self) -> Option<String> {
        None
    }
    /// Quantities derived from a parameter set (e.g. psi_tlp). Key order matters.
    fn derived(&self, _p: &Params) -> Option<IndexMap<String, f64>> {
        None
    }
}

/// `p[name]`, or NaN when absent.
#[inline]
pub fn get(p: &Params, name: &str) -> f64 {
    p.get(name).copied().unwrap_or(f64::NAN)
}

/// `p[name] ?? default`.
#[inline]
pub fn get_or(p: &Params, name: &str, default: f64) -> f64 {
    p.get(name).copied().unwrap_or(default)
}

/// A data column, or an empty slice when absent.
#[inline]
pub fn col<'a>(x: &'a Columns, name: &str) -> &'a [f64] {
    x.get(name).map(|v| v.as_slice()).unwrap_or(&[])
}

/// Default values of a model's parameters, in declaration order.
pub fn defaults(m: &dyn Model) -> Params {
    m.params()
        .iter()
        .map(|s| (s.name.to_string(), s.default))
        .collect()
}
