//! Farquhar-von Caemmerer-Berry C3.

use super::smooth_min::smooth_min3;
use crate::model::{
    col, get, get_or, param, Columns, ConstraintSpec, Model, ParamSpec, Params, PenaltyOptions,
};
use crate::numbers;

const R_FVCB: f64 = 0.0083144598;
const TREF: f64 = 298.15;

fn peaked(t: f64, k25: f64, dha: f64, dhd: f64, topt: f64) -> f64 {
    let arr = k25 * (dha / (R_FVCB * TREF) - dha / (R_FVCB * t)).exp();
    let c = (numbers::max(dhd / dha - 1.0, 1e-9)).ln();
    let num = 1.0 + ((dhd / R_FVCB) * (1.0 / topt - 1.0 / TREF) - c).exp();
    let den = 1.0 + ((dhd / R_FVCB) * (1.0 / topt - 1.0 / t) - c).exp();
    arr * (num / den)
}
fn arrh(t: f64, k25: f64, dha: f64) -> f64 {
    k25 * (dha / (R_FVCB * TREF) - dha / (R_FVCB * t)).exp()
}

pub struct Rates {
    pub ac: Vec<f64>,
    pub aj: Vec<f64>,
    pub ap: Vec<f64>,
    pub a: Vec<f64>,
}

/// Net Rubisco-, RuBP- and TPU-limited rates and A at every point.
pub fn rates(p_in: &Params, x: &Columns) -> Rates {
    let mut p = p_in.clone();
    for f in ["Vcmax_Topt", "Jmax_Topt", "TPU_Topt"] {
        if get(&p, f) < 200.0 {
            let v = get(&p, f) + 273.15;
            p.insert(f.to_string(), v);
        }
    }
    let (ci, q, t) = (col(x, "Ci"), col(x, "Q"), col(x, "T"));
    let aobs = x.get("Aobs");
    let n = ci.len();
    let mut out = Rates {
        ac: vec![0.0; n],
        aj: vec![0.0; n],
        ap: vec![0.0; n],
        a: vec![0.0; n],
    };
    let gm = numbers::max(get(&p, "gm"), 1e-4);
    let theta = numbers::min(numbers::max(get(&p, "theta"), 0.01), 0.99);
    let alpha_g = get_or(&p, "alphaG", 0.0);
    // Read every parameter once; looking them up by name per point dominated the run time.
    let (vc25, vc_ha, vc_hd, vc_to) = (
        get(&p, "Vcmax25"),
        get(&p, "Vcmax_dHa"),
        get(&p, "Vcmax_dHd"),
        get(&p, "Vcmax_Topt"),
    );
    let (jm25, jm_ha, jm_hd, jm_to) = (
        get(&p, "Jmax25"),
        get(&p, "Jmax_dHa"),
        get(&p, "Jmax_dHd"),
        get(&p, "Jmax_Topt"),
    );
    let (tp25, tp_ha, tp_hd, tp_to) = (
        get(&p, "TPU25"),
        get(&p, "TPU_dHa"),
        get(&p, "TPU_dHd"),
        get(&p, "TPU_Topt"),
    );
    let (kc25, kc_ha, ko25, ko_ha) = (
        get(&p, "Kc25"),
        get(&p, "Kc_dHa"),
        get(&p, "Ko25"),
        get(&p, "Ko_dHa"),
    );
    let (gs25, gs_ha, rd25, rd_ha) = (
        get(&p, "GammaS25"),
        get(&p, "GammaS_dHa"),
        get(&p, "Rd25"),
        get(&p, "Rd_dHa"),
    );
    let (o2, alpha) = (get(&p, "O"), get(&p, "alpha"));
    for i in 0..n {
        let ti = t.get(i).copied().unwrap_or(f64::NAN);
        let qi = q.get(i).copied().unwrap_or(f64::NAN);
        let cii = ci[i];
        let vcmax = peaked(ti, vc25, vc_ha, vc_hd, vc_to);
        let jmax = peaked(ti, jm25, jm_ha, jm_hd, jm_to);
        let tpu = peaked(ti, tp25, tp_ha, tp_hd, tp_to);
        let kc = arrh(ti, kc25, kc_ha);
        let ko = arrh(ti, ko25, ko_ha);
        let gamma = arrh(ti, gs25, gs_ha);
        let rd = arrh(ti, rd25, rd_ha);
        let kco = kc * (1.0 + o2 / ko);
        let aq = alpha * qi;
        let j = (aq + jmax
            - numbers::max((aq + jmax).powf(2.0) - 4.0 * theta * aq * jmax, 0.0).sqrt())
            / (2.0 * theta);

        let triplet = |cc_in: f64| -> (f64, f64, f64, f64) {
            let cc = numbers::max(cc_in, 1e-2);
            let gcc = 1.0 - gamma / cc;
            let wc = (vcmax * cc) / (cc + kco);
            let wj = (j * cc) / (4.0 * cc + 8.0 * gamma);
            let denp = cc - gamma * (1.0 + 3.0 * alpha_g);
            let wp = if denp > 1e-6 {
                (3.0 * tpu * cc) / denp
            } else {
                f64::INFINITY
            };
            let a = gcc * smooth_min3(wc, wj, wp) - rd;
            let ap = if wp.is_finite() {
                gcc * wp - rd
            } else {
                f64::INFINITY
            };
            (gcc * wc - rd, gcc * wj - rd, ap, a)
        };
        let cc = if let Some(aobs) = aobs {
            cii - aobs[i] / gm
        } else {
            let f = |c: f64| c - cii + triplet(c).3 / gm;
            let mut lo = 1e-3;
            let mut hi = numbers::max(cii, 1.0) + 2000.0;
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                if f(mid) > 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            0.5 * (lo + hi)
        };
        let (ac, aj, ap, a) = triplet(cc);
        out.ac[i] = ac;
        out.aj[i] = aj;
        out.ap[i] = ap;
        out.a[i] = a;
    }
    out
}

const VCMAX_LO: f64 = 20.0;
const VCMAX_HI: f64 = 130.0;
const VJ_LO: f64 = 1.6;
const VJ_HI: f64 = 2.0;
const C_AP: &str = "ap_nonneg";
const C_JC: &str = "aj_le_ac_mid";
const C_LSAJ: &str = "rubisco_region";
const C_INTER: &str = "ap_above_cross";
const C_LAST: &str = "ap_limits_high";
const C_VCMAX: &str = "vcmax_window";
const C_ALPHAG: &str = "alphag_zero";
const C_VJ: &str = "vj_ratio";
/// Constraints that are only applied when asked for (`--constraint`). On by default they bias
/// ordinary fits: `ap_limits_high` forces TPU limitation onto every curve, even curves with no TPU
/// plateau, and `vcmax_window` pulls Vcmax25 of high-capacity leaves down toward 130.
const OFF_BY_DEFAULT: [&str; 2] = [C_LAST, C_VCMAX];

fn default_weight(k: &str) -> f64 {
    match k {
        C_AP => 1.0,
        C_JC => 0.3,
        C_LSAJ => 1.0,
        C_INTER => 1.0,
        C_LAST => 1.0,
        C_VCMAX => 0.05,
        C_ALPHAG => 2.0,
        C_VJ => 0.1,
        _ => f64::NAN,
    }
}

pub struct Fvcb {
    params: Vec<ParamSpec>,
    constraints: Vec<ConstraintSpec>,
}

impl Fvcb {
    pub fn new() -> Self {
        let c = |key, label, description, per_curve| ConstraintSpec {
            key,
            label,
            weight: default_weight(key),
            description: Some(description),
            per_curve,
            default_on: !OFF_BY_DEFAULT.contains(&key),
        };
        Fvcb {
            params: vec![
                param("Vcmax25", 100.0, 5.0, 400.0).units("µmol m⁻² s⁻¹").describe("max Rubisco carboxylation at 25C"),
                param("Jmax25", 200.0, 5.0, 700.0).units("µmol m⁻² s⁻¹").describe("max electron transport at 25C"),
                param("TPU25", 25.0, 1.0, 100.0).units("µmol m⁻² s⁻¹").describe("triose-phosphate utilization at 25C"),
                param("Rd25", 1.5, 0.0, 15.0).units("µmol m⁻² s⁻¹").describe("day respiration at 25C"),
                param("gm", 0.25, 0.01, 5.0).units("mol m⁻² s⁻¹ bar⁻¹").describe("mesophyll conductance (Cc = Ci − A/gm)"),
                param("alpha", 0.5, 0.1, 0.9).units("mol mol⁻¹").describe("quantum yield of electron transport"),
                param("theta", 0.7, 0.1, 0.99).units("dimensionless").describe("curvature of J(Q)"),
                param("alphaG", 0.0, 0.0, 1.0).units("dimensionless").describe("glycolate carbon returned (TPU)"),
                param("Vcmax_dHa", 65.33, 1.0, 200.0).units("kJ mol⁻¹").describe("activation energy of Vcmax (peaked Arrhenius)"),
                param("Vcmax_dHd", 200.0, 100.0, 500.0).units("kJ mol⁻¹").describe("deactivation energy of Vcmax"),
                param("Vcmax_Topt", 311.15, 288.0, 333.0).units("K").describe("temperature optimum of Vcmax, in kelvin (°C + 273.15; a value below 200 is read as °C)"),
                param("Jmax_dHa", 43.9, 1.0, 200.0).units("kJ mol⁻¹").describe("activation energy of Jmax (peaked Arrhenius)"),
                param("Jmax_dHd", 200.0, 100.0, 500.0).units("kJ mol⁻¹").describe("deactivation energy of Jmax"),
                param("Jmax_Topt", 311.15, 288.0, 333.0).units("K").describe("temperature optimum of Jmax, in kelvin (°C + 273.15; a value below 200 is read as °C)"),
                param("TPU_dHa", 53.1, 1.0, 200.0).units("kJ mol⁻¹").describe("activation energy of TPU (peaked Arrhenius)"),
                param("TPU_dHd", 201.8, 100.0, 500.0).units("kJ mol⁻¹").describe("deactivation energy of TPU"),
                param("TPU_Topt", 306.4, 288.0, 333.0).units("K").describe("temperature optimum of TPU, in kelvin (°C + 273.15; a value below 200 is read as °C)"),
                param("Rd_dHa", 46.39, 1.0, 200.0).units("kJ mol⁻¹").describe("activation energy of Rd (Arrhenius)"),
                param("GammaS25", 42.75, 10.0, 120.0).units("µmol mol⁻¹").fixed().describe("CO2 compensation point (no Rd) at 25C"),
                param("GammaS_dHa", 37.83, 1.0, 200.0).units("kJ mol⁻¹").describe("activation energy of Γ* (Arrhenius)").fixed(),
                param("Kc25", 404.9, 100.0, 1000.0).units("µmol mol⁻¹").describe("Michaelis constant of Rubisco for CO2 at 25C").fixed(),
                param("Kc_dHa", 79.43, 1.0, 200.0).units("kJ mol⁻¹").describe("activation energy of Kc (Arrhenius)").fixed(),
                param("Ko25", 278.4, 100.0, 600.0).units("mmol mol⁻¹").describe("Michaelis constant of Rubisco for O2 at 25C").fixed(),
                param("Ko_dHa", 36.38, 1.0, 200.0).units("kJ mol⁻¹").describe("activation energy of Ko (Arrhenius)").fixed(),
                param("O", 213.5, 100.0, 300.0).units("mmol mol⁻¹").fixed().describe("chloroplastic O2 (mmol/mol)"),
            ],
            constraints: vec![
                c(C_AP, "Ap ≥ 0", "TPU-limited rate non-negative at every point", false),
                c(C_JC, "Aj ≤ Ac (mid Ci)", "Light-limited ≤ Rubisco-limited at Ci 500–700", true),
                c(C_LSAJ, "Rubisco region exists", "A low-Ci Rubisco-limited stretch must be present", true),
                c(C_INTER, "Ap above Ac/Aj crossing", "TPU not limiting at the Ac–Aj transition", true),
                c(C_LAST, "Ap limits at high Ci", "TPU is the limiter at the highest-Ci point", true),
                c(C_VCMAX, "Vcmax25 ∈ [20, 130]", "Soft prior keeping Vcmax25 in a typical range (PhoTorch)", false),
                c(C_ALPHAG, "αG → 0", "Pull the glycolate-return fraction toward 0 (PhoTorch)", false),
                c(C_VJ, "Jmax:Vcmax ≈ 1.6–2.0", "Soft prior on the Jmax25:Vcmax25 ratio (single-curve analog of PhoTorch's correlation term)", false),
            ],
        }
    }
}

impl Default for Fvcb {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for Fvcb {
    fn name(&self) -> &str {
        "fvcb"
    }
    fn inputs(&self) -> &[&'static str] {
        &["Ci", "Q", "T"]
    }
    fn params(&self) -> &[ParamSpec] {
        &self.params
    }
    fn predict(&self, p: &Params, x: &Columns) -> Vec<f64> {
        rates(p, x).a
    }
    fn has_penalties(&self) -> bool {
        true
    }
    fn penalties(&self, p: &Params, x: &Columns, o: &PenaltyOptions) -> Option<Vec<f64>> {
        let on = |k: &str| match o.enabled {
            Some(e) => e.contains(k),
            None => !OFF_BY_DEFAULT.contains(&k),
        };
        let wt = |k: &str| {
            o.weights
                .and_then(|w| w.get(k).copied())
                .unwrap_or_else(|| default_weight(k))
                .sqrt()
        };
        let r = rates(p, x);
        let ci = col(x, "Ci");
        let mut res = Vec::new();
        if on(C_AP) {
            for i in 0..r.ap.len() {
                res.push(wt(C_AP) * numbers::max(-r.ap[i], 0.0));
            }
        }
        let all: Vec<Vec<usize>>;
        let groups: &[Vec<usize>] = match o.curves {
            Some(c) => c,
            None => {
                all = vec![(0..ci.len()).collect()];
                &all
            }
        };
        if on(C_JC) || on(C_LSAJ) || on(C_INTER) || on(C_LAST) {
            for idx in groups {
                let mut order = idx.clone();
                // Array.prototype.sort is stable; a NaN comparison counts as equal.
                order.sort_by(|&a, &b| {
                    ci[a]
                        .partial_cmp(&ci[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let (mut ls_aj, mut best_k, mut best_d) = (0.0, 0usize, f64::INFINITY);
                for (k, &i) in order.iter().enumerate() {
                    ls_aj += numbers::max(r.aj[i] - r.ac[i], 0.0);
                    let d = (r.ac[i] - r.aj[i]).abs();
                    if d < best_d {
                        best_d = d;
                        best_k = k;
                    }
                    if on(C_JC) && ci[i] > 500.0 && ci[i] < 700.0 {
                        res.push(wt(C_JC) * numbers::max(r.aj[i] - r.ac[i], 0.0));
                    }
                }
                if on(C_LSAJ) {
                    res.push(wt(C_LSAJ) * numbers::max(5.0 - ls_aj, 0.0));
                }
                if on(C_INTER) {
                    if let Some(&ii) = order.get(best_k) {
                        res.push(wt(C_INTER) * numbers::max(1.1 * r.aj[ii] - r.ap[ii], 0.0));
                    } else {
                        res.push(f64::NAN);
                    }
                }
                if let Some(&il) = order.last() {
                    if on(C_LAST) && ci[il] > 500.0 {
                        res.push(wt(C_LAST) * numbers::max(r.ap[il] - r.aj[il], 0.0));
                    }
                }
            }
        }
        let vc = get(p, "Vcmax25");
        let jm = get(p, "Jmax25");
        let ag = get_or(p, "alphaG", 0.0);
        if on(C_VCMAX) {
            res.push(wt(C_VCMAX) * numbers::max(VCMAX_LO - vc, 0.0));
            res.push(wt(C_VCMAX) * numbers::max(vc - VCMAX_HI, 0.0));
        }
        if on(C_ALPHAG) {
            res.push(wt(C_ALPHAG) * numbers::max(ag, 0.0));
        }
        if on(C_VJ) {
            res.push(wt(C_VJ) * numbers::max(VJ_LO * vc - jm, 0.0));
            res.push(wt(C_VJ) * numbers::max(jm - VJ_HI * vc, 0.0));
        }
        Some(res)
    }
    fn constraints(&self) -> &[ConstraintSpec] {
        &self.constraints
    }
    fn latex(&self) -> Option<String> {
        Some(r"A = \left(1-\tfrac{\Gamma^*}{C_c}\right)\min\{\,w_c,\ w_j,\ w_p\,\} - R_d,\quad C_c = C_i - A/g_m".to_string())
    }
}
