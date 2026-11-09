//! Parsed arguments and the error type. Parsing itself is in `options`.

use phytkit_core::numbers;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Flag {
    Str(String),
    Bool(bool),
    List(Vec<String>),
}

#[derive(Clone, Debug, Default)]
pub struct ParsedArgs {
    pub positional: Vec<String>,
    pub flags: HashMap<String, Flag>,
}

/// A failed command. `usage` is true when the command line itself was wrong (and so
/// retrying with corrected arguments will help), false when the input data or a file was.
#[derive(Debug)]
pub struct CliError {
    /// A stable identifier for the kind of failure, such as `unknown_option`.
    pub code: &'static str,
    pub message: String,
    pub usage: bool,
    /// What to do about it, when there's something specific to suggest.
    pub hint: Option<String>,
}

impl CliError {
    pub fn is_known_code(code: &str) -> bool {
        ERROR_CODES.iter().any(|(c, _)| *c == code)
    }
    pub fn usage(code: &'static str, message: impl Into<String>) -> Self {
        CliError { code, message: message.into(), usage: true, hint: None }
    }
    pub fn failed(code: &'static str, message: impl Into<String>) -> Self {
        CliError { code, message: message.into(), usage: false, hint: None }
    }
    /// The input file couldn't be read as the data this command needs.
    pub fn data(message: impl Into<String>) -> Self {
        Self::failed("invalid_data", message)
    }
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        let h = hint.into();
        self.hint = (!h.is_empty()).then_some(h);
        self
    }
}

pub type Result<T> = std::result::Result<T, CliError>;

/// Every error code, with what it means. Usage errors first, then failures.
pub const ERROR_CODES: &[(&str, &str)] = &[
    ("unknown_command", "No such command"),
    ("unknown_option", "The command has no such option"),
    ("missing_argument", "A required argument, such as the input file, is missing"),
    ("too_many_arguments", "More arguments than the command takes"),
    ("missing_value", "An option that needs a value was given without one"),
    ("invalid_value", "An option's value is the wrong type or out of range"),
    ("duplicate_option", "An option that takes one value was given twice with different values"),
    ("invalid_combination", "Options or arguments that can't be used together"),
    ("not_applicable", "An option that doesn't apply to this tool or model"),
    ("unknown_tool", "pk fit was given a tool it doesn't have"),
    ("unknown_model", "No such model, or not one for this tool"),
    ("unknown_parameter", "The model has no parameter by that name"),
    ("unknown_constraint", "The model has no soft constraint by that name"),
    ("nothing_to_fit", "Every parameter is fixed"),
    ("missing_input", "Data passed from Python lacks one of the model's input columns"),
    ("file_not_found", "An input file doesn't exist"),
    ("not_a_file", "An input path is a folder"),
    ("permission_denied", "An input file can't be read"),
    ("read_failed", "An input file couldn't be read for another reason"),
    ("write_failed", "An --out file couldn't be written"),
    ("invalid_data", "A file was read but doesn't hold the data the command needs"),
    ("rescale_failed", "Rescale mode couldn't compute a factor from the survey and curve data"),
];

impl ParsedArgs {
    pub fn str(&self, k: &str) -> Result<Option<String>> {
        match self.flags.get(k) {
            None => Ok(None),
            Some(Flag::Bool(_)) => Err(CliError::usage("missing_value", format!("--{k} needs a value."))),
            Some(Flag::List(l)) => Ok(l.last().cloned()),
            Some(Flag::Str(s)) => Ok(Some(s.clone())),
        }
    }
    pub fn num(&self, k: &str, dflt: f64) -> Result<f64> {
        match self.str(k)? {
            None => Ok(dflt),
            Some(s) => {
                let n = numbers::parse(&s);
                if n.is_finite() { Ok(n) } else { Err(CliError::usage("invalid_value", format!("--{k} must be a number; got \"{s}\"."))) }
            }
        }
    }
    pub fn bool(&self, k: &str, dflt: bool) -> bool {
        match self.flags.get(k) {
            None => dflt,
            Some(Flag::Bool(b)) => *b,
            Some(Flag::Str(s)) => !matches!(s.to_lowercase().as_str(), "false" | "0" | "no" | "off"),
            Some(Flag::List(l)) => !matches!(l.last().map(|s| s.to_lowercase()).as_deref(), Some("false" | "0" | "no" | "off")),
        }
    }
    pub fn list(&self, k: &str) -> Vec<String> {
        let arr: Vec<String> = match self.flags.get(k) {
            None | Some(Flag::Bool(_)) => return vec![],
            Some(Flag::Str(s)) => vec![s.clone()],
            Some(Flag::List(l)) => l.clone(),
        };
        arr.iter().flat_map(|s| s.split(',')).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    }
    /// `--set k=v` (repeatable or comma-joined) to an ordered numeric map.
    pub fn kv(&self, k: &str) -> Result<Vec<(String, f64)>> {
        let mut out: Vec<(String, f64)> = Vec::new();
        for item in self.list(k) {
            let (name, val) = match item.find('=') {
                Some(eq) if eq > 0 && eq + 1 < item.len() => (&item[..eq], &item[eq + 1..]),
                _ => return Err(CliError::usage("invalid_value", format!("--{k} expects name=value; got \"{item}\"."))),
            };
            let v = numbers::parse(val);
            if !v.is_finite() {
                return Err(CliError::usage("invalid_value", format!("--{k} {name}: \"{val}\" is not a number.")));
            }
            let name = name.trim().to_string();
            match out.iter_mut().find(|(n, _)| *n == name) {
                Some(slot) => slot.1 = v,
                None => out.push((name, v)),
            }
        }
        Ok(out)
    }
}
