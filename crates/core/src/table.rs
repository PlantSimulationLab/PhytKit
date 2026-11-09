//! Reading and writing CSV text. The delimiter (comma, tab, semicolon or pipe) is
//! detected from the first rows, and rows with only blank cells are skipped.

const DELIMITERS: [u8; 4] = [b',', b'\t', b';', b'|'];

fn reader(text: &str, delimiter: u8) -> csv::Reader<&[u8]> {
    csv::ReaderBuilder::new().has_headers(false).flexible(true).delimiter(delimiter).from_reader(text.as_bytes())
}

fn is_blank(row: &[String]) -> bool {
    row.iter().all(|c| c.trim().is_empty())
}

/// The delimiter that splits the first rows into the same number of fields, preferring
/// the one that gives the most fields; comma if none does.
pub fn detect_delimiter(text: &str) -> u8 {
    let mut best = (b',', 1usize);
    for &d in &DELIMITERS {
        let counts: Vec<usize> = reader(text, d)
            .records()
            .filter_map(Result::ok)
            .filter(|r| r.iter().any(|c| !c.trim().is_empty()))
            .take(10)
            .map(|r| r.len())
            .collect();
        if let Some(&n) = counts.first() {
            if n > best.1 && counts.iter().all(|&c| c == n) {
                best = (d, n);
            }
        }
    }
    best.0
}

/// Every non-blank row, with the delimiter detected.
pub fn read_rows(text: &str) -> Vec<Vec<String>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    read_rows_with(text, detect_delimiter(text))
}

/// Every non-blank row, split on `delimiter`.
pub fn read_rows_with(text: &str, delimiter: u8) -> Vec<Vec<String>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    reader(text, delimiter)
        .records()
        .filter_map(Result::ok)
        .map(|r| r.iter().map(str::to_string).collect::<Vec<_>>())
        .filter(|r| !is_blank(r))
        .collect()
}

/// Rows as CSV text, one line per row, quoting cells where needed.
pub fn write_rows<R, S>(rows: impl IntoIterator<Item = R>) -> String
where
    R: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut w = csv::WriterBuilder::new().flexible(true).terminator(csv::Terminator::Any(b'\n')).from_writer(Vec::new());
    for row in rows {
        let cells: Vec<String> = row.into_iter().map(|c| c.as_ref().to_string()).collect();
        // A row with one empty cell would be written as an empty line; quote it instead.
        if cells.len() == 1 && cells[0].is_empty() {
            w.write_record([""]).ok();
        } else {
            w.write_record(&cells).ok();
        }
    }
    String::from_utf8(w.into_inner().unwrap_or_default()).unwrap_or_default()
}

/// A CSV with one header row: trimmed column names and the data rows.
#[derive(Clone, Debug)]
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    /// `None` for text with no rows at all.
    pub fn parse(text: &str) -> Option<Table> {
        let mut rows = read_rows(text);
        if rows.is_empty() {
            return None;
        }
        let header = rows.remove(0).iter().map(|h| h.trim().to_string()).collect();
        Some(Table { header, rows })
    }

    /// Non-empty column names.
    pub fn names(&self) -> Vec<String> {
        self.header.iter().filter(|h| !h.is_empty()).cloned().collect()
    }

    /// The column with this name, matched exactly and then ignoring case.
    pub fn index(&self, name: &str) -> Option<usize> {
        self.header.iter().position(|h| h == name).or_else(|| self.header.iter().position(|h| h.eq_ignore_ascii_case(name)))
    }

    /// The trimmed cell, or `""` past the end of a short row.
    pub fn cell(row: &[String], j: usize) -> &str {
        row.get(j).map_or("", |s| s.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_delimiters_and_skips_blank_rows() {
        assert_eq!(read_rows("a,b\n1,2\n\n , \n3,4\n"), vec![vec!["a", "b"], vec!["1", "2"], vec!["3", "4"]]);
        assert_eq!(read_rows("a\tb\n1\t2\n")[1], vec!["1", "2"]);
        assert_eq!(read_rows("a;b\n1,5;2\n")[1], vec!["1,5", "2"]);
        assert_eq!(read_rows("\u{feff}\"x, y\",z\n")[0], vec!["x, y", "z"]);
    }

    #[test]
    fn writes_and_reads_back() {
        let rows = vec![vec!["a", "b,c"], vec!["say \"hi\"", ""]];
        let text = write_rows(rows.clone());
        assert_eq!(text, "a,\"b,c\"\n\"say \"\"hi\"\"\",\n");
        assert_eq!(read_rows(&text), rows);
    }

    #[test]
    fn table_lookup() {
        let t = Table::parse(" Leaf ,psi\nL1,0.1\nL2\n").unwrap();
        assert_eq!(t.index("leaf"), Some(0));
        assert_eq!(Table::cell(&t.rows[1], 1), "");
    }
}
