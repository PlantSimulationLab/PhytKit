//! A smooth minimum, so fits have continuous derivatives where the limiting rate changes.
use crate::numbers;

const SHARPNESS: f64 = 0.999;

pub fn smooth_min(a: f64, b: f64) -> f64 {
    if !a.is_finite() {
        return b;
    }
    if !b.is_finite() {
        return a;
    }
    let s = a + b;
    let disc = numbers::max(s * s - 4.0 * SHARPNESS * a * b, 0.0);
    (s - disc.sqrt()) / (2.0 * SHARPNESS)
}

pub fn smooth_min3(a: f64, b: f64, c: f64) -> f64 {
    smooth_min(smooth_min(a, b), c)
}
