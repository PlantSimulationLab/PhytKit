//! Reading `.xlsx` workbooks with `calamine`.

use crate::numbers;
use calamine::{open_workbook_from_rs, Data, Reader, Xlsx};
use std::io::Cursor;

type Workbook = Xlsx<Cursor<Vec<u8>>>;

fn open(buf: &[u8]) -> Result<Workbook, String> {
    open_workbook_from_rs(Cursor::new(buf.to_vec())).map_err(|e| format!("Could not read the workbook: {e}."))
}

/// The workbook's sheet names, in order.
pub fn sheet_names(buf: &[u8]) -> Result<Vec<String>, String> {
    Ok(open(buf)?.sheet_names())
}

/// Every row of a sheet (the first one when `name` is `None`), as cells.
pub fn read_sheet(buf: &[u8], name: Option<&str>) -> Result<Vec<Vec<Data>>, String> {
    let mut wb = open(buf)?;
    let name = match name {
        Some(n) => n.to_string(),
        None => wb.sheet_names().first().cloned().ok_or("Workbook has no readable sheet.")?,
    };
    let range = wb.worksheet_range(&name).map_err(|e| format!("Could not read sheet \"{name}\": {e}."))?;
    Ok(range.rows().map(|r| r.to_vec()).collect())
}

/// A cell as text: numbers in their shortest form, dates as ISO 8601, errors as shown in
/// a spreadsheet (`#DIV/0!`).
pub fn cell_text(c: &Data) -> String {
    match c {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(x) => numbers::text(*x),
        Data::Bool(b) => b.to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => e.to_string(),
        Data::DateTime(dt) => {
            if !dt.is_datetime() {
                return numbers::text(dt.as_f64());
            }
            let (y, mo, d, h, mi, s, _) = dt.to_ymd_hms_milli();
            let serial = dt.as_f64();
            if serial < 1.0 {
                format!("{h:02}:{mi:02}:{s:02}")
            } else if serial.fract() == 0.0 {
                format!("{y:04}-{mo:02}-{d:02}")
            } else {
                format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
            }
        }
    }
}

/// A cell as a number, when it holds one.
pub fn cell_number(c: &Data) -> Option<f64> {
    match c {
        Data::Int(i) => Some(*i as f64),
        Data::Float(x) => Some(*x),
        Data::String(s) => Some(numbers::parse(s)).filter(|x| x.is_finite()),
        _ => None,
    }
}

/// The first sheet as CSV text, without blank rows.
pub fn to_csv(buf: &[u8]) -> Result<String, String> {
    let rows = read_sheet(buf, None)?;
    let text: Vec<Vec<String>> = rows.iter().map(|r| r.iter().map(cell_text).collect::<Vec<_>>()).filter(|r| r.iter().any(|c| !c.is_empty())).collect();
    // Trailing empty cells come from other rows' widths; drop them so rows don't end in commas.
    let width = text.iter().map(|r| r.iter().rposition(|c| !c.is_empty()).map_or(0, |i| i + 1)).max().unwrap_or(0);
    Ok(crate::table::write_rows(text.into_iter().map(|mut r| {
        r.truncate(width);
        r
    })))
}
