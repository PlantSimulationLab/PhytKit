//! Warnings: what a command noticed or decided while reading data or fitting.
//!
//! Every warning has a stable `code` (listed in `WARNING_CODES`, documented in
//! `docs/docs/reference/output.md`), a `level`, and a message for people:
//! - `info`: a choice PhytKit made that the reader may want to know (columns used, a
//!   documented unit convention applied). The result is as good as the data.
//! - `warning`: something that may make the result wrong (rows dropped, a unit guessed from
//!   the values, an assumed constant, no convergence, a parameter on its bound). Read it
//!   before using the result.

use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Info,
    Warning,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Info => "info",
            Level::Warning => "warning",
        }
    }
}

/// Every warning code, its level and what it means. Codes are part of the output contract:
/// never rename or remove one.
pub const WARNING_CODES: &[(&str, Level, &str)] = &[
    ("columns_mapped", Level::Info, "Which file columns were used for each model input (also in `mapping`)."),
    ("temperature_celsius", Level::Info, "Leaf temperatures below 200 were read as °C and converted to kelvin."),
    ("curves_detected", Level::Info, "Rows were grouped into curves by a CurveID column."),
    ("single_curve", Level::Info, "No CurveID column: every row is treated as one curve."),
    ("sign_convention", Level::Info, "Positive water or matric potentials were negated (read as tension or suction)."),
    ("relative_conductance", Level::Info, "Relative conductance K = 1 − PLC/100 was fitted from a PLC column."),
    ("unit_converted_by_name", Level::Info, "A column was converted because its name states its unit (bar, kPa, cm, ...)."),
    ("vpd_converted", Level::Info, "VPD (kPa) and Ds (mmol mol⁻¹) were converted into each other, using Patm or 101.325 kPa."),
    ("observations_parsed", Level::Info, "How many LI-600 observations were read, and from which header row."),
    ("gsw_corrected_used", Level::Info, "The LI-600 gsw_corrected column was used."),
    ("labels_disambiguated", Level::Info, "Files sharing a label were labelled by their full names so their curves stay apart."),
    ("rows_dropped", Level::Warning, "Rows with missing or non-numeric values were left out of the fit."),
    ("unit_guessed_from_values", Level::Warning, "A column's unit was guessed from its values (mmol→mol, mol/mol→mmol/mol, percent→fraction) and converted."),
    ("co2_assumed", Level::Warning, "No CO2 column: leaf-surface CO2 was assumed to be 420 µmol mol⁻¹."),
    ("soil_units_assumed", Level::Warning, "No --units given and the ψ column name states no unit: ψ was read as MPa."),
    ("soil_units_overridden", Level::Warning, "The ψ column name states one unit but --units gave another; --units was used."),
    ("gsw_uncorrected_used", Level::Warning, "No gsw_corrected column: the uncorrected LI-600 gsw was used."),
    ("rows_skipped", Level::Warning, "LI-600 rows with a blank gsw_corrected were skipped."),
    ("readings_unsolved", Level::Warning, "Some LI-600 readings could not be solved and are left out of the summary."),
    ("rows_would_drop", Level::Warning, "Some merged files lack the column the fitter prefers; their rows would be dropped by pk fit."),
    ("not_converged", Level::Warning, "The fit stopped before converging; the parameters may not be the best fit."),
    ("parameter_at_bound", Level::Warning, "A fitted parameter ended on its bound: its value is set by the bound, not the data."),
];

#[derive(Clone, Debug, PartialEq)]
pub struct Warning {
    pub code: &'static str,
    pub level: Level,
    pub message: String,
}

impl Warning {
    /// A warning with `code`; its level comes from `WARNING_CODES`.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        let level = WARNING_CODES
            .iter()
            .find(|(c, _, _)| *c == code)
            .map(|(_, l, _)| *l)
            .unwrap_or_else(|| panic!("warning code {code:?} is not in WARNING_CODES"));
        Warning {
            code,
            level,
            message: message.into(),
        }
    }
    pub fn to_json(&self) -> Value {
        json!({ "code": self.code, "level": self.level.as_str(), "message": self.message })
    }
}

/// Warnings as the JSON array commands put in their output.
pub fn to_json(ws: &[Warning]) -> Value {
    Value::Array(ws.iter().map(Warning::to_json).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_unique_and_snake_case() {
        let mut seen = std::collections::HashSet::new();
        for (c, _, d) in WARNING_CODES {
            assert!(seen.insert(*c), "duplicate {c}");
            assert!(
                c.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'),
                "{c}"
            );
            assert!(!d.is_empty());
        }
    }

    #[test]
    fn every_code_used_in_the_source_is_listed() {
        // Scan the sources of both crates for each Warning::new call so an unlisted code is
        // caught here rather than as a panic at run time.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut stack = vec![root.join("core/src"), root.join("cli/src")];
        let mut found = 0;
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") && !p.ends_with("warn.rs") {
                    let s = std::fs::read_to_string(&p).unwrap();
                    for part in s.split("Warning::new(").skip(1) {
                        // rustfmt may put the code on the next line.
                        let Some(rest) = part.trim_start().strip_prefix('"') else {
                            continue;
                        };
                        let code = &rest[..rest.find('"').unwrap()];
                        assert!(
                            WARNING_CODES.iter().any(|(c, _, _)| *c == code),
                            "{}: code {code:?} not in WARNING_CODES",
                            p.display()
                        );
                        found += 1;
                    }
                }
            }
        }
        assert!(found > 10);
    }
}
