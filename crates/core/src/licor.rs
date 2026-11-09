//! LI-COR LI-6800 logs: the native tab-delimited log (no file extension) and the `.xlsx`
//! export. Both hold a group row, a column-name row, a units row and the data; these
//! functions reduce them to a plain CSV with one header row.

use crate::parse::{parse_csv, ParsedDataset};
use crate::table::{read_rows_with, write_rows};
use crate::xlsx;
use std::collections::HashSet;

/// Rows of cell text to a CSV: the column-name row is the first holding both "A" and
/// "Ci"; data rows are those below it whose first named column is numeric.
fn licor_rows_to_csv(
    rows: &[Vec<String>],
    is_number: impl Fn(usize, usize) -> bool,
) -> Option<String> {
    let header_idx = rows
        .iter()
        .position(|r| r.iter().any(|c| c == "A") && r.iter().any(|c| c == "Ci"))?;
    let header = &rows[header_idx];
    let mut seen = HashSet::new();
    let keep: Vec<usize> = (0..header.len())
        .filter(|&c| !header[c].is_empty() && seen.insert(header[c].as_str()))
        .collect();
    let obs = *keep.first()?;
    let data: Vec<Vec<&str>> = (header_idx + 1..rows.len())
        .filter(|&i| is_number(i, obs))
        .map(|i| {
            keep.iter()
                .map(|&c| rows[i].get(c).map_or("", String::as_str))
                .collect()
        })
        .collect();
    if data.is_empty() {
        return None;
    }
    let names: Vec<&str> = keep.iter().map(|&c| header[c].as_str()).collect();
    Some(write_rows(std::iter::once(names).chain(data)))
}

/// The first sheet of any `.xlsx` workbook as CSV.
pub fn xlsx_to_csv(buf: &[u8]) -> Result<String, String> {
    xlsx::to_csv(buf)
}

/// An LI-6800 `.xlsx` export: the "Measurements" sheet, or the first sheet.
pub fn parse_licor_xlsx(buf: &[u8]) -> Result<ParsedDataset, String> {
    let names = xlsx::sheet_names(buf)?;
    let sheet = names
        .iter()
        .find(|n| *n == "Measurements")
        .map(String::as_str);
    let cells = xlsx::read_sheet(buf, sheet)?;
    let text: Vec<Vec<String>> = cells
        .iter()
        .map(|r| r.iter().map(xlsx::cell_text).collect())
        .collect();
    let csv = licor_rows_to_csv(&text, |i, j| cells[i].get(j).and_then(xlsx::cell_number).is_some()).ok_or(
        "Could not find the LI-6800 column header (looked for a row with both 'A' and 'Ci'). Is this a standard LI-6800 logfile?",
    )?;
    parse_csv(&csv)
}

/// An LI-6800 native log (a `[Header]` block, then `[Data]`) as CSV; `None` if `text`
/// isn't one, so callers can fall back to reading it as an ordinary CSV.
pub fn licor_text_to_csv(text: &str) -> Option<String> {
    let mut lines = text.lines();
    lines.by_ref().find(|l| l.trim() == "[Data]")?;
    let body: Vec<&str> = lines.collect();
    let rows: Vec<Vec<String>> = read_rows_with(&body.join("\n"), b'\t')
        .into_iter()
        .map(|r| r.into_iter().map(|c| c.trim().to_string()).collect())
        .collect();
    if rows.len() < 2 {
        return None;
    }
    licor_rows_to_csv(&rows, |i, j| {
        rows[i]
            .get(j)
            .is_some_and(|c| c.parse::<f64>().is_ok_and(f64::is_finite))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_log_to_csv() {
        let log = "[Header]\r\nFoo\tbar\r\n[Data]\r\nSysObs\tGasEx\tGasEx\r\nobs\tA\tCi\tA\r\n\tumol\tumol\r\n1\t10.5\t300\t9\r\n2\t11\t\"4,0\"\t9\r\n";
        assert_eq!(
            licor_text_to_csv(log).unwrap(),
            "obs,A,Ci\n1,10.5,300\n2,11,\"4,0\"\n"
        );
        assert_eq!(licor_text_to_csv("A,Ci\n1,2"), None);
    }
}
