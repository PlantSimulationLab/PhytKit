//! Von Caemmerer (2021) C4 photosynthesis.

use super::smooth_min::smooth_min;
use crate::model::{
    col, get, param, Columns, ConstraintSpec, Model, ParamSpec, Params, PenaltyOptions,
};
use crate::numbers;

const R_C4: f64 = 0.0083144598;
const TREF: f64 = 298.15;

const AO: f64 = 0.047;
const F_SPECTRAL: f64 = 0.15;
const RM_FRAC: f64 = 0.5;
const H_PROTONS: f64 = 4.0;
const H_J: f64 = 3.0;
const H_JCYC: f64 = 3.4;

#[inline]
fn at(v: &[f64], i: usize) -> f64 {
    v.get(i).copied().unwrap_or(f64::NAN)
}

fn arrh(t: f64, k25: f64, dha: f64) -> f64 {
    k25 * (dha / (R_C4 * TREF) - dha / (R_C4 * t)).exp()
}
fn peaked(t: f64, k25: f64, dha: f64, dhd: f64, topt: f64) -> f64 {
    let k1 = arrh(t, k25, dha);
    if topt > 1000.0 {
        return k1;
    }
    let ratio = numbers::max(dhd / numbers::max(dha, 1e-6), 1.0001);
    let log_r = (ratio - 1.0).ln();
    let dhd_r = dhd / R_C4;
    let num = 1.0 + (dhd_r * (1.0 / topt - 1.0 / TREF) - log_r).exp();
    let den = 1.0 + (dhd_r * (1.0 / topt - 1.0 / t) - log_r).exp();
    (k1 * num) / den
}

pub struct Rates {
    pub ac: Vec<f64>,
    pub aj: Vec<f64>,
    pub a: Vec<f64>,
}

/// Enzyme-limited, electron-transport-limited and net A at every point.
pub fn rates(p_in: &Params, x: &Columns) -> Rates {
    let mut p = p_in.clone();
    if get(&p, "Jmax_Topt") < 200.0 {
        let v = get(&p, "Jmax_Topt") + 273.15;
        p.insert("Jmax_Topt".to_string(), v);
    }
    let (ci, q, t) = (col(x, "Ci"), col(x, "Q"), col(x, "T"));
    let aobs = x.get("Aobs");
    let n = ci.len();
    let mut out = Rates {
        ac: vec![0.0; n],
        aj: vec![0.0; n],
        a: vec![0.0; n],
    };

    let gm = numbers::max(get(&p, "gm"), 1e-4);
    let theta = numbers::min(numbers::max(get(&p, "theta_etr"), 0.01), 0.99);
    let fcyc = get(&p, "fcyc");
    let alpha = get(&p, "alpha_psII");
    let gbs = get(&p, "gbs");
    let vpr = get(&p, "Vpr");
    let xetr = get(&p, "x_etr");

    // Read each parameter once, not per point.
    let p_vpmax25 = get(&p, "Vpmax25");
    let p_vpmax_dha = get(&p, "Vpmax_dHa");
    let p_vcmax25 = get(&p, "Vcmax25");
    let p_vcmax_dha = get(&p, "Vcmax_dHa");
    let p_jmax25 = get(&p, "Jmax25");
    let p_jmax_dha = get(&p, "Jmax_dHa");
    let p_jmax_dhd = get(&p, "Jmax_dHd");
    let p_jmax_topt = get(&p, "Jmax_Topt");
    let p_rd25 = get(&p, "Rd25");
    let p_rd_dha = get(&p, "Rd_dHa");
    let p_kc25 = get(&p, "Kc25");
    let p_kc_dha = get(&p, "Kc_dHa");
    let p_ko25 = get(&p, "Ko25");
    let p_ko_dha = get(&p, "Ko_dHa");
    let p_kp25 = get(&p, "Kp25");
    let p_kp_dha = get(&p, "Kp_dHa");
    let p_gammastar25 = get(&p, "GammaStar25");
    let p_gammastar_dha = get(&p, "GammaStar_dHa");
    let p_om25 = get(&p, "Om25");
    for i in 0..n {
        let ti = at(t, i);
        let qi = at(q, i);
        let cii = ci[i];

        let vpmax = arrh(ti, p_vpmax25, p_vpmax_dha);
        let vcmax = arrh(ti, p_vcmax25, p_vcmax_dha);
        let jmax = peaked(ti, p_jmax25, p_jmax_dha, p_jmax_dhd, p_jmax_topt);
        let rd = arrh(ti, p_rd25, p_rd_dha);
        let kc = arrh(ti, p_kc25, p_kc_dha);
        let ko = arrh(ti, p_ko25, p_ko_dha);
        let kp = arrh(ti, p_kp25, p_kp_dha);
        let gamma_star = arrh(ti, p_gammastar25, p_gammastar_dha);
        let om = p_om25;

        let rho = (1.0 - fcyc) / (2.0 - fcyc);
        let i2 = qi * rho * (1.0 - F_SPECTRAL);
        let sum_ij = i2 + jmax;
        let j = (sum_ij - numbers::max(sum_ij * sum_ij - 4.0 * theta * i2 * jmax, 0.0).sqrt())
            / (2.0 * theta);

        let z = (H_J * (1.0 - fcyc) + H_JCYC * fcyc) / numbers::max(H_PROTONS * (1.0 - fcyc), 1e-6);
        let rm = RM_FRAC * rd;
        let jmes = xetr * j;
        let jbun = (1.0 - xetr) * j;
        let kco = 1.0 + om / ko;

        let an_of = |cm_in: f64| -> (f64, f64, f64) {
            let cm = numbers::max(cm_in, 0.1);
            let vp = smooth_min((cm * vpmax) / (cm + kp), vpr);

            let term_pep = vp - rm + gbs * cm;
            let term_rub = vcmax - rd;
            let ac_a = 1.0 - (alpha * kc) / (AO * ko);
            let ac_b = -(term_pep
                + term_rub
                + gbs * kc * kco
                + (alpha / AO) * (gamma_star * vcmax + (rd * kc) / ko));
            let ac_c = term_rub * term_pep - gbs * (vcmax * gamma_star * om + rd * kc * kco);
            let ac_disc = numbers::max(ac_b * ac_b - 4.0 * ac_a * ac_c, 0.0);
            let ac = (-ac_b - ac_disc.sqrt()) / (2.0 * ac_a);

            let term_mes_j = jmes * z * 0.5 - rm + gbs * cm;
            let term_bun_j = (jbun * z) / 3.0 - rd;
            let aj_a = 1.0 - (7.0 * gamma_star * alpha) / (3.0 * AO);
            let aj_b = -(term_mes_j
                + term_bun_j
                + (gbs * 7.0 * gamma_star * om) / 3.0
                + ((alpha * gamma_star) / AO) * ((jbun * z) / 3.0 + (7.0 * rd) / 3.0));
            let aj_c = term_mes_j * term_bun_j
                - gbs * gamma_star * om * ((jbun * z) / 3.0 + (7.0 * rd) / 3.0);
            let aj_disc = numbers::max(aj_b * aj_b - 4.0 * aj_a * aj_c, 0.0);
            let aj = (-aj_b - aj_disc.sqrt()) / (2.0 * aj_a);

            (ac, aj, smooth_min(ac, aj))
        };

        let cm = if let Some(aobs) = aobs {
            cii - at(aobs, i) / gm
        } else {
            let mut lo = 1e-3;
            let mut hi = numbers::max(cii, 1.0) + 2000.0;
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                if mid - cii + an_of(mid).2 / gm > 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            0.5 * (lo + hi)
        };
        let (ac, aj, a) = an_of(cm);
        out.ac[i] = ac;
        out.aj[i] = aj;
        out.a[i] = a;
    }
    out
}

/// Net assimilation at every point (`fvcbC4.predict`).
pub fn predict(p: &Params, x: &Columns) -> Vec<f64> {
    rates(p, x).a
}

const C_VPVPR: &str = "vpmax_gt_vpr";
const C_VPBREAK: &str = "vp_breakpoint";
const C_REGIME: &str = "acaj_regime";
const C_RDTETHER: &str = "rd_tether";
const C_RATEWIN: &str = "rate_windows";

fn default_weight(k: &str) -> f64 {
    match k {
        C_VPVPR => 1.0,
        C_VPBREAK => 0.02,
        C_REGIME => 0.1,
        C_RDTETHER => 10.0,
        C_RATEWIN => 0.1,
        _ => f64::NAN,
    }
}

pub struct FvcbC4 {
    params: Vec<ParamSpec>,
    constraints: Vec<ConstraintSpec>,
}

impl FvcbC4 {
    pub fn new() -> Self {
        let c = |key, label, description, per_curve| ConstraintSpec {
            key,
            label,
            weight: default_weight(key),
            description: Some(description),
            per_curve,
        };
        FvcbC4 {
            params: vec![
                param("Vpmax25", 200.0, 1.0, 600.0).describe("max PEP carboxylation at 25C"),
                param("Vcmax25", 40.0, 1.0, 200.0).describe("max Rubisco carboxylation at 25C"),
                param("Jmax25", 247.69, 5.0, 700.0)
                    .describe("max linear electron transport at 25C"),
                param("Rd25", 1.0, 0.0, 15.0).describe("day respiration at 25C"),
                param("Vpmax_dHa", 50.1, 1.0, 200.0),
                param("Vcmax_dHa", 78.0, 1.0, 200.0),
                param("Jmax_dHa", 77.9, 1.0, 200.0),
                param("Jmax_dHd", 260.0, 100.0, 500.0),
                param("Jmax_Topt", 316.15, 288.0, 333.0),
                param("Rd_dHa", 66.4, 1.0, 200.0),
                param("gm", 1.0, 0.05, 5.0)
                    .fixed()
                    .describe("mesophyll conductance (Cm = Ci − A/gm)"),
                param("gbs", 0.003, 1e-4, 0.05)
                    .fixed()
                    .describe("bundle-sheath conductance"),
                param("Vpr", 80.0, 20.0, 200.0)
                    .fixed()
                    .describe("PEP regeneration cap"),
                param("alpha_psII", 0.0, 0.0, 1.0)
                    .fixed()
                    .describe("fraction of PSII activity in bundle sheath"),
                param("x_etr", 0.4, 0.05, 0.95)
                    .fixed()
                    .describe("fraction of linear ETR to mesophyll"),
                param("fcyc", 0.45, 0.0, 0.85)
                    .fixed()
                    .describe("cyclic electron flow fraction"),
                param("theta_etr", 0.7, 0.1, 0.99)
                    .fixed()
                    .describe("curvature of J(Q)"),
                param("Kc25", 1210.0, 500.0, 3000.0)
                    .fixed()
                    .describe("Rubisco Kc at 25C (μbar)"),
                param("Kc_dHa", 64.2, 1.0, 200.0).fixed(),
                param("Ko25", 292000.0, 1e5, 5e5)
                    .fixed()
                    .describe("Rubisco Ko at 25C (μbar)"),
                param("Ko_dHa", 10.5, 1.0, 200.0).fixed(),
                param("Kp25", 82.0, 20.0, 300.0)
                    .fixed()
                    .describe("PEPC Kp at 25C (μbar)"),
                param("Kp_dHa", 38.3, 1.0, 200.0).fixed(),
                param("GammaStar25", 3.81679e-4, 1e-4, 1e-3)
                    .fixed()
                    .describe("half Rubisco O2/CO2 specificity (dimensionless)"),
                param("GammaStar_dHa", 31.1, 1.0, 200.0).fixed(),
                param("Om25", 210000.0, 1e5, 3e5)
                    .fixed()
                    .describe("bundle-sheath O2 partial pressure (μbar)"),
            ],
            constraints: vec![
                c(
                    C_VPVPR,
                    "Vpmax > Vpr",
                    "PEP kinetic curve must reach its regeneration cap",
                    false,
                ),
                c(
                    C_VPBREAK,
                    "Vpr breakpoint in Ci range",
                    "Keep the Vpmax→Vpr transition inside the data (Vpmax/Vpr identifiability)",
                    true,
                ),
                c(
                    C_REGIME,
                    "Ac/Aj regime at high Ci",
                    "High light → Ac binds; low light → Aj binds, at saturating Ci",
                    true,
                ),
                c(
                    C_RDTETHER,
                    "Rd ≈ 0.02·Vcmax",
                    "Soft tether of day respiration to Vcmax (poorly identified in C4)",
                    false,
                ),
                c(
                    C_RATEWIN,
                    "Rate windows",
                    "Soft priors: Vpmax∈[20,400], Vcmax∈[5,200], Jmax∈[50,600]",
                    false,
                ),
            ],
        }
    }
}

impl Default for FvcbC4 {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for FvcbC4 {
    fn name(&self) -> &str {
        "fvcbC4"
    }
    fn inputs(&self) -> &[&'static str] {
        &["Ci", "Q", "T"]
    }
    fn params(&self) -> &[ParamSpec] {
        &self.params
    }
    fn predict(&self, p: &Params, x: &Columns) -> Vec<f64> {
        predict(p, x)
    }
    fn has_penalties(&self) -> bool {
        true
    }
    fn penalties(&self, p: &Params, x: &Columns, o: &PenaltyOptions) -> Option<Vec<f64>> {
        let on = |k: &str| o.enabled.is_none_or(|e| e.contains(k));
        let wt = |k: &str| {
            o.weights
                .and_then(|w| w.get(k).copied())
                .unwrap_or_else(|| default_weight(k))
                .sqrt()
        };
        let mut res = Vec::new();
        let ci = col(x, "Ci");
        let q = col(x, "Q");

        let vpmax = get(p, "Vpmax25");
        let vcmax = get(p, "Vcmax25");
        let jmax = get(p, "Jmax25");
        let rd = get(p, "Rd25");
        let vpr = get(p, "Vpr");
        let kp = get(p, "Kp25");

        if on(C_VPVPR) {
            res.push(wt(C_VPVPR) * numbers::max(vpr - vpmax + 1.0, 0.0));
        }
        if on(C_RDTETHER) {
            res.push(wt(C_RDTETHER) * (rd - 0.02 * vcmax));
        }
        if on(C_RATEWIN) {
            res.push(wt(C_RATEWIN) * numbers::max(20.0 - vpmax, 0.0));
            res.push(wt(C_RATEWIN) * numbers::max(vpmax - 400.0, 0.0));
            res.push(wt(C_RATEWIN) * numbers::max(5.0 - vcmax, 0.0));
            res.push(wt(C_RATEWIN) * numbers::max(vcmax - 200.0, 0.0));
            res.push(wt(C_RATEWIN) * numbers::max(50.0 - jmax, 0.0));
            res.push(wt(C_RATEWIN) * numbers::max(jmax - 600.0, 0.0));
        }

        if on(C_VPBREAK) || on(C_REGIME) {
            let r = rates(p, x);
            let all: Vec<Vec<usize>>;
            let groups: &[Vec<usize>] = match o.curves {
                Some(c) => c,
                None => {
                    all = vec![(0..ci.len()).collect()];
                    &all
                }
            };
            for idx in groups {
                let mut ci_lo = f64::INFINITY;
                let mut ci_hi = f64::NEG_INFINITY;
                for &i in idx {
                    let c = at(ci, i);
                    if c < ci_lo {
                        ci_lo = c;
                    }
                    if c > ci_hi {
                        ci_hi = c;
                    }
                }
                if on(C_VPBREAK) {
                    if vpmax > vpr {
                        let breakpt = (vpr * kp) / (vpmax - vpr);
                        res.push(wt(C_VPBREAK) * numbers::max(ci_lo - breakpt, 0.0));
                        res.push(wt(C_VPBREAK) * numbers::max(breakpt - ci_hi, 0.0));
                    } else {
                        res.push(0.0);
                        res.push(0.0);
                    }
                }
                if on(C_REGIME) {
                    let thresh = ci_lo + 0.75 * (ci_hi - ci_lo);
                    for &i in idx {
                        if at(ci, i) < thresh {
                            continue;
                        }
                        let (aci, aji) = (at(&r.ac, i), at(&r.aj, i));
                        if at(q, i) >= 1200.0 {
                            res.push(wt(C_REGIME) * numbers::max(aci - aji, 0.0));
                        } else if at(q, i) <= 500.0 {
                            res.push(5f64.sqrt() * wt(C_REGIME) * numbers::max(aji - aci, 0.0));
                        }
                    }
                }
            }
        }
        Some(res)
    }
    fn constraints(&self) -> &[ConstraintSpec] {
        &self.constraints
    }
    fn latex(&self) -> Option<String> {
        Some(r"A = \min\{A_c,\ A_j\},\quad V_p = \min\!\left(\tfrac{C_m V_{pmax}}{C_m+K_p},\ V_{pr}\right),\quad C_m = C_i - A/g_m".to_string())
    }
}
