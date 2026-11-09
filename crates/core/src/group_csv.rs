//! Split one CSV into one CSV per value of a column, so a file holding several curves
//! can be fitted as a batch (`pk fit --group-by`).

use crate::table::{write_rows, Table};
use indexmap::IndexMap;

/// One group: the column value, its row count, and the CSV (header plus its rows).
#[derive(Clone, Debug, PartialEq)]
pub struct CsvGroup {
    pub name: String,
    pub rows: usize,
    pub csv: String,
}

/// One CSV per distinct value of `column`, in order of first appearance. Rows with a
/// blank value are dropped. The column is matched exactly, then ignoring case.
pub fn split_csv_by_column(text: &str, column: &str) -> Result<Vec<CsvGroup>, String> {
    let table = Table::parse(text).ok_or("Empty file.")?;
    let j = table
        .index(column)
        .ok_or_else(|| format!("No column \"{column}\" in the file. Columns: {}.", table.names().join(", ")))?;
    let mut groups: IndexMap<&str, Vec<&Vec<String>>> = IndexMap::new();
    for row in &table.rows {
        let key = Table::cell(row, j);
        if !key.is_empty() {
            groups.entry(key).or_default().push(row);
        }
    }
    if groups.is_empty() {
        return Err(format!("Column \"{column}\" has no values to group by."));
    }
    Ok(groups
        .into_iter()
        .map(|(name, rows)| CsvGroup {
            name: name.to_string(),
            rows: rows.len(),
            csv: write_rows(std::iter::once(&table.header).chain(rows.iter().copied())),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_by_column() {
        let text = "Leaf,psi,RWC\nL1,0.1,1\nL1,0.2,0.9\nL2,0.1,1\n,0.3,0.8\n";
        let g = split_csv_by_column(text, "leaf").unwrap();
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].csv, "Leaf,psi,RWC\nL1,0.1,1\nL1,0.2,0.9\n");
        assert_eq!(split_csv_by_column(text, "x").unwrap_err(), "No column \"x\" in the file. Columns: Leaf, psi, RWC.");
        assert_eq!(split_csv_by_column("", "x").unwrap_err(), "Empty file.");
    }
}
