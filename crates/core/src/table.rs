//! Reading and writing CSV text. The delimiter (comma, tab, semicolon or pipe) is
//! detected from the first rows, and rows with only blank cells are skipped. In files not
//! delimited by commas, decimal commas ("1,5") are read as decimal points.

const DELIMITERS: [u8; 4] = *b",\t;|";

fn reader(text: &str, delimiter: u8) -> csv::Reader<&[u8]> {
    csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delimiter)
        .from_reader(text.as_bytes())
}

fn is_blank(row: &[String]) -> bool {
    row.iter().all(|c| c.trim().is_empty())
}

/// Fields up to the last non-blank one, so trailing delimiters don't count.
fn used_fields(r: &csv::StringRecord) -> usize {
    r.iter()
        .enumerate()
        .filter(|(_, c)| !c.trim().is_empty())
        .last()
        .map_or(0, |(i, _)| i + 1)
}

/// The delimiter that splits the header into the most fields (at least two) while at least
/// four in five of the first data rows have that many fields too, ignoring trailing empty
/// fields (instrument exports often end rows with a delimiter); comma if none does.
pub fn detect_delimiter(text: &str) -> u8 {
    let mut best = (b',', 1usize);
    for &d in &DELIMITERS {
        let counts: Vec<usize> = reader(text, d)
            .records()
            .filter_map(Result::ok)
            .filter(|r| r.iter().any(|c| !c.trim().is_empty()))
            .take(11)
            .map(|r| used_fields(&r))
            .collect();
        let Some((&n, data)) = counts.split_first() else {
            continue;
        };
        let agree = data.iter().filter(|&&c| c == n).count();
        if n > best.1 && (data.is_empty() || agree * 5 >= data.len() * 4) {
            best = (d, n);
        }
    }
    best.0
}

/// A number written with a decimal comma, such as "1,5", "-0,25" or "2,5e-3".
fn is_decimal_comma(s: &str) -> bool {
    let s = s.trim();
    let s = s.strip_prefix(['-', '+']).unwrap_or(s);
    let (mantissa, exp) = match s.find(['e', 'E']) {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };
    let Some((int, frac)) = mantissa.split_once(',') else {
        return false;
    };
    let digits = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
    (int.is_empty() || digits(int))
        && digits(frac)
        && exp.is_none_or(|e| digits(e.strip_prefix(['-', '+']).unwrap_or(e)))
}

/// A number written with a decimal point and digits after it, such as "1.5".
fn is_decimal_point(s: &str) -> bool {
    let s = s.trim();
    s.contains('.') && !s.contains(',') && s.parse::<f64>().is_ok()
}

/// Every non-blank row, with the delimiter detected.
pub fn read_rows(text: &str) -> Vec<Vec<String>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let d = detect_delimiter(text);
    let mut rows = read_rows_with(text, d);
    // Decimal commas: always in semicolon files (the European convention), and in tab or
    // pipe files when no cell uses a decimal point.
    let convert = d == b';'
        || (d != b','
            && !rows.iter().flatten().any(|c| is_decimal_point(c))
            && rows.iter().flatten().any(|c| is_decimal_comma(c)));
    if convert {
        for c in rows.iter_mut().flatten() {
            if is_decimal_comma(c) {
                *c = c.trim().replacen(',', ".", 1);
            }
        }
    }
    rows
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
    let mut w = csv::WriterBuilder::new()
        .flexible(true)
        .terminator(csv::Terminator::Any(b'\n'))
        .from_writer(Vec::new());
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
        let header = rows
            .remove(0)
            .iter()
            .map(|h| h.trim().to_string())
            .collect();
        Some(Table { header, rows })
    }

    /// Non-empty column names.
    pub fn names(&self) -> Vec<String> {
        self.header
            .iter()
            .filter(|h| !h.is_empty())
            .cloned()
            .collect()
    }

    /// The column with this name, matched exactly and then ignoring case.
    pub fn index(&self, name: &str) -> Option<usize> {
        self.header.iter().position(|h| h == name).or_else(|| {
            self.header
                .iter()
                .position(|h| h.eq_ignore_ascii_case(name))
        })
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
        assert_eq!(
            read_rows("a,b\n1,2\n\n , \n3,4\n"),
            vec![vec!["a", "b"], vec!["1", "2"], vec!["3", "4"]]
        );
        assert_eq!(read_rows("a\tb\n1\t2\n")[1], vec!["1", "2"]);
        assert_eq!(read_rows("a;b\n1,5;2\n")[1], vec!["1.5", "2"]);
        assert_eq!(read_rows("a;b\n-0,25;x,y\n")[1], vec!["-0.25", "x,y"]);
        // Trailing delimiters and an occasional short row don't defeat detection.
        assert_eq!(
            read_rows("A\tCi\tQ\n1\t2\t3\t\n4\t5\t6\t\n")[1][..3],
            ["1", "2", "3"]
        );
        assert_eq!(read_rows("a\tb\n1,5\t2\n")[1], vec!["1.5", "2"]);
        assert_eq!(read_rows("a\tb\n1,5\t2.5\n")[1], vec!["1,5", "2.5"]);
        assert_eq!(read_rows("a,b\n\"1,5\",2\n")[1], vec!["1,5", "2"]);
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
