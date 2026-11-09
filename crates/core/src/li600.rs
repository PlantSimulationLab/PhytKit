//! The Rizzo & Bailey (2025) LI-600 gsw correction.

use crate::numbers;
use crate::linalg::solve_linear;
use crate::table::read_rows;

#[derive(Clone, Copy, Debug)]
pub struct Li600Row {
    pub gsw: f64,
    pub tref: f64,
    pub tleaf: f64,
    pub rh_r: f64,
    pub rh_s: f64,
    pub flow: f64,
    pub p_atm: f64,
    pub e_apparent: f64,
}

#[derive(Clone, Debug)]
pub struct Li600Parsed {
    pub rows: Vec<Li600Row>,
    pub n: usize,
    pub warnings: Vec<String>,
    /// Every parsed cell, kept so a corrected file can be rebuilt from the original.
    pub grid: Vec<Vec<String>>,
    pub h_idx: usize,
    pub data_start: usize,
    pub units_row: bool,
    pub row_map: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct Li600Options {
    pub sidedness: f64,
    pub thermal_conductance: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct CorrectedRow {
    pub measured: f64,
    pub corrected: f64,
    pub overestimate: f64,
    pub tin: f64,
    pub ta_chamb: f64,
    pub tout: f64,
    pub wchamb: f64,
    pub e: f64,
    pub rh_s: f64,
    pub sidedness: f64,
    pub converged: bool,
}

const REQUIRED: [&str; 8] = ["gsw", "Tref", "Tleaf", "rh_r", "rh_s", "flow", "P_atm", "E_apparent"];

/// Parse a raw LI-600 CSV export (group row / name row / units row / data).
pub fn parse_li600(text: &str) -> Result<Li600Parsed, String> {
    let grid = read_rows(text);
    if grid.is_empty() {
        return Err("Empty file.".into());
    }
    let h_idx = (0..grid.len().min(5))
        .find(|&i| grid[i].iter().any(|c| c.trim().to_lowercase() == "gsw"))
        .ok_or_else(|| "Could not find a \"gsw\" column — is this an LI-600 export?".to_string())?;
    let header: Vec<String> = grid[h_idx].iter().map(|c| c.trim().to_string()).collect();
    let mut col = [0usize; 8];
    let mut missing = Vec::new();
    for (k, name) in REQUIRED.iter().enumerate() {
        match header.iter().position(|h| h.to_lowercase() == name.to_lowercase()) {
            Some(j) => col[k] = j,
            None => missing.push(*name),
        }
    }
    if !missing.is_empty() {
        return Err(format!("Missing column(s): {}.\nLI-600 export needs: {}.", missing.join(", "), REQUIRED.join(", ")));
    }
    let gsw_col = col[0];
    let mut data_start = h_idx + 1;
    let units_row = match grid.get(data_start).and_then(|r| r.get(gsw_col)) {
        Some(first) => first.trim() != "" && !numbers::parse(first).is_finite(),
        None => false,
    };
    if units_row {
        data_start += 1;
    }
    let (mut rows, mut row_map) = (Vec::new(), Vec::new());
    for i in data_start..grid.len() {
        let r = &grid[i];
        if r.len() <= gsw_col {
            continue;
        }
        let v = |k: usize| r.get(col[k]).map_or(f64::NAN, |s| numbers::parse(s));
        let row = Li600Row { gsw: v(0), tref: v(1), tleaf: v(2), rh_r: v(3), rh_s: v(4), flow: v(5), p_atm: v(6), e_apparent: v(7) };
        if !row.gsw.is_finite() || !row.tref.is_finite() || !row.flow.is_finite() {
            continue;
        }
        rows.push(row);
        row_map.push(i);
    }
    if rows.is_empty() {
        return Err("No usable numeric rows found.".into());
    }
    let n = rows.len();
    Ok(Li600Parsed { rows, n, warnings: vec![format!("Parsed {n} observation(s) (header row {}).", h_idx + 1)], grid, h_idx, data_start, units_row, row_map })
}

/// Newton solve of the 3x3 system with a finite-difference Jacobian and backtracking.
fn solve3(f: &dyn Fn(&[f64; 3]) -> [f64; 3], x0: [f64; 3]) -> ([f64; 3], bool) {
    let sse = |v: &[f64; 3]| v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    let mut x = x0;
    for _ in 0..60 {
        let fx = f(&x);
        let fnorm = sse(&fx);
        if fnorm < 1e-12 {
            return (x, true);
        }
        let mut jm = vec![vec![0.0; 3]; 3];
        for j in 0..3 {
            let h = numbers::max(1e-7, x[j].abs() * 1e-6);
            let mut xp = x;
            xp[j] += h;
            let fp = f(&xp);
            for k in 0..3 {
                jm[k][j] = (fp[k] - fx[k]) / h;
            }
        }
        let Some(dx) = solve_linear(&jm, &fx) else {
            return (x, false);
        };
        let mut lambda = 1.0;
        let step = |l: f64| [x[0] - l * dx[0], x[1] - l * dx[1], x[2] - l * dx[2]];
        let mut xn = step(lambda);
        let mut tries = 0;
        while sse(&f(&xn)) > fnorm && tries < 25 {
            lambda *= 0.5;
            xn = step(lambda);
            tries += 1;
        }
        x = xn;
    }
    let ok = sse(&f(&x)) < 1e-8;
    (x, ok)
}

/// Apply the correction to every parsed observation.
pub fn correct_li600(rows: &[Li600Row], opts: Li600Options) -> Vec<CorrectedRow> {
    let sidedness = opts.sidedness;
    let c_cond = opts.thermal_conductance;
    let (a, b, c) = (0.61365, 17.502, 240.97);
    let (cpa, cpw, lambdaw) = (29.14, 33.5, 45502.0);
    let s = 0.441786 * 1e-4; // m², LI-600 aperture area
    let gbw = 2.921;
    let es = |t: f64| a * ((b * t) / (t + c)).exp();
    let w = |t: f64, rh: f64, p: f64| (es(t) * rh) / p;
    let h = |t: f64, rh: f64, p: f64| {
        let wv = w(t, rh, p);
        (1.0 - wv) * cpa * t + wv * (lambdaw + cpw * t)
    };
    rows.iter()
        .map(|d| {
            let (t_in, t_leaf) = (d.tref, d.tleaf);
            let (rh_in, rh_out) = (d.rh_r / 100.0, d.rh_s / 100.0);
            let u_in = d.flow * 1e-6;
            let p = d.p_atm;
            let equations = |v: &[f64; 3]| -> [f64; 3] {
                let [t_out, e, gsw] = *v;
                let t_chamb = 0.5 * (t_in + t_out);
                let rh_chamb = 0.5 * (rh_in + rh_out);
                let w_chamb = w(t_chamb, rh_chamb, p);
                let w_in = w(t_in, rh_in, p);
                let w_out = w(t_in, rh_out, p);
                let w_leaf = w(t_leaf, 1.0, p);
                let h_in = h(t_in, rh_in, p);
                let h_out = h(t_in, rh_out, p);
                let q = c_cond * (t_in - t_chamb);
                let gtw = (gsw * gbw) / (gsw + gbw);
                [e - gtw * (w_leaf - w_chamb), e - (u_in * (w_out - w_in)) / (s * (1.0 - w_out)), e - ((q + u_in * h_in) / h_out - u_in) / s]
            };
            let (x, ok) = solve3(&equations, [t_in - 0.1, d.e_apparent * 1e-3, d.gsw * 0.75]);
            let [t_out_sol, e_sol, gsw_sol] = x;
            let corrected = if ok { gsw_sol * sidedness } else { f64::NAN };
            let ta_chamb = 0.5 * (t_in + t_out_sol);
            let wchamb = w(ta_chamb, 0.5 * (rh_in + rh_out), p);
            let measured = d.gsw * sidedness;
            CorrectedRow {
                measured,
                corrected,
                overestimate: if ok && corrected > 0.0 { (100.0 * (measured - corrected)) / corrected } else { f64::NAN },
                tin: t_in,
                ta_chamb,
                tout: t_out_sol,
                wchamb,
                e: e_sol * sidedness,
                rh_s: d.rh_s,
                sidedness,
                converged: ok,
            }
        })
        .collect()
}

const NEW_COLS: [&str; 7] = ["gsw_corrected", "E_corrected", "T_in_corrected", "Ta_chamb_corrected", "T_out_corrected", "W_chamb_corrected", "stomatal_sidedness"];
const NEW_UNITS: [&str; 7] = ["mol+1m-2s-1", "mmol+1m-2s-1", "C", "C", "C", "mol+1mol-1", ""];

fn fmt(v: f64) -> String {
    if v.is_finite() { numbers::fixed(v, 6) } else { String::new() }
}

fn cells_for(cr: Option<&CorrectedRow>) -> Vec<String> {
    match cr {
        None => vec![String::new(); 7],
        Some(cr) if !cr.converged => {
            let mut v = vec![String::new(); 6];
            v.push(fmt(cr.sidedness));
            v
        }
        Some(cr) => vec![fmt(cr.corrected), fmt(cr.e * 1e3), fmt(cr.tin), fmt(cr.ta_chamb), fmt(cr.tout), fmt(cr.wchamb), fmt(cr.sidedness)],
    }
}

fn csv_cell(v: &str) -> String {
    if v.contains(['"', ',', '\n']) { format!("\"{}\"", v.replace('"', "\"\"")) } else { v.to_string() }
}

/// Just the generated columns, one row per observation.
pub fn corrected_columns_csv(corrected: &[CorrectedRow]) -> String {
    let mut lines = vec![NEW_COLS.join(",")];
    for cr in corrected {
        lines.push(cells_for(Some(cr)).join(","));
    }
    lines.join("\n")
}

/// The original file with the generated columns appended.
pub fn corrected_full_csv(parsed: &Li600Parsed, corrected: &[CorrectedRow]) -> String {
    let by_grid: std::collections::HashMap<usize, &CorrectedRow> = corrected.iter().enumerate().map(|(i, cr)| (parsed.row_map[i], cr)).collect();
    let mut out = Vec::with_capacity(parsed.grid.len());
    for (r, base) in parsed.grid.iter().enumerate() {
        let extra: Vec<String> = if r == parsed.h_idx {
            NEW_COLS.iter().map(|s| s.to_string()).collect()
        } else if parsed.units_row && r == parsed.h_idx + 1 {
            NEW_UNITS.iter().map(|s| s.to_string()).collect()
        } else if r < parsed.data_start {
            vec!["CORRECTION".to_string(); 7]
        } else {
            cells_for(by_grid.get(&r).copied())
        };
        out.push(base.iter().cloned().chain(extra).map(|c| csv_cell(&c)).collect::<Vec<_>>().join(","));
    }
    out.join("\n")
}
