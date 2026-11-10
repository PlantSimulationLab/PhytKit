//! Photosynthesis CSVs: one header row naming A, Ci, light and leaf temperature.
//! Temperatures below 200 are taken as °C and converted to Kelvin.

use crate::model::Columns;
use crate::numbers;
use crate::table::Table;
use crate::warn::Warning;
use indexmap::IndexMap;

/// Accepted column names for each input, in order of preference.
pub const CANDIDATES: [(&str, &[&str]); 4] = [
    ("A", &["A"]),
    ("Ci", &["Ci"]),
    ("Q", &["Qabs", "Q"]),
    ("T", &["Tleaf", "T"]),
];

/// Accepted column names for one input.
pub fn candidates(key: &str) -> &'static [&'static str] {
    CANDIDATES
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, c)| *c)
        .unwrap_or(&[])
}

/// A photosynthesis dataset ready to fit.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedDataset {
    /// `Ci`, `Q` and `T` (Kelvin), for rows with every input present.
    pub x: Columns,
    /// A, for the same rows.
    pub y: Vec<f64>,
    pub n: usize,
    /// The file column used for A, Ci, Q and T.
    pub mapping: IndexMap<String, String>,
    /// Every column as numbers, for all rows.
    pub columns: Columns,
    /// Row indices of each curve, when the file has a CurveID column.
    pub curves: Option<Vec<Vec<usize>>>,
    pub warnings: Vec<Warning>,
}

pub fn parse_csv(text: &str) -> Result<ParsedDataset, String> {
    let table = Table::parse(text)
        .filter(|t| !t.names().is_empty())
        .ok_or("No header row found in the CSV.")?;
    if table.rows.is_empty() {
        return Err("CSV has a header but no data rows.".into());
    }
    let fields = table.names();
    let find = |cands: &[&str]| {
        cands
            .iter()
            .find_map(|c| table.header.iter().position(|h| h.eq_ignore_ascii_case(c)))
    };

    let mut mapping: IndexMap<String, String> = IndexMap::new();
    let mut idx: IndexMap<&str, usize> = IndexMap::new();
    let mut missing = Vec::new();
    for (key, cands) in CANDIDATES {
        match find(cands) {
            Some(j) => {
                mapping.insert(key.to_string(), table.header[j].clone());
                idx.insert(key, j);
            }
            None => missing.push(format!("{key} ({})", cands.join(" or "))),
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "CSV is missing required column(s): {}. Found columns: {}.",
            missing.join(", "),
            fields.join(", ")
        ));
    }

    let column = |j: usize| -> Vec<f64> {
        table
            .rows
            .iter()
            .map(|r| numbers::parse_finite(Table::cell(r, j)))
            .collect()
    };
    let mut columns = Columns::new();
    for (j, name) in table.header.iter().enumerate() {
        if !name.is_empty() && !columns.contains_key(name) {
            columns.insert(name.clone(), column(j));
        }
    }

    let mut warnings = vec![Warning::new(
        "columns_mapped",
        format!(
            "Using A=\"{}\", Ci=\"{}\", light=\"{}\", T=\"{}\".",
            mapping["A"], mapping["Ci"], mapping["Q"], mapping["T"]
        ),
    )];
    let (a, ci, q, t_raw) = (
        column(idx["A"]),
        column(idx["Ci"]),
        column(idx["Q"]),
        column(idx["T"]),
    );
    let celsius = t_raw
        .iter()
        .filter(|v| v.is_finite() && **v < 200.0)
        .count();
    let t: Vec<f64> = t_raw
        .iter()
        .map(|&v| {
            if v.is_finite() && v < 200.0 {
                v + 273.15
            } else {
                v
            }
        })
        .collect();
    if celsius > 0 {
        warnings.push(Warning::new(
            "temperature_celsius",
            format!(
                "Treated \"{}\" as °C and converted {celsius} value(s) to Kelvin.",
                mapping["T"]
            ),
        ));
    }

    let curve_col = find(&["CurveID", "curve"]);
    let mut groups: IndexMap<String, Vec<usize>> = IndexMap::new();
    let (mut x_ci, mut x_q, mut x_t, mut y) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (i, row) in table.rows.iter().enumerate() {
        if [a[i], ci[i], q[i], t[i]].iter().all(|v| v.is_finite()) {
            if let Some(j) = curve_col {
                groups
                    .entry(Table::cell(row, j).to_string())
                    .or_default()
                    .push(y.len());
            }
            x_ci.push(ci[i]);
            x_q.push(q[i]);
            x_t.push(t[i]);
            y.push(a[i]);
        }
    }
    let dropped = table.rows.len() - y.len();
    if dropped > 0 {
        warnings.push(Warning::new(
            "rows_dropped",
            format!("Dropped {dropped} row(s) with missing/non-numeric values."),
        ));
    }
    if y.is_empty() {
        return Err("No usable numeric rows after cleaning.".into());
    }
    let curves = curve_col.map(|_| groups.into_values().collect::<Vec<_>>());
    warnings.push(match (&curves, curve_col) {
        (Some(c), Some(j)) => Warning::new("curves_detected", format!("Per-curve constraints: {} curve(s) via \"{}\".", c.len(), table.header[j])),
        _ => Warning::new("single_curve", "Per-curve constraints: single curve assumed (add a CurveID column for multiple curves)."),
    });

    let mut x = Columns::new();
    x.insert("Ci".into(), x_ci);
    x.insert("Q".into(), x_q);
    x.insert("T".into(), x_t);
    Ok(ParsedDataset {
        n: y.len(),
        x,
        y,
        mapping,
        columns,
        curves,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_and_converts() {
        let d = parse_csv("A,Ci,Qabs,Q,Tleaf,CurveID\n10,300,1500,1600,25,a\n12,400,1500,1600,25,b\nx,1,1,1,1,a\n").unwrap();
        assert_eq!(d.mapping["Q"], "Qabs");
        assert_eq!(d.n, 2);
        assert_eq!(d.x["T"], vec![298.15, 298.15]);
        assert_eq!(d.curves, Some(vec![vec![0], vec![1]]));
        assert_eq!(d.warnings.len(), 4);
    }

    #[test]
    fn error_messages_are_specific() {
        assert_eq!(
            parse_csv("").unwrap_err(),
            "No header row found in the CSV."
        );
        assert_eq!(
            parse_csv("A,Ci\n").unwrap_err(),
            "CSV has a header but no data rows."
        );
        assert_eq!(
            parse_csv("A,Ci,foo\n1,2,3").unwrap_err(),
            "CSV is missing required column(s): Q (Qabs or Q), T (Tleaf or T). Found columns: A, Ci, foo."
        );
    }
}
