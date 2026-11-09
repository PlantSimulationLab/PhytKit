//! Peaked Arrhenius temperature response.


pub const R_GAS: f64 = 0.008314;

pub fn tresp(t: f64, dha: f64, dhd: f64, topt: f64) -> f64 {
    let c = (dhd / dha - 1.0).ln();
    let num = 1.0 + ((dhd / R_GAS) * (1.0 / topt - 1.0 / 298.0) - c).exp();
    let den = 1.0 + ((dhd / R_GAS) * (1.0 / topt - 1.0 / t) - c).exp();
    ((dha / R_GAS) * (1.0 / 298.0 - 1.0 / t)).exp() * (num / den)
}
