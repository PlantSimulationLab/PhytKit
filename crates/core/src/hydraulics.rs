//! Reading plant-hydraulics, stomatal-conductance and soil-retention CSVs.
//!
//! Error messages reach the CLI's JSON output, so they're written for users.

use crate::model::Columns;
use crate::numbers;
use crate::table::{read_rows, write_rows, Table};
use crate::warn::Warning;
use indexmap::IndexMap;
use std::collections::HashSet;

/// `ParsedFitData`.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedFitData {
    /// `X`: predictor columns (e.g. `{ P }` or `{ w }`).
    pub x: Columns,
    /// Response column.
    pub y: Vec<f64>,
    pub n: usize,
    /// Column name used for each field, in a fixed order.
    pub mapping: IndexMap<String, String>,
    pub warnings: Vec<Warning>,
    /// `K`: raw hydraulic conductance (vulnerability only).
    pub k: Option<Vec<f64>>,
    /// `Kmax` (vulnerability only).
    pub kmax: Option<f64>,
}

// MPa-named variants are listed before bar-named ones so a file carrying both
// (e.g. "Psi (MPa)" and "Psi (bar)") prefers the column already in MPa.
pub const P_ALIASES: &[&str] = &[
    "P",
    "Psi (MPa)",
    "Psi(MPa)",
    "WP (MPa)",
    "WP(MPa)",
    "Psi_MPa",
    "ψ_MPa",
    "Psi",
    "psi",
    "ψ",
    "WP",
    "water_potential",
    "waterpotential",
    "potential",
    "MPa",
    "Pressure",
    "P_MPa",
    "Pa",
    "Psi (bar)",
    "Psi(bar)",
    "WP (bar)",
    "Psi_bar",
    "ψ_bar",
];
pub const PLC_ALIASES: &[&str] = &[
    "PLC",
    "plc",
    "loss",
    "percent_loss",
    "percentloss",
    "PLC_pct",
    "PLCpercent",
    "pld",
];
const SOIL_EXTRA_PSI_ALIASES: &[&str] = &[
    "matric_potential",
    "matricpotential",
    "matric",
    "suction",
    "tension",
    "pressure_head",
    "pressurehead",
    "head",
    "h",
    "kPa",
    "hPa",
    "cm",
    "psi_MPa",
];
pub const THETA_ALIASES: &[&str] = &[
    "theta",
    "θ",
    "VWC",
    "vwc",
    "theta_v",
    "SWC",
    "swc",
    "volumetric_water_content",
    "vol_water_content",
    "volwatercontent",
    "water_content",
    "watercontent",
    "moisture",
    "VMC",
    "vmc",
    "WC",
];
pub const RWC_ALIASES: &[&str] = &[
    "RWC",
    "rwc",
    "RWC (%)",
    "RWC(%)",
    "relative_water_content",
    "relativewatercontent",
    "water_content",
    "watercontent",
    "RWC_pct",
    "RWCpercent",
    "w",
];
pub const GS_ALIASES: &[&str] = &[
    "gsw_corrected",
    "gsw",
    "gs",
    "gtw",
    "g_sw",
    "gvw",
    "stomatal_conductance",
    "conductance",
    "cond",
    "gs_w",
];
pub const Q_ALIASES: &[&str] = &[
    "Qin", "Qabs", "Q", "PARi", "PPFD", "PAR", "Qamb", "PFD", "light", "Qi", "PARin",
];
pub const VPD_ALIASES: &[&str] = &[
    "VPDleaf", "VPDl", "VpdL", "VPD", "VpdA", "SVPleaf", "leaf_vpd", "vpd_leaf", "VPDL", "D",
    "Dleaf",
];
pub const DS_ALIASES: &[&str] = &["Ds", "D_s"];
pub const PATM_ALIASES: &[&str] = &[
    "Pa",
    "Patm",
    "P_atm",
    "pressure",
    "atm_pressure",
    "press",
    "Patm_kPa",
];
pub const A_ALIASES: &[&str] = &[
    "A",
    "A_solved",
    "Anet",
    "A_net",
    "Photo",
    "assimilation",
    "An",
];
pub const CS_ALIASES: &[&str] = &["Cs", "C_s", "CO2_s", "Ca", "CO2_a", "CO2", "Ca_s"];
/// µmol mol⁻¹.
pub const CS_DEFAULT: f64 = 420.0;

/// Water-potential names that are combined with a unit suffix ("Psi (kPa)", "h_cm").
const SOIL_PSI_BASES: &[&str] = &[
    "psi",
    "ψ",
    "wp",
    "water_potential",
    "matric_potential",
    "matric",
    "suction",
    "tension",
    "pressure_head",
    "head",
    "h",
];
/// Unit suffixes `soil_unit_in_name` recognises, so the column found is also read in its unit.
const SOIL_PSI_UNITS: &[&str] = &[
    "kPa", "hPa", "mbar", "bar", "MPa", "cm", "cmH2O", "m", "mH2O",
];

/// `P_ALIASES`, then the soil-specific names, then every water-potential name with a unit
/// suffix. Column names are matched ignoring case, spaces and punctuation, so "psikpa" finds
/// `Psi (kPa)`, `psi_kPa` and `PSI kPa`.
fn soil_psi_aliases() -> Vec<String> {
    let mut out: Vec<String> = P_ALIASES
        .iter()
        .chain(SOIL_EXTRA_PSI_ALIASES)
        .map(|s| s.to_string())
        .collect();
    for base in SOIL_PSI_BASES {
        for unit in SOIL_PSI_UNITS {
            out.push(format!("{base}_{unit}"));
        }
    }
    out
}

/// Runs of whitespace collapsed to one space.
fn collapse_space(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(c);
            in_space = false;
        }
    }
    out
}

/// Whether a column name looks like a hydraulic conductance: `K`, optionally followed by a
/// tissue or qualifier (`Kleaf`, `K_stem`, `k-max`), or `conductance` / `conductivity`.
fn is_conductance_name(s: &str) -> bool {
    const SUFFIXES: &[&str] = &[
        "leaf",
        "stem",
        "root",
        "shoot",
        "branch",
        "plant",
        "xylem",
        "stem_segment",
        "x",
        "h",
        "s",
        "sat",
        "max",
        "tot",
        "total",
    ];
    let l = s.to_ascii_lowercase();
    if l == "conductance" || l == "conductivity" {
        return true;
    }
    let Some(rest) = l.strip_prefix('k') else {
        return false;
    };
    if rest.is_empty() || SUFFIXES.contains(&rest) {
        return true;
    }
    // [\s_-] then a suffix (after the /\s+/ → " " collapse, the only whitespace left is " ")
    let mut cs = rest.chars();
    match cs.next() {
        Some(c) if c == '_' || c == '-' || c.is_whitespace() => SUFFIXES.contains(&cs.as_str()),
        _ => false,
    }
}

/// The first column named like a hydraulic conductance.
fn find_conductance_field(fields: &[String]) -> Option<String> {
    fields
        .iter()
        .find(|f| is_conductance_name(collapse_space(f).trim()))
        .cloned()
}

/// Soil water-potential units: label and factor to MPa. `--units` takes these labels.
pub const SOIL_UNITS: &[(&str, f64)] = &[
    ("MPa", 1.0),
    ("kPa", 0.001),
    ("hPa", 0.0001),
    ("bar", 0.1),
    ("m", 0.00980665),
    ("cm", 0.0000980665),
];

/// The unit a soil water-potential column names in its header ("Psi (bar)", "kPa",
/// "h_cm", ...), as an entry of `SOIL_UNITS`. `None` when the name doesn't say.
pub fn soil_unit_in_name(field: &str) -> Option<(&'static str, f64)> {
    let lower = field.to_lowercase();
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    let label = if lower.contains("mpa") {
        "MPa"
    } else if lower.contains("kpa") {
        "kPa"
    } else if lower.contains("hpa") || lower.contains("mbar") {
        "hPa"
    } else if lower.contains("bar") {
        "bar"
    } else if tokens
        .iter()
        .any(|t| matches!(*t, "cm" | "cmh2o" | "cmwater"))
    {
        "cm"
    } else if tokens.iter().any(|t| matches!(*t, "m" | "mh2o")) {
        "m"
    } else {
        return None;
    };
    SOIL_UNITS.iter().find(|(l, _)| *l == label).copied()
}

/// A water-potential column named in bar is converted to MPa.
fn bar_to_mpa(field: &str, vals: &mut [f64], warnings: &mut Vec<Warning>) {
    if field.to_ascii_lowercase().contains("bar") {
        for v in vals.iter_mut() {
            *v /= 10.0;
        }
        warnings.push(Warning::new(
            "unit_converted_by_name",
            format!("Water potential \"{field}\" is named in bar; converted to MPa (÷10)."),
        ));
    }
}

/// `norm`: lowercase, then drop everything but ASCII letters and digits.
fn norm(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        .collect()
}

/// Like `norm`, but keeping non-ASCII letters, so "θ" and "σ" stay distinct instead of
/// both becoming "".
fn norm_unicode(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

struct CsvTable {
    fields: Vec<String>,
    rows: Vec<Vec<String>>,
    /// Column index by name, and by normalized name for loose matching.
    index: IndexMap<String, usize>,
    by_norm: IndexMap<String, String>,
    by_unicode: IndexMap<String, String>,
}

impl CsvTable {
    /// The first column matching one of `cands`, ignoring case, spaces and punctuation.
    /// The first candidate that names a column: matched on `norm_unicode`, then on the ASCII
    /// `norm` (so "Ψ (MPa)" still matches "MPa"). An empty normalized name never matches.
    fn find_field(&self, cands: &[&str]) -> Option<String> {
        cands.iter().find_map(|c| {
            let (u, a) = (norm_unicode(c), norm(c));
            let hit = |m: &IndexMap<String, String>, k: &str| {
                (!k.is_empty())
                    .then(|| m.get(k))
                    .flatten()
                    .filter(|h| !h.is_empty())
                    .cloned()
            };
            hit(&self.by_unicode, &u).or_else(|| hit(&self.by_norm, &a))
        })
    }
    /// The cell in `field` as a number; NaN when missing or not a number.
    fn num(&self, r: &[String], field: &str) -> f64 {
        self.index
            .get(field)
            .map_or(f64::NAN, |&j| numbers::parse_finite(Table::cell(r, j)))
    }
}

fn read_csv(text: &str) -> Result<CsvTable, String> {
    let table = Table::parse(text).ok_or("No header row found in the CSV.")?;
    let fields = table.names();
    if fields.is_empty() {
        return Err("No header row found in the CSV.".into());
    }
    if table.rows.is_empty() {
        return Err("CSV has a header but no data rows.".into());
    }
    let mut index = IndexMap::new();
    for (j, h) in table.header.iter().enumerate() {
        if !h.is_empty() {
            index.entry(h.clone()).or_insert(j);
        }
    }
    // The first column wins when two normalize to the same name.
    let mut by_norm: IndexMap<String, String> = IndexMap::new();
    let mut by_unicode: IndexMap<String, String> = IndexMap::new();
    for f in &fields {
        by_norm.entry(norm(f)).or_insert_with(|| f.clone());
        by_unicode
            .entry(norm_unicode(f))
            .or_insert_with(|| f.clone());
    }
    Ok(CsvTable {
        fields,
        rows: table.rows,
        index,
        by_norm,
        by_unicode,
    })
}

fn missing_error(missing: &[String], fields: &[String]) -> String {
    format!(
        "CSV is missing required column(s): {}. Found columns: {}.",
        missing.join(", "),
        fields.join(", ")
    )
}

fn first(aliases: &[&str], n: usize) -> String {
    aliases[..n].join(" or ")
}

fn columns(pairs: Vec<(&str, Vec<f64>)>) -> Columns {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

fn mapping(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// Vulnerability curve data: water potential, and PLC or a conductance column.
pub fn parse_vulnerability(text: &str) -> Result<ParsedFitData, String> {
    let t = read_csv(text)?;

    let p_field = t.find_field(P_ALIASES);
    let plc_field = t.find_field(PLC_ALIASES);
    let k_field = if plc_field.is_some() {
        None
    } else {
        find_conductance_field(&t.fields)
    };
    let y_field = plc_field.clone().or_else(|| k_field.clone());
    let mut missing = Vec::new();
    if p_field.is_none() {
        missing.push(format!("water potential ({})", first(P_ALIASES, 4)));
    }
    if y_field.is_none() {
        missing.push(format!(
            "PLC ({}) or conductance (K, Kleaf, Kstem, …)",
            first(PLC_ALIASES, 3)
        ));
    }
    if !missing.is_empty() {
        return Err(missing_error(&missing, &t.fields));
    }
    let (p_field, y_field) = (p_field.unwrap(), y_field.unwrap());

    let mut p = Vec::new();
    let mut y = Vec::new();
    for r in &t.rows {
        let pv = t.num(r, &p_field);
        let lv = t.num(r, &y_field);
        if pv.is_finite() && lv.is_finite() {
            p.push(pv);
            y.push(lv);
        }
    }
    if p.is_empty() {
        return Err("No usable numeric rows found.".into());
    }

    let mut warnings = vec![Warning::new(
        "columns_mapped",
        format!(
            "Using Ψ=\"{p_field}\", {}.",
            match &plc_field {
                Some(f) => format!("PLC=\"{f}\""),
                None => format!("K=\"{}\"", k_field.as_deref().unwrap()),
            }
        ),
    )];

    bar_to_mpa(&p_field, &mut p, &mut warnings);
    if numbers::min_of(&p) >= 0.0 && numbers::max_of(&p) > 0.0 {
        for v in p.iter_mut() {
            *v = -*v;
        }
        warnings.push(Warning::new(
            "sign_convention",
            "Water potential was positive; treated as tension and negated to MPa.",
        ));
    }

    if let Some(plc) = &plc_field {
        let plc_is_fraction = numbers::max_of(&y) <= 1.5;
        for v in y.iter_mut() {
            let frac = if plc_is_fraction { *v } else { *v / 100.0 };
            *v = 1.0 - frac;
        }
        warnings.push(Warning::new(
            "relative_conductance",
            format!("Fitting relative conductance K = 1 − PLC/100 from \"{plc}\"."),
        ));
    } else if !(numbers::max_of(&y) > 0.0) {
        return Err(format!(
            "Conductance column \"{}\" has no positive values to fit.",
            k_field.as_deref().unwrap()
        ));
    }
    let kmax = numbers::max_of(&y);

    let n = p.len();
    Ok(ParsedFitData {
        x: columns(vec![("P", p)]),
        k: Some(y.clone()),
        y,
        n,
        mapping: mapping(&[("P", &p_field), ("K", &y_field)]),
        warnings,
        kmax: Some(kmax),
    })
}

/// Pressure–volume curve data: RWC is the predictor (`w`), Ψ the response.
pub fn parse_pv_curve(text: &str) -> Result<ParsedFitData, String> {
    let t = read_csv(text)?;

    let w_field = t.find_field(RWC_ALIASES);
    let p_field = t.find_field(P_ALIASES);
    let mut missing = Vec::new();
    if w_field.is_none() {
        missing.push(format!(
            "relative water content ({})",
            first(RWC_ALIASES, 3)
        ));
    }
    if p_field.is_none() {
        missing.push(format!("water potential ({})", first(P_ALIASES, 4)));
    }
    if !missing.is_empty() {
        return Err(missing_error(&missing, &t.fields));
    }
    let (w_field, p_field) = (w_field.unwrap(), p_field.unwrap());

    let mut w = Vec::new();
    let mut y = Vec::new();
    for r in &t.rows {
        let wv = t.num(r, &w_field);
        let pv = t.num(r, &p_field);
        if wv.is_finite() && pv.is_finite() {
            w.push(wv);
            y.push(pv);
        }
    }
    if w.is_empty() {
        return Err("No usable numeric rows found.".into());
    }

    let mut warnings = vec![Warning::new(
        "columns_mapped",
        format!("Using RWC=\"{w_field}\", Ψ=\"{p_field}\"."),
    )];

    bar_to_mpa(&p_field, &mut y, &mut warnings);
    if numbers::max_of(&w) > 1.5 {
        for v in w.iter_mut() {
            *v /= 100.0;
        }
        warnings.push(Warning::new(
            "unit_guessed_from_values",
            "RWC looked like a percent; scaled to a 0–1 fraction.",
        ));
    }
    if numbers::min_of(&y) >= 0.0 && numbers::max_of(&y) > 0.0 {
        for v in y.iter_mut() {
            *v = -*v;
        }
        warnings.push(Warning::new(
            "sign_convention",
            "Water potential was positive; treated as tension and negated to MPa.",
        ));
    }

    let n = w.len();
    Ok(ParsedFitData {
        x: columns(vec![("w", w)]),
        y,
        n,
        mapping: mapping(&[("w", &w_field), ("P", &p_field)]),
        warnings,
        k: None,
        kmax: None,
    })
}

/// Rebuild an instrument export (category row, header row,
/// units row, LI-6800 constant tables) as a plain single-header CSV. A plain CSV, or one
/// whose header is not recognized, is returned unchanged.
fn strip_instrument_header(text: &str, anchor_aliases: &[&str]) -> String {
    let grid = read_rows(text);
    if grid.len() < 2 {
        return text.to_string();
    }
    let anchors: HashSet<String> = anchor_aliases.iter().map(|a| norm(a)).collect();

    let mut h_idx: Option<usize> = None;
    for (i, row) in grid.iter().enumerate().take(40) {
        if row.iter().any(|c| anchors.contains(&norm(c))) {
            h_idx = Some(i);
            break;
        }
    }
    let Some(h_idx) = h_idx else {
        return text.to_string();
    };

    let mut data_start = h_idx + 1;
    let looks_like_units = |row: Option<&Vec<String>>| {
        let vals: Vec<&String> = row
            .map(|r| r.iter().filter(|c| !c.trim().is_empty()).collect())
            .unwrap_or_default();
        if vals.is_empty() {
            return false;
        }
        let numeric = vals
            .iter()
            .filter(|c| numbers::parse(c).is_finite())
            .count();
        (numeric as f64) / (vals.len() as f64) < 0.5
    };
    if looks_like_units(grid.get(data_start)) {
        data_start += 1;
    }

    if h_idx == 0 && data_start == 1 {
        return text.to_string();
    }
    let mut out: Vec<Vec<String>> = Vec::with_capacity(grid.len());
    out.push(grid[h_idx].iter().map(|c| c.trim().to_string()).collect());
    out.extend(grid.iter().skip(data_start).cloned());
    write_rows(out)
}

/// BTA data: gs from light Q and the leaf-to-air mole-fraction deficit Ds.
pub fn parse_stomatal(text: &str) -> Result<ParsedFitData, String> {
    let t = read_csv(&strip_instrument_header(text, GS_ALIASES))?;

    let gs_field = t.find_field(GS_ALIASES);
    let q_field = t.find_field(Q_ALIASES);
    let ds_field = t.find_field(DS_ALIASES);
    let vpd_field = if ds_field.is_some() {
        None
    } else {
        t.find_field(VPD_ALIASES)
    };
    let patm_field = t.find_field(PATM_ALIASES);

    let mut missing = Vec::new();
    if gs_field.is_none() {
        missing.push(format!("stomatal conductance ({})", first(GS_ALIASES, 3)));
    }
    if q_field.is_none() {
        missing.push(format!("light ({})", first(Q_ALIASES, 4)));
    }
    if ds_field.is_none() && vpd_field.is_none() {
        missing.push(format!("VPD ({}) or Ds", first(VPD_ALIASES, 3)));
    }
    if !missing.is_empty() {
        return Err(missing_error(&missing, &t.fields));
    }
    let (gs_field, q_field) = (gs_field.unwrap(), q_field.unwrap());
    let d_field = ds_field.clone().or_else(|| vpd_field.clone()).unwrap();

    let mut gs = Vec::new();
    let mut q = Vec::new();
    let mut ds = Vec::new();
    for r in &t.rows {
        let g = t.num(r, &gs_field);
        let qv = t.num(r, &q_field);
        let d_raw = t.num(r, &d_field);
        if !(g.is_finite() && qv.is_finite() && d_raw.is_finite()) {
            continue;
        }
        gs.push(g);
        q.push(qv);
        if ds_field.is_some() {
            ds.push(d_raw);
        } else {
            // VPD (kPa) → Ds (mmol mol⁻¹): Ds = VPD/Patm · 1000.
            let patm = patm_field.as_ref().map_or(f64::NAN, |f| t.num(r, f));
            let p = if patm.is_finite() && patm > 0.0 {
                patm
            } else {
                101.325
            };
            ds.push((d_raw / p) * 1000.0);
        }
    }
    if gs.is_empty() {
        return Err("No usable numeric rows found.".into());
    }

    let mut warnings = vec![Warning::new(
        "columns_mapped",
        format!(
            "Using gs=\"{gs_field}\", Q=\"{q_field}\", {}.",
            match &ds_field {
                Some(f) => format!("Ds=\"{f}\""),
                None => format!("VPD=\"{d_field}\""),
            }
        ),
    )];

    if numbers::max_of(&gs) > 10.0 {
        for v in gs.iter_mut() {
            *v /= 1000.0;
        }
        warnings.push(Warning::new(
            "unit_guessed_from_values",
            "Stomatal conductance looked like mmol m⁻² s⁻¹; scaled to mol (÷1000).",
        ));
    }
    if ds_field.is_some() && numbers::max_of(&ds) < 1.0 {
        for v in ds.iter_mut() {
            *v *= 1000.0;
        }
        warnings.push(Warning::new(
            "unit_guessed_from_values",
            format!("Ds \"{d_field}\" looked like mol/mol; scaled to mmol/mol (×1000)."),
        ));
    } else if let Some(vpd) = &vpd_field {
        warnings.push(Warning::new(
            "vpd_converted",
            format!(
                "Converted VPD \"{vpd}\" (kPa) to Ds (mmol/mol){}.",
                match &patm_field {
                    Some(f) => format!(" using Patm \"{f}\""),
                    None => " at 101.325 kPa".into(),
                }
            ),
        ));
    }

    let n = gs.len();
    Ok(ParsedFitData {
        x: columns(vec![("Q", q), ("Ds", ds)]),
        y: gs,
        n,
        mapping: mapping(&[("gs", &gs_field), ("Q", &q_field), ("Ds", &d_field)]),
        warnings,
        k: None,
        kmax: None,
    })
}

/// Medlyn data: gs with net assimilation A, leaf-surface CO2 Cs and VPD D (kPa).
pub fn parse_medlyn(text: &str) -> Result<ParsedFitData, String> {
    let t = read_csv(&strip_instrument_header(text, GS_ALIASES))?;

    let gs_field = t.find_field(GS_ALIASES);
    let a_field = t.find_field(A_ALIASES);
    let cs_field = t.find_field(CS_ALIASES);
    let vpd_field = t.find_field(VPD_ALIASES);
    let ds_field = if vpd_field.is_some() {
        None
    } else {
        t.find_field(DS_ALIASES)
    };
    let patm_field = t.find_field(PATM_ALIASES);

    let mut missing = Vec::new();
    if gs_field.is_none() {
        missing.push(format!("stomatal conductance ({})", first(GS_ALIASES, 3)));
    }
    if a_field.is_none() {
        missing.push(format!("net assimilation ({})", first(A_ALIASES, 3)));
    }
    if vpd_field.is_none() && ds_field.is_none() {
        missing.push(format!("VPD ({}) or Ds", first(VPD_ALIASES, 3)));
    }
    if !missing.is_empty() {
        return Err(missing_error(&missing, &t.fields));
    }
    let (gs_field, a_field) = (gs_field.unwrap(), a_field.unwrap());
    let d_field = vpd_field.clone().or_else(|| ds_field.clone()).unwrap();

    let mut gs = Vec::new();
    let mut a = Vec::new();
    let mut cs = Vec::new();
    let mut d = Vec::new();
    for r in &t.rows {
        let g = t.num(r, &gs_field);
        let av = t.num(r, &a_field);
        let c = cs_field.as_ref().map_or(CS_DEFAULT, |f| t.num(r, f));
        let d_raw = t.num(r, &d_field);
        if !(g.is_finite() && av.is_finite() && c.is_finite() && c > 0.0 && d_raw.is_finite()) {
            continue;
        }
        gs.push(g);
        a.push(av);
        cs.push(c);
        if vpd_field.is_some() {
            d.push(d_raw);
        } else {
            // Ds (mmol mol⁻¹) → D (kPa): D = Ds·Patm/1000.
            let patm = patm_field.as_ref().map_or(f64::NAN, |f| t.num(r, f));
            let p = if patm.is_finite() && patm > 0.0 {
                patm
            } else {
                101.325
            };
            d.push((d_raw * p) / 1000.0);
        }
    }
    if gs.is_empty() {
        return Err("No usable numeric rows found.".into());
    }

    let cs_default = numbers::text(CS_DEFAULT);
    let mut warnings = vec![Warning::new(
        "columns_mapped",
        format!(
            "Using gs=\"{gs_field}\", A=\"{a_field}\", Cs={}, {}.",
            match &cs_field {
                Some(f) => format!("\"{f}\""),
                None => format!("{cs_default} µmol mol⁻¹ (no CO2 column)"),
            },
            match &vpd_field {
                Some(f) => format!("VPD=\"{f}\""),
                None => format!("Ds=\"{d_field}\""),
            }
        ),
    )];
    if cs_field.is_none() {
        warnings.push(Warning::new(
            "co2_assumed",
            format!("No CO2 column found; leaf-surface CO2 assumed to be {cs_default} µmol mol⁻¹."),
        ));
    }
    if numbers::max_of(&gs) > 10.0 {
        for v in gs.iter_mut() {
            *v /= 1000.0;
        }
        warnings.push(Warning::new(
            "unit_guessed_from_values",
            "Stomatal conductance looked like mmol m⁻² s⁻¹; scaled to mol (÷1000).",
        ));
    }
    if let Some(dsf) = &ds_field {
        warnings.push(Warning::new(
            "vpd_converted",
            format!(
                "Converted Ds \"{dsf}\" (mmol/mol) to VPD (kPa){}.",
                match &patm_field {
                    Some(f) => format!(" using Patm \"{f}\""),
                    None => " at 101.325 kPa".into(),
                }
            ),
        ));
    }

    let cs_name = cs_field
        .clone()
        .unwrap_or_else(|| format!("{cs_default} (default)"));
    let n = gs.len();
    Ok(ParsedFitData {
        x: columns(vec![("A", a), ("Cs", cs), ("D", d)]),
        y: gs,
        n,
        mapping: mapping(&[
            ("gs", &gs_field),
            ("A", &a_field),
            ("Cs", &cs_name),
            ("D", &d_field),
        ]),
        warnings,
        k: None,
        kmax: None,
    })
}

/// Soil retention data: ψ is the predictor (`psi`), θ the response.
pub fn parse_soil_retention(text: &str) -> Result<ParsedFitData, String> {
    let t = read_csv(text)?;

    let aliases = soil_psi_aliases();
    let p_field = t.find_field(&aliases.iter().map(String::as_str).collect::<Vec<_>>());
    let t_field = t.find_field(THETA_ALIASES);
    let mut missing = Vec::new();
    if p_field.is_none() {
        missing.push("matric potential (psi, suction, or head)".to_string());
    }
    if t_field.is_none() {
        missing.push(format!("water content ({})", first(THETA_ALIASES, 3)));
    }
    if !missing.is_empty() {
        return Err(missing_error(&missing, &t.fields));
    }
    let (p_field, t_field) = (p_field.unwrap(), t_field.unwrap());

    let mut psi = Vec::new();
    let mut theta = Vec::new();
    for r in &t.rows {
        let pv = t.num(r, &p_field);
        let tv = t.num(r, &t_field);
        if pv.is_finite() && tv.is_finite() {
            psi.push(pv);
            theta.push(tv);
        }
    }
    if psi.is_empty() {
        return Err("No usable numeric rows found.".into());
    }

    let mut warnings = vec![Warning::new(
        "columns_mapped",
        format!("Using ψ=\"{p_field}\", θ=\"{t_field}\"."),
    )];

    // Units are not converted here: the caller applies --units, or the unit the column
    // names (soil_unit_in_name), so the two can never both apply.
    if numbers::min_of(&psi) >= 0.0 && numbers::max_of(&psi) > 0.0 {
        for v in psi.iter_mut() {
            *v = -*v;
        }
        warnings.push(Warning::new(
            "sign_convention",
            "Matric potential was positive; treated as suction and negated.",
        ));
    }
    if numbers::max_of(&theta) > 1.5 {
        for v in theta.iter_mut() {
            *v /= 100.0;
        }
        warnings.push(Warning::new(
            "unit_guessed_from_values",
            "Water content looked like a percent; scaled to a 0–1 fraction.",
        ));
    }

    let n = psi.len();
    Ok(ParsedFitData {
        x: columns(vec![("psi", psi)]),
        y: theta,
        n,
        mapping: mapping(&[("psi", &p_field), ("theta", &t_field)]),
        warnings,
        k: None,
        kmax: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vulnerability_from_plc_and_k() {
        assert_eq!(soil_unit_in_name("Psi (bar)"), Some(("bar", 0.1)));
        assert_eq!(soil_unit_in_name("kPa"), Some(("kPa", 0.001)));
        assert_eq!(soil_unit_in_name("h_cm"), Some(("cm", 0.0000980665)));
        assert_eq!(soil_unit_in_name("psi_MPa"), Some(("MPa", 1.0)));
        assert_eq!(soil_unit_in_name("suction"), None);
        assert_eq!(soil_unit_in_name("matric"), None);
        let d = parse_soil_retention("h,σ,θ\n1,9,0.4\n10,9,0.3\n").unwrap();
        assert_eq!(d.mapping["theta"], "θ");
        let d = parse_soil_retention("ψ,θ\n1,0.4\n10,0.3\n").unwrap();
        assert_eq!(d.mapping["psi"], "ψ");
        let d = parse_vulnerability("Ψ (MPa),PLC\n-1,10\n-3,80\n").unwrap();
        assert_eq!(d.mapping["P"], "Ψ (MPa)");
        let d = parse_vulnerability("Psi (bar),PLC\n2,10\n4,0.5\n").unwrap();
        assert_eq!(d.x["P"], vec![-0.2, -0.4]);
        assert_eq!(d.y, vec![0.9, 0.995]);
        assert_eq!(d.mapping.keys().collect::<Vec<_>>(), ["P", "K"]);
        let d = parse_vulnerability("P,K leaf\n-1,3\n-2,1\n").unwrap();
        assert_eq!(d.kmax, Some(3.0));
        assert_eq!(d.warnings[0].code, "columns_mapped");
        assert_eq!(d.warnings[0].message, "Using Ψ=\"P\", K=\"K leaf\".");
    }

    #[test]
    fn conductance_names() {
        for n in [
            "K",
            "kleaf",
            "K_stem",
            "k-max",
            "K stem_segment",
            "Conductivity",
        ] {
            assert!(is_conductance_name(n), "{n}");
        }
        for n in ["Kl", "K__leaf", "conduct", "kPa"] {
            assert!(!is_conductance_name(n), "{n}");
        }
    }

    #[test]
    fn error_messages_are_specific() {
        assert_eq!(
            parse_vulnerability("").unwrap_err(),
            "No header row found in the CSV."
        );
        assert_eq!(
            parse_pv_curve("a,b\n1,2\n").unwrap_err(),
            "CSV is missing required column(s): relative water content (RWC or rwc or RWC (%)), water potential (P or Psi (MPa) or Psi(MPa) or WP (MPa)). Found columns: a, b."
        );
    }
}
