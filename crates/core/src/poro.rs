//! Solve A, Ci and Cc from LI-600 gsw and a photosynthesis
//! parameter set.

use crate::model::{Columns, Params};
use crate::models::model_by_name;
use crate::numbers;
use crate::table::read_rows;

#[derive(Clone, Copy, Debug)]
pub struct PoroRow {
    pub gsw: f64,
    pub qamb: f64,
    pub tleaf: f64,
    pub etr: f64,
}

#[derive(Clone, Debug)]
pub struct PoroParsed {
    pub rows: Vec<PoroRow>,
    pub n: usize,
    pub used_corrected: bool,
    pub has_etr: bool,
    pub warnings: Vec<String>,
    pub grid: Vec<Vec<String>>,
    pub h_idx: usize,
    pub data_start: usize,
    pub units_row: bool,
    pub row_map: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct PoroOptions {
    pub ca: f64,
    pub absorptance: f64,
    pub min_gsw: f64,
}

pub const DEFAULT_MIN_GSW: f64 = 0.001;

impl Default for PoroOptions {
    fn default() -> Self {
        PoroOptions {
            ca: 420.0,
            absorptance: 0.85,
            min_gsw: DEFAULT_MIN_GSW,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PoroResult {
    pub a: f64,
    pub ci: f64,
    pub cc: f64,
    pub gsw: f64,
    pub qabs: f64,
    pub tleaf: f64,
    pub etr: f64,
    pub ok: bool,
    pub clamped: bool,
}

/// A photosynthesis model the solver can use.
pub struct PoroModel {
    pub name: &'static str,
    pub label: &'static str,
    model: &'static str,
    /// empiricalA takes the CO2 passed as Cc only when gm >= 99.
    force_gm_99: bool,
}

impl PoroModel {
    pub fn params(&self) -> Params {
        model_by_name(self.model)
            .expect("registered model")
            .params()
            .iter()
            .map(|p| (p.name.to_string(), p.default))
            .collect()
    }
    pub fn param_names(&self) -> Vec<&'static str> {
        model_by_name(self.model)
            .expect("registered model")
            .params()
            .iter()
            .map(|p| p.name)
            .collect()
    }
    /// Net A with `c` taken as the site of carboxylation (no internal mesophyll drawdown).
    pub fn assimilation(&self, p: &Params, c: f64, q: f64, tk: f64) -> f64 {
        let m = model_by_name(self.model).expect("registered model");
        let mut x = Columns::new();
        x.insert("Ci".into(), vec![c]);
        x.insert("Q".into(), vec![q]);
        x.insert("T".into(), vec![tk]);
        if self.force_gm_99 {
            let mut p2 = p.clone();
            p2.insert("gm".into(), 99.0);
            return m.predict(&p2, &x)[0];
        }
        x.insert("Aobs".into(), vec![0.0]);
        m.predict(p, &x)[0]
    }
    /// Mesophyll conductance, or None when absent or off (gm <= 0 or >= 99).
    pub fn gm(&self, p: &Params) -> Option<f64> {
        p.get("gm")
            .copied()
            .filter(|g| g.is_finite() && *g > 0.0 && *g < 99.0)
    }
}

pub static PORO_MODELS: [PoroModel; 3] = [
    PoroModel {
        name: "fvcb",
        label: "FvCB C3",
        model: "fvcb",
        force_gm_99: false,
    },
    PoroModel {
        name: "fvcbC4",
        label: "FvCB C4",
        model: "fvcbC4",
        force_gm_99: false,
    },
    PoroModel {
        name: "empiricalA",
        label: "Empirical A",
        model: "empiricalA",
        force_gm_99: true,
    },
];

const REQUIRED: [&str; 3] = ["gsw", "Qamb", "Tleaf"];

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Parse an LI-600 CSV export, taking `gsw_corrected` in place of `gsw` when present.
pub fn parse_poro_file(text: &str) -> Result<PoroParsed, String> {
    let grid = read_rows(text);
    if grid.is_empty() {
        return Err("Empty file.".into());
    }
    let h_idx = (0..grid.len().min(5))
        .find(|&i| grid[i].iter().any(|c| c.trim().to_lowercase() == "gsw"))
        .ok_or_else(|| "Could not find a \"gsw\" column — is this an LI-600 export?".to_string())?;
    let header: Vec<String> = grid[h_idx]
        .iter()
        .map(|c| c.trim().to_lowercase())
        .collect();
    let find = |name: &str| header.iter().position(|h| *h == name.to_lowercase());
    let missing: Vec<&str> = REQUIRED
        .iter()
        .copied()
        .filter(|n| find(n).is_none())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "Missing column(s): {}.\nNeeds: {} (and optionally gsw_corrected).",
            missing.join(", "),
            REQUIRED.join(", ")
        ));
    }
    let corr = find("gsw_corrected");
    let used_corrected = corr.is_some();
    let g_col = corr.unwrap_or_else(|| find("gsw").unwrap());
    let (q_col, t_col, e_col) = (find("Qamb").unwrap(), find("Tleaf").unwrap(), find("ETR"));
    let mut data_start = h_idx + 1;
    let units_row = match grid
        .get(data_start)
        .and_then(|r| r.get(find("gsw").unwrap()))
    {
        Some(first) => first.trim() != "" && !numbers::parse(first).is_finite(),
        None => false,
    };
    if units_row {
        data_start += 1;
    }
    let num_at = |r: &Vec<String>, j: usize| r.get(j).map_or(f64::NAN, |s| numbers::parse(s));
    let (mut rows, mut row_map) = (Vec::new(), Vec::new());
    let mut blank_corrected = 0;
    for i in data_start..grid.len() {
        let r = &grid[i];
        if r.len() <= g_col {
            continue;
        }
        let (gsw, qamb, tleaf) = (num_at(r, g_col), num_at(r, q_col), num_at(r, t_col));
        if !qamb.is_finite() || !tleaf.is_finite() {
            continue;
        }
        if !gsw.is_finite() || r[g_col].trim().is_empty() {
            if used_corrected {
                blank_corrected += 1;
            }
            continue;
        }
        let etr = e_col
            .map(|j| num_at(r, j))
            .filter(|v| v.is_finite())
            .unwrap_or(f64::NAN);
        rows.push(PoroRow {
            gsw,
            qamb,
            tleaf,
            etr,
        });
        row_map.push(i);
    }
    if rows.is_empty() {
        return Err("No usable numeric rows found.".into());
    }
    let n = rows.len();
    let mut warnings = vec![if used_corrected {
        format!("Using gsw_corrected ({n} observation{}).", plural(n))
    } else {
        format!(
            "No gsw_corrected column — using uncorrected gsw ({n} observation{}).",
            plural(n)
        )
    }];
    if blank_corrected > 0 {
        warnings.push(format!(
            "{blank_corrected} row(s) skipped: gsw_corrected blank."
        ));
    }
    let has_etr = rows.iter().any(|r| r.etr.is_finite());
    Ok(PoroParsed {
        rows,
        n,
        used_corrected,
        has_etr,
        warnings,
        grid,
        h_idx,
        data_start,
        units_row,
        row_map,
    })
}

/// Bracketed root of a monotone-increasing f on [lo, hi]; None if there's no sign change.
fn bisect(f: &dyn Fn(f64) -> f64, mut lo: f64, mut hi: f64) -> Option<f64> {
    let mut flo = f(lo);
    let fhi = f(hi);
    if !flo.is_finite() || !fhi.is_finite() || flo * fhi > 0.0 {
        return None;
    }
    if flo == 0.0 {
        return Some(lo);
    }
    if fhi == 0.0 {
        return Some(hi);
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let fm = f(mid);
        if !fm.is_finite() {
            return None;
        }
        if fm == 0.0 {
            return Some(mid);
        }
        if fm * flo < 0.0 {
            hi = mid;
        } else {
            lo = mid;
            flo = fm;
        }
        if hi - lo < 1e-9 {
            break;
        }
    }
    Some(0.5 * (lo + hi))
}

const C_LO: f64 = 0.01;
const C_HI: f64 = 2000.0;

/// Solve `f(c) = 0` for c in [C_LO, hi], where hi starts at max(2000, 2·Ca) and doubles
/// (up to 1e6) until it brackets the root. A positive A puts Ci below Ca, but a negative A
/// with a small conductance can put it far above, and Ca itself can exceed 2000.
fn solve_c(f: &dyn Fn(f64) -> f64, ca: f64) -> Option<f64> {
    let flo = f(C_LO);
    let mut hi = numbers::max(C_HI, 2.0 * ca);
    while hi < 1e6 && f(hi) * flo > 0.0 {
        hi *= 2.0;
    }
    bisect(f, C_LO, hi)
}

/// Solve A and Ci (and Cc) for one observation.
pub fn solve_poro_point(
    model: &PoroModel,
    p: &Params,
    row: &PoroRow,
    opts: &PoroOptions,
) -> PoroResult {
    let qabs = row.qamb * opts.absorptance;
    let tk = row.tleaf + 273.15;
    let mut gsw = row.gsw;
    let mut clamped = false;
    if !(gsw >= opts.min_gsw) {
        if opts.min_gsw > 0.0 && gsw.is_finite() {
            gsw = opts.min_gsw;
            clamped = true;
        } else {
            return PoroResult {
                a: f64::NAN,
                ci: f64::NAN,
                cc: f64::NAN,
                gsw: row.gsw,
                qabs,
                tleaf: row.tleaf,
                etr: row.etr,
                ok: false,
                clamped: false,
            };
        }
    }
    let base = PoroResult {
        a: f64::NAN,
        ci: f64::NAN,
        cc: f64::NAN,
        gsw,
        qabs,
        tleaf: row.tleaf,
        etr: row.etr,
        ok: false,
        clamped,
    };
    let gsc = gsw / 1.6;
    let a_at = |c: f64| model.assimilation(p, c, qabs, tk);
    let ca = opts.ca;
    if let Some(gm) = model.gm(p) {
        let gtot = (gsc * gm) / (gsc + gm);
        let Some(cc) = solve_c(&|c| a_at(c) - gtot * (ca - c), ca) else {
            return base;
        };
        let an = gtot * (ca - cc);
        return PoroResult {
            a: an,
            ci: cc + an / gm,
            cc,
            ok: true,
            ..base
        };
    }
    let Some(ci) = solve_c(&|c| a_at(c) - gsc * (ca - c), ca) else {
        return base;
    };
    PoroResult {
        a: a_at(ci),
        ci,
        cc: ci,
        ok: true,
        ..base
    }
}

pub fn solve_poro_a(
    model: &PoroModel,
    p: &Params,
    rows: &[PoroRow],
    opts: &PoroOptions,
) -> Vec<PoroResult> {
    rows.iter()
        .map(|r| solve_poro_point(model, p, r, opts))
        .collect()
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Matches `^\s*([A-Za-z_]\w*)\s*(?:[=:]|\s|,)\s*(-?[\d.]+(?:[eE][+-]?\d+)?)\s*$`.
fn match_kv_line(line: &str) -> Option<(String, String)> {
    let s = line.trim_start_matches(char::is_whitespace);
    let mut chars = s.char_indices().peekable();
    match chars.peek() {
        Some(&(_, c)) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return None,
    }
    let name_end = s.find(|c: char| !is_word(c)).unwrap_or(s.len());
    let name = &s[..name_end];
    let rest = &s[name_end..];
    // Gap: whitespace, at most one of '=' ':' ',', whitespace; at least one character.
    let after_ws = rest.trim_start_matches(char::is_whitespace);
    let (gap_has_sep, after_sep) = match after_ws.chars().next() {
        Some(c @ ('=' | ':' | ',')) => (true, &after_ws[c.len_utf8()..]),
        _ => (false, after_ws),
    };
    let value_start = after_sep.trim_start_matches(char::is_whitespace);
    if !gap_has_sep && value_start.len() == rest.len() {
        return None; // no separator character at all
    }
    // Value: -?[\d.]+ then an optional exponent, then only whitespace.
    let b = value_start.as_bytes();
    let mut i = 0;
    if i < b.len() && b[i] == b'-' {
        i += 1;
    }
    let digits_start = i;
    while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
        i += 1;
    }
    if i == digits_start {
        return None;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let ds = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > ds {
            i = j;
        }
    }
    if !value_start[i..].chars().all(char::is_whitespace) {
        return None;
    }
    Some((name.to_string(), value_start[..i].to_string()))
}

/// The model named in a `pk fit` JSON result (`{"model": ..., "params": ...}`), if any.
pub fn param_text_model(text: &str) -> Option<String> {
    let t = text.trim();
    if !t.starts_with('{') {
        return None;
    }
    let obj: serde_json::Value = serde_json::from_str(t).ok()?;
    obj.get("params")?;
    obj.get("model")?.as_str().map(str::to_string)
}

/// Read a parameter set: fitter JSON (`{params: ...}` or flat), a CSV (last row wins),
/// a two-column Parameter/Value table, or `name = value` lines. Only numeric entries.
pub fn parse_param_text(text: &str) -> Result<Params, String> {
    let t = text.trim();
    let mut out = Params::new();
    if t.is_empty() {
        return Ok(out);
    }
    if t.starts_with('{') {
        let obj: serde_json::Value =
            serde_json::from_str(t).map_err(|e| format!("Invalid JSON: {e}"))?;
        let src = match obj.get("params") {
            Some(p) if p.is_object() => p,
            _ => {
                if let Some(rs) = obj.get("results").and_then(|r| r.as_array()) {
                    return Err(format!(
                        "This is a batch fit result with {} fits; give the JSON of a single fit, such as one entry of its \"results\" list.",
                        rs.len()
                    ));
                }
                if obj.get("ok").is_some() && obj.get("command").is_some() {
                    return Err(
                        "This pk output has no \"params\"; give the JSON of a successful pk fit."
                            .to_string(),
                    );
                }
                &obj
            }
        };
        if let Some(m) = src.as_object() {
            for (k, v) in m {
                if let Some(x) = v.as_f64().filter(|x| x.is_finite()) {
                    out.insert(k.to_string(), x);
                }
            }
        }
        return Ok(out);
    }
    let lines: Vec<&str> = t
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .filter(|l| !l.trim().is_empty())
        .collect();
    let kv: Vec<Option<(String, String)>> = lines.iter().map(|l| match_kv_line(l)).collect();
    if kv.iter().all(|m| m.is_some()) {
        for (k, v) in kv.into_iter().flatten() {
            out.insert(k.to_string(), numbers::parse(&v));
        }
        return Ok(out);
    }
    let grid: Vec<Vec<String>> = read_rows(t)
        .into_iter()
        .map(|r| r.into_iter().map(|c| c.trim().to_string()).collect())
        .collect();
    if grid.len() < 2 {
        return Ok(out);
    }
    let header = &grid[0];
    let h0 = header.first().map(|s| s.to_lowercase());
    let h1 = header.get(1).map(|s| s.to_lowercase());
    if matches!(h0.as_deref(), Some("parameter" | "name" | "param"))
        && h1.as_deref() == Some("value")
    {
        for r in &grid[1..] {
            let name = r.first().map(String::as_str).unwrap_or("");
            let raw = r.get(1);
            let v = raw.map_or(f64::NAN, |s| numbers::parse(s));
            let ident = name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && name.chars().all(is_word);
            if !name.is_empty() && ident && raw.is_none_or(|s| !s.is_empty()) && v.is_finite() {
                out.insert(name.to_string(), v);
            }
        }
        return Ok(out);
    }
    let row = &grid[grid.len() - 1];
    for (j, h) in header.iter().enumerate() {
        let raw = row.get(j);
        let v = raw.map_or(f64::NAN, |s| numbers::parse(s));
        if !h.is_empty() && v.is_finite() && raw.is_none_or(|s| !s.is_empty()) {
            out.insert(h.to_string(), v);
        }
    }
    Ok(out)
}

const NEW_COLS: [&str; 5] = ["A_solved", "Ci_solved", "Cc_solved", "gsw_used", "Qabs"];
const NEW_UNITS: [&str; 5] = [
    "umol+1m-2s-1",
    "umol+1mol-1",
    "umol+1mol-1",
    "mol+1m-2s-1",
    "umol+1m-2s-1",
];

fn fmt(v: f64) -> String {
    if v.is_finite() {
        numbers::fixed(v, 4)
    } else {
        String::new()
    }
}
fn cells_for(r: Option<&PoroResult>) -> Vec<String> {
    match r {
        None => vec![String::new(); 5],
        Some(r) => vec![fmt(r.a), fmt(r.ci), fmt(r.cc), fmt(r.gsw), fmt(r.qabs)],
    }
}
fn csv_cell(v: &str) -> String {
    if v.contains(['"', ',', '\n']) {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}

pub fn poro_columns_csv(results: &[PoroResult]) -> String {
    std::iter::once(NEW_COLS.join(","))
        .chain(results.iter().map(|r| cells_for(Some(r)).join(",")))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn poro_full_csv(parsed: &PoroParsed, results: &[PoroResult]) -> String {
    let by_grid: std::collections::HashMap<usize, &PoroResult> = results
        .iter()
        .enumerate()
        .map(|(i, r)| (parsed.row_map[i], r))
        .collect();
    let mut out = Vec::with_capacity(parsed.grid.len());
    for (r, base) in parsed.grid.iter().enumerate() {
        let extra: Vec<String> = if r == parsed.h_idx {
            NEW_COLS.iter().map(|s| s.to_string()).collect()
        } else if parsed.units_row && r == parsed.h_idx + 1 {
            NEW_UNITS.iter().map(|s| s.to_string()).collect()
        } else if r < parsed.data_start {
            vec!["POROA".to_string(); 5]
        } else {
            cells_for(by_grid.get(&r).copied())
        };
        out.push(
            base.iter()
                .cloned()
                .chain(extra)
                .map(|c| csv_cell(&c))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_above_2000_ppm_and_in_the_dark() {
        let m = &PORO_MODELS[0];
        let p = m.params();
        let row = |gsw, qamb| PoroRow {
            gsw,
            qamb,
            tleaf: 25.0,
            etr: f64::NAN,
        };
        let opts = |ca| PoroOptions {
            ca,
            ..Default::default()
        };
        let r = solve_poro_point(m, &p, &row(0.3, 1500.0), &opts(2500.0));
        assert!(r.ok && r.ci > 2000.0 && r.ci < 2500.0, "{r:?}");
        // Dark respiration with the floor conductance puts Ci far above Ca.
        let r = solve_poro_point(m, &p, &row(0.001, 0.0), &opts(420.0));
        assert!(r.ok && r.a < 0.0 && r.ci > 2000.0, "{r:?}");
    }
}
