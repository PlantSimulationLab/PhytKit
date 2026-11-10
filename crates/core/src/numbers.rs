//! Small numeric helpers shared by the models, parsers and output.

use serde_json::Value;

/// The larger of two values, or NaN if either is NaN.
///
/// `f64::max` ignores NaN. Propagating it instead means an invalid parameter region
/// produces a NaN residual, which the fitter rejects, rather than a finite wrong value.
#[inline(always)]
pub fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}

/// The smaller of two values, or NaN if either is NaN. See [`max`].
#[inline(always)]
pub fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.min(b)
    }
}

/// The largest value; `-inf` for an empty slice, NaN if any value is NaN.
pub fn max_of(v: &[f64]) -> f64 {
    v.iter().fold(f64::NEG_INFINITY, |s, &x| max(s, x))
}

/// The smallest value; `inf` for an empty slice, NaN if any value is NaN.
pub fn min_of(v: &[f64]) -> f64 {
    v.iter().fold(f64::INFINITY, |s, &x| min(s, x))
}

/// A number from text, ignoring surrounding whitespace; NaN if it isn't one.
pub fn parse(s: &str) -> f64 {
    s.trim().parse().unwrap_or(f64::NAN)
}

/// A number from text, or NaN unless it's finite.
pub fn parse_finite(s: &str) -> f64 {
    let x = parse(s);
    if x.is_finite() {
        x
    } else {
        f64::NAN
    }
}

/// `v` rounded to `digits` decimal places. Non-finite values are returned unchanged.
pub fn round_to(v: f64, digits: usize) -> f64 {
    if v.is_finite() {
        format!("{v:.digits$}").parse().unwrap_or(v)
    } else {
        v
    }
}

/// `v` rounded for reporting a fitted value: six decimal places, or six significant figures
/// when that keeps more digits (so 1.2378e-5 isn't reported as 0.000012).
pub fn round_fit(v: f64) -> f64 {
    if !v.is_finite() || v == 0.0 {
        return v;
    }
    let mag = v.abs().log10().floor() as i64;
    let digits = (5 - mag).clamp(6, 300) as usize;
    round_to(v, digits)
}

/// `v` as text with exactly `digits` decimal places.
pub fn fixed(v: f64, digits: usize) -> String {
    format!("{v:.digits$}")
}

/// `v` as text: the shortest form that reads back as the same value.
pub fn text(v: f64) -> String {
    format!("{v}")
}

/// `v` as a JSON value: a number, or `null` when it isn't finite.
pub fn json(v: f64) -> Value {
    serde_json::Number::from_f64(v).map_or(Value::Null, Value::Number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nan_propagates() {
        assert!(max(1.0, f64::NAN).is_nan());
        assert!(min_of(&[1.0, f64::NAN]).is_nan());
        assert_eq!(max_of(&[]), f64::NEG_INFINITY);
    }

    #[test]
    fn parsing_and_rounding() {
        assert_eq!(parse(" 12.5 "), 12.5);
        assert!(parse("").is_nan());
        assert!(parse("12abc").is_nan());
        assert!(parse_finite("inf").is_nan());
        assert_eq!(round_to(1.23456789, 6), 1.234568);
        assert_eq!(round_fit(123.456789123), 123.456789);
        assert_eq!(round_fit(0.123456789), 0.123457);
        assert_eq!(round_fit(1.23456789e-5), 1.23457e-5);
        assert_eq!(round_fit(-2.0399321), -2.039932);
        assert_eq!(fixed(2.0, 3), "2.000");
        assert_eq!(text(100.0), "100");
        assert_eq!(json(f64::NAN), Value::Null);
    }
}
