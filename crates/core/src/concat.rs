//! Merge several LI-6800 logs or CSVs into one CSV with a CurveID per curve (`pk concat`).

use crate::parse::candidates;
use crate::table::{write_rows, Table};
use indexmap::IndexSet;
use std::collections::HashSet;

const CURVE_ID: &str = "CurveID";

/// The merged CSV and what happened while merging.
#[derive(Clone, Debug, PartialEq)]
pub struct ConcatResult {
    pub csv: String,
    /// Distinct CurveID values in the output.
    pub curves: usize,
    /// True when CurveIDs were prefixed with their file's label to keep files apart.
    pub namespaced: bool,
    pub warnings: Vec<String>,
}

/// A file's label: its name without the extension or a leading LI-6800
/// `YYYY-MM-DD-HHMM_` date and time.
pub fn label_of(name: &str) -> String {
    let stem = match name.rsplit_once('.') {
        Some((stem, ext)) if !ext.is_empty() && ext.chars().all(|c| c.is_ascii_alphanumeric()) => stem,
        _ => name,
    };
    let b = stem.as_bytes();
    let digits = |from: usize, n: usize| b.len() >= from + n && b[from..from + n].iter().all(u8::is_ascii_digit);
    let sep = |i: usize| matches!(b.get(i), Some(b'-' | b'_'));
    if !(digits(0, 4) && sep(4) && digits(5, 2) && sep(7) && digits(8, 2)) {
        return stem.to_string();
    }
    let mut i = 10;
    if matches!(b.get(i), Some(b'-' | b'_' | b' ')) {
        let run = b[i + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
        if run >= 3 {
            i += 1 + run.min(4);
        }
    }
    if matches!(b.get(i), Some(b'-' | b'_' | b' ')) {
        i += 1;
    }
    stem[i..].to_string()
}

struct Source {
    label: String,
    table: Table,
}

impl Source {
    fn get(&self, row: &[String], field: &str) -> String {
        self.table.header.iter().position(|h| h == field).map_or(String::new(), |j| Table::cell(row, j).to_string())
    }
    fn has(&self, field: &str) -> bool {
        self.table.header.iter().any(|h| h.eq_ignore_ascii_case(field))
    }
}

/// A warning when some files have the preferred light column and others don't: the
/// fitter would drop the rows of the files without it.
fn light_column_warning(sources: &[Source]) -> Option<String> {
    if sources.len() < 2 {
        return None;
    }
    let q = candidates("Q");
    let winner = *q.iter().find(|c| sources.iter().any(|s| s.has(c)))?;
    let lacking: Vec<&Source> = sources.iter().filter(|s| !s.has(winner)).collect();
    if lacking.is_empty() {
        return None;
    }
    let lost: usize = lacking.iter().map(|s| s.table.rows.len()).sum();
    let others: Vec<&str> = q.iter().copied().filter(|c| *c != winner).collect();
    let others = if others.is_empty() { String::new() } else { format!(" (they log \"{}\")", others.join("\"/\"")) };
    let labels: Vec<&str> = lacking.iter().map(|s| s.label.as_str()).collect();
    Some(format!(
        "{} of {} file(s) have no \"{winner}\" column{others}. The fitter prefers \"{winner}\", so their {lost} row(s) would be dropped: {}. Merge logs recorded the same way.",
        lacking.len(),
        sources.len(),
        labels.join(", ")
    ))
}

/// Merge `(file name, CSV text)` pairs under one header holding every column. Without a
/// CurveID column, one is added with each file's label. Existing CurveIDs are kept, but
/// prefixed with the file label if two files share a value.
pub fn concat_csvs<N: AsRef<str>, C: AsRef<str>>(sources: &[(N, C)]) -> ConcatResult {
    let sources: Vec<Source> = sources
        .iter()
        .map(|(name, csv)| {
            let name = name.as_ref();
            let label = Some(label_of(name)).filter(|l| !l.is_empty()).unwrap_or_else(|| name.to_string());
            let table = Table::parse(csv.as_ref()).unwrap_or(Table { header: vec![], rows: vec![] });
            Source { label, table }
        })
        .collect();
    let id_field = sources.iter().flat_map(|s| s.table.header.iter()).find(|h| h.eq_ignore_ascii_case(CURVE_ID)).cloned();

    let mut header: IndexSet<String> = sources.iter().flat_map(|s| s.table.header.iter().filter(|h| !h.is_empty()).cloned()).collect();
    let key = id_field.clone().unwrap_or_else(|| CURVE_ID.to_string());
    header.insert(key.clone());

    // Two files sharing a CurveID value would merge their curves, so prefix them.
    let namespaced = id_field.as_ref().is_some_and(|f| {
        let mut seen = HashSet::new();
        sources.iter().any(|s| {
            let own: HashSet<String> = s.table.rows.iter().map(|r| s.get(r, f)).collect();
            own.into_iter().any(|v| !seen.insert(v))
        })
    });

    let key_col = header.get_index_of(&key).unwrap();
    let mut ids = HashSet::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    for s in &sources {
        for r in &s.table.rows {
            let mut out: Vec<String> = header.iter().map(|f| s.get(r, f)).collect();
            out[key_col] = match &id_field {
                None => s.label.clone(),
                Some(f) if namespaced => format!("{}:{}", s.label, s.get(r, f)),
                Some(f) => s.get(r, f),
            };
            ids.insert(out[key_col].clone());
            rows.push(out);
        }
    }
    ConcatResult {
        csv: write_rows(std::iter::once(header.iter().cloned().collect::<Vec<_>>()).chain(rows)),
        curves: ids.len(),
        namespaced,
        warnings: light_column_warning(&sources).into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(label_of("2026-06-15-1116_ACiQT_Leaf1.csv"), "ACiQT_Leaf1");
        assert_eq!(label_of("2026-06-01_12_x"), "12_x");
        assert_eq!(label_of("leaf.csv"), "leaf");
    }

    #[test]
    fn adds_and_namespaces_curve_ids() {
        let r = concat_csvs(&[("a.csv", "A,Ci\n1,2\n"), ("b.csv", "A,Ci\n3,4\n")]);
        assert_eq!(r.csv, "A,Ci,CurveID\n1,2,a\n3,4,b\n");
        assert_eq!(r.curves, 2);
        let r = concat_csvs(&[("a.csv", "A,CurveID\n1,1\n"), ("b.csv", "A,CurveID\n3,1\n")]);
        assert!(r.namespaced);
        assert_eq!(r.csv, "A,CurveID\n1,a:1\n3,b:1\n");
    }
}
