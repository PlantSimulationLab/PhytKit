//! The fitting tools the CLI exposes, with their models,
//! data-seeded starting values, and unit options.

use phytkit_core::model::{Columns, Params};
use phytkit_core::models::model_by_name;
use phytkit_core::numbers;

/// Data ready to fit: predictor columns, the response, and what the parser learned.
#[derive(Clone, Debug, Default)]
pub struct FitData {
    pub x: Columns,
    pub y: Vec<f64>,
    /// Column used for each field, in the parser's order.
    pub mapping: Vec<(String, String)>,
    pub warnings: Vec<String>,
    /// Row groups per CurveID (photosynthesis only).
    pub curves: Option<Vec<Vec<usize>>>,
    /// Every parsed column (photosynthesis only), used when rescaling.
    pub columns: Option<Columns>,
    /// Maximum conductance, when vulnerability PLC was derived from a K column.
    pub kmax: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Parser {
    Photosynthesis,
    Stomatal,
    Medlyn,
    Vulnerability,
    PvCurve,
    Soil,
}

pub type Seed = fn(&FitData) -> Params;
pub type Starts = fn(&FitData) -> Vec<Params>;

pub struct ToolModel {
    pub model: &'static str,
    pub label: &'static str,
    pub parser: Parser,
    pub seed: Option<Seed>,
    /// Extra starting points to try after the seeded one (partial parameter sets). The fit
    /// with the lowest rmse is kept. Used where one start can land in a poor local minimum.
    pub starts: Option<Starts>,
}

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub models: &'static [ToolModel],
    pub unit_column: Option<&'static str>,
    pub unit_options: Option<&'static [(&'static str, f64)]>,
    pub photosynthesis: bool,
}

/// Upper median (`s[n / 2]` of the sorted values); NaN for an empty list.
fn median(a: &[f64]) -> f64 {
    let mut s = a.to_vec();
    s.sort_by(|x, y| {
        (x - y)
            .partial_cmp(&0.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    s.get(s.len() / 2).copied().unwrap_or(f64::NAN)
}

/// `x`, or `fallback` when `x` is zero or NaN.
fn or(x: f64, fallback: f64) -> f64 {
    if x == 0.0 || x.is_nan() {
        fallback
    } else {
        x
    }
}

fn default_of(model: &str, param: &str) -> f64 {
    model_by_name(model)
        .and_then(|m| m.params().iter().find(|p| p.name == param))
        .map_or(f64::NAN, |p| p.default)
}

fn abs_psi(d: &FitData) -> Vec<f64> {
    d.x.get("psi")
        .map(|v| v.iter().map(|x| x.abs()).collect())
        .unwrap_or_default()
}

fn theta_seed(d: &FitData) -> Params {
    let mut p = Params::new();
    p.insert("θs".into(), numbers::max_of(&d.y));
    p.insert("θr".into(), numbers::min_of(&d.y));
    p
}

fn seed_bta(d: &FitData) -> Params {
    let mut p = Params::new();
    p.insert(
        "Em".into(),
        numbers::max(numbers::max_of(&d.y), default_of("bta", "Em")),
    );
    p
}

fn seed_vulnerability(d: &FitData) -> Params {
    let mut p = Params::new();
    if let Some(k) = d.kmax {
        p.insert("Kmax".into(), k);
    }
    p
}

fn seed_raats(d: &FitData) -> Params {
    let mut p = theta_seed(d);
    p.insert(
        "b*".into(),
        or(median(&abs_psi(d)), default_of("raats", "b*")),
    );
    p
}

fn seed_brooks_corey(d: &FitData) -> Params {
    let mut p = theta_seed(d);
    let pos: Vec<f64> = abs_psi(d).into_iter().filter(|h| *h > 0.0).collect();
    let min_abs = numbers::min_of(&pos);
    if min_abs.is_finite() {
        p.insert("ψb".into(), min_abs);
    }
    p
}

fn seed_fractal(d: &FitData) -> Params {
    let mut p = theta_seed(d);
    let med = or(median(&abs_psi(d)), 0.1);
    p.insert(
        "r_max".into(),
        numbers::min(1e4, numbers::max(1e-3, 0.2 / med)),
    );
    p
}

use phytkit_core::hydraulics::SOIL_UNITS;

/// |ψ| at the 10th, 30th, 50th, 70th and 90th percentiles (positive values only): the
/// scales a retention curve's air-entry parameter is tried at.
fn psi_scales(d: &FitData) -> Vec<f64> {
    let mut a: Vec<f64> = abs_psi(d)
        .into_iter()
        .filter(|h| *h > 0.0 && h.is_finite())
        .collect();
    a.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    if a.is_empty() {
        return vec![];
    }
    [0.1, 0.3, 0.5, 0.7, 0.9]
        .iter()
        .map(|q| a[((a.len() - 1) as f64 * q).round() as usize])
        .collect()
}

/// Every combination of a scale value (from `psi_scales`, mapped by `scale`) and the given
/// values of other parameters.
fn grid(d: &FitData, scale: (&str, fn(f64) -> f64), others: &[(&str, &[f64])]) -> Vec<Params> {
    let mut out: Vec<Params> = psi_scales(d)
        .into_iter()
        .map(|h| {
            let mut p = theta_seed(d);
            p.insert(scale.0.into(), (scale.1)(h));
            p
        })
        .collect();
    for (name, values) in others {
        out = out
            .into_iter()
            .flat_map(|p| {
                values.iter().map(move |v| {
                    let mut q = p.clone();
                    q.insert((*name).into(), *v);
                    q
                })
            })
            .collect();
    }
    out
}

fn starts_van_genuchten(d: &FitData) -> Vec<Params> {
    grid(d, ("α", |h| 1.0 / h), &[("n", &[1.3, 2.0])])
}

fn starts_modified_van_genuchten(d: &FitData) -> Vec<Params> {
    grid(d, ("α1", |h| 1.0 / h), &[("n1", &[1.3, 2.0])])
}

fn starts_brooks_corey(d: &FitData) -> Vec<Params> {
    grid(d, ("ψb", |h| h), &[("λ", &[0.3, 1.0])])
}

fn starts_raats(d: &FitData) -> Vec<Params> {
    grid(d, ("b*", |h| h), &[("α", &[1.0, 5.0]), ("β", &[2.0, 5.0])])
}

fn starts_fractal(d: &FitData) -> Vec<Params> {
    grid(d, ("r_max", |h| 0.2 / h), &[])
}

pub static TOOLS: &[Tool] = &[
    Tool {
        name: "photosynthesis",
        description: "A–Ci / light / temperature response fitting (LI-6800 logs or CSV with A, Ci, Q|Qabs, Tleaf|T)",
        photosynthesis: true,
        unit_column: None,
        unit_options: None,
        models: &[
            ToolModel { model: "fvcb", label: "Farquhar–von Caemmerer–Berry (C3)", parser: Parser::Photosynthesis, seed: None, starts: None },
            ToolModel { model: "fvcbC4", label: "von Caemmerer (C4)", parser: Parser::Photosynthesis, seed: None, starts: None },
            ToolModel { model: "empiricalA", label: "Empirical", parser: Parser::Photosynthesis, seed: None, starts: None },
        ],
    },
    Tool {
        name: "stomatal",
        description: "Stomatal conductance: BTA (gs, Q, VPD) or Medlyn (gs, A, Cs, VPD)",
        photosynthesis: false,
        unit_column: None,
        unit_options: None,
        models: &[
            ToolModel { model: "bta", label: "Buckley–Turnbull–Adams 2012", parser: Parser::Stomatal, seed: Some(seed_bta), starts: None },
            ToolModel { model: "medlyn", label: "Medlyn et al. 2011 (optimality)", parser: Parser::Medlyn, seed: None, starts: None },
        ],
    },
    Tool {
        name: "hydraulics",
        description: "Xylem vulnerability (Ψ, K or PLC) and pressure–volume (RWC, Ψ) curves",
        photosynthesis: false,
        unit_column: None,
        unit_options: None,
        models: &[
            ToolModel { model: "vulnerability", label: "Sigmoidal vulnerability curve", parser: Parser::Vulnerability, seed: Some(seed_vulnerability), starts: None },
            ToolModel { model: "pvcurve", label: "Pressure–volume curve", parser: Parser::PvCurve, seed: None, starts: None },
        ],
    },
    Tool {
        name: "soil",
        description: "Soil water retention (ψ, θ)",
        photosynthesis: false,
        unit_column: Some("psi"),
        unit_options: Some(SOIL_UNITS),
        models: &[
            ToolModel { model: "vanGenuchten", label: "van Genuchten", parser: Parser::Soil, seed: Some(theta_seed), starts: Some(starts_van_genuchten) },
            ToolModel { model: "modifiedVanGenuchten", label: "modified van Genuchten", parser: Parser::Soil, seed: Some(theta_seed), starts: Some(starts_modified_van_genuchten) },
            ToolModel { model: "raats", label: "Raats superclass", parser: Parser::Soil, seed: Some(seed_raats), starts: Some(starts_raats) },
            ToolModel { model: "brooksCorey", label: "Brooks–Corey", parser: Parser::Soil, seed: Some(seed_brooks_corey), starts: Some(starts_brooks_corey) },
            ToolModel { model: "fractalSWRC", label: "fractal (Li et al. 2025)", parser: Parser::Soil, seed: Some(seed_fractal), starts: Some(starts_fractal) },
        ],
    },
];

pub fn tool_by_name(name: &str) -> Option<&'static Tool> {
    TOOLS.iter().find(|t| t.name == name)
}

/// The LI-600 estimate models (name, label).
pub fn poro_models() -> &'static [(&'static str, &'static str)] {
    &[
        ("fvcb", "FvCB C3"),
        ("fvcbC4", "FvCB C4"),
        ("empiricalA", "Empirical A"),
    ]
}

fn from_dataset(d: phytkit_core::parse::ParsedDataset) -> FitData {
    FitData {
        x: d.x,
        y: d.y,
        mapping: d.mapping.into_iter().collect(),
        warnings: d.warnings,
        curves: d.curves,
        columns: Some(d.columns),
        kmax: None,
    }
}

/// Parse CSV text with a tool model's parser.
pub fn parse_with(parser: Parser, text: &str) -> Result<FitData, String> {
    match parser {
        Parser::Photosynthesis => phytkit_core::parse::parse_csv(text).map(from_dataset),
        other => crate::hydraulics_adapter::parse(other, text),
    }
}

/// An LI-6800 `.xlsx` export.
pub fn parse_licor_xlsx_data(buf: &[u8]) -> Result<FitData, String> {
    phytkit_core::licor::parse_licor_xlsx(buf).map(from_dataset)
}

/// Multiply every observed A by `factor` (rescale mode), in `y` and in the A column.
pub fn scale_response(d: &mut FitData, factor: f64) {
    for v in d.y.iter_mut() {
        *v *= factor;
    }
    let a_col = d
        .mapping
        .iter()
        .find(|(k, _)| k == "A")
        .map(|(_, v)| v.clone());
    if let (Some(cols), Some(a)) = (d.columns.as_mut(), a_col) {
        let scaled: Vec<f64> = cols
            .get(&a)
            .map(|v| v.iter().map(|x| x * factor).collect())
            .unwrap_or_default();
        cols.insert(a, scaled);
    }
}
