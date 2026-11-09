//! Put response curves onto the amplitude of survey points.
//! Functions take plain columns so they don't depend on a particular dataset type.

use crate::model::{Columns, Model, Params};
use crate::numbers;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stat {
    Mean,
    Median,
    Max,
}

impl Stat {
    pub fn parse(s: &str) -> Option<Stat> {
        match s {
            "mean" => Some(Stat::Mean),
            "median" => Some(Stat::Median),
            "max" => Some(Stat::Max),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Stat::Mean => "mean",
            Stat::Median => "median",
            Stat::Max => "max",
        }
    }
}

/// mean, median or max of the finite values (NaN if there are none).
pub fn stat_of(v: &[f64], stat: Stat) -> f64 {
    let a: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if a.is_empty() {
        return f64::NAN;
    }
    match stat {
        Stat::Max => numbers::max_of(&a),
        Stat::Mean => a.iter().fold(0.0, |s, x| s + x) / a.len() as f64,
        Stat::Median => {
            let mut s = a.clone();
            s.sort_by(|x, y| x.partial_cmp(y).unwrap());
            let mid = s.len() >> 1;
            if s.len() % 2 == 1 {
                s[mid]
            } else {
                (s[mid - 1] + s[mid]) / 2.0
            }
        }
    }
}

/// The survey's light and temperature: the mean of all finite pooled values.
pub fn survey_conditions(q: &[f64], t: &[f64]) -> Option<(f64, f64)> {
    let q: Vec<f64> = q.iter().copied().filter(|x| x.is_finite()).collect();
    let t: Vec<f64> = t.iter().copied().filter(|x| x.is_finite()).collect();
    if q.is_empty() || t.is_empty() {
        return None;
    }
    let mean = |a: &[f64]| a.iter().fold(0.0, |s, x| s + x) / a.len() as f64;
    Some((mean(&q), mean(&t)))
}

/// The highest finite Ci across the curves.
pub fn saturating_ci(ci: &[f64]) -> Option<f64> {
    let a: Vec<f64> = ci.iter().copied().filter(|x| x.is_finite()).collect();
    if a.is_empty() {
        None
    } else {
        Some(numbers::max_of(&a))
    }
}

/// A the fitted model predicts at the reference condition (T in Kelvin).
pub fn predict_at_ref(model: &dyn Model, params: &Params, ci: f64, q: f64, t: f64) -> f64 {
    let mut x = Columns::new();
    x.insert("Ci".into(), vec![ci]);
    x.insert("Q".into(), vec![q]);
    x.insert("T".into(), vec![t]);
    model
        .predict(params, &x)
        .first()
        .copied()
        .unwrap_or(f64::NAN)
}

/// stat(survey A) / stat(reference A), or NaN when either side is unusable.
pub fn rescale_factor(survey: &[f64], refs: &[f64], stat: Stat) -> f64 {
    let num = stat_of(survey, stat);
    let den = stat_of(refs, stat);
    if !num.is_finite() || !den.is_finite() || den == 0.0 {
        return f64::NAN;
    }
    num / den
}
