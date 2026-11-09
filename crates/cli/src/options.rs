//! The commands `pk` knows, with their arguments and options. This one table drives
//! parsing, validation, `--help` and the `commands` section of `pk describe`.

use crate::args::{CliError, Flag, ParsedArgs, Result};
use phytkit_core::numbers;
use serde_json::{json, Value};

/// What an option takes.
#[derive(Clone, Copy)]
pub enum Kind {
    /// On or off: `--rows`, `--no-rows`, `--rows=false`.
    Switch,
    /// Text such as a file or column name; the string names the value in help.
    Text(&'static str),
    /// One of a fixed set of words (matched ignoring case).
    Choice(&'static [&'static str]),
    /// A number in a range.
    Number(Range),
    /// A whole number of at least this much.
    Count(u64),
    /// `name=number`; repeatable, and comma-separated pairs are accepted.
    Pairs,
    /// Names; repeatable, and comma-separated names are accepted.
    Names,
    /// Files; repeatable.
    Files,
}

#[derive(Clone, Copy)]
pub enum Range {
    Positive,
    NonNegative,
    /// Greater than 0 and at most 1.
    Fraction,
}

impl Range {
    fn check(self, v: f64) -> Option<&'static str> {
        match self {
            Range::Positive if v <= 0.0 => Some("greater than 0"),
            Range::NonNegative if v < 0.0 => Some("0 or greater"),
            Range::Fraction if v <= 0.0 || v > 1.0 => Some("greater than 0 and at most 1"),
            _ => None,
        }
    }
}

pub struct Opt {
    pub name: &'static str,
    pub kind: Kind,
    pub default: Option<&'static str>,
    pub help: &'static str,
}

impl Opt {
    pub fn repeatable(&self) -> bool {
        matches!(self.kind, Kind::Pairs | Kind::Names | Kind::Files)
    }
    /// The value as shown in help: `<number>`, `json|csv`, `<name=value>`.
    pub fn value_hint(&self) -> Option<String> {
        Some(match self.kind {
            Kind::Switch => return None,
            Kind::Text(what) => format!("<{what}>"),
            Kind::Choice(c) => c.join("|"),
            Kind::Number(_) => "<number>".into(),
            Kind::Count(_) => "<n>".into(),
            Kind::Pairs => "<name=value>".into(),
            Kind::Names => "<name,...>".into(),
            Kind::Files => "<file>".into(),
        })
    }
}

pub struct Arg {
    pub name: &'static str,
    pub help: &'static str,
    pub required: bool,
    /// Takes one or more values.
    pub many: bool,
}

/// A check on a command's positional arguments.
pub type ArgCheck = fn(&[String]) -> Result<()>;

pub struct Command {
    pub name: &'static str,
    pub summary: &'static str,
    pub args: &'static [Arg],
    pub options: &'static [Opt],
    pub example: &'static str,
    /// Checks the positional arguments given so far, before their count is checked, so a
    /// wrong first argument is reported as that rather than as a missing later one.
    pub check_args: Option<ArgCheck>,
}

const fn opt(name: &'static str, kind: Kind, default: Option<&'static str>, help: &'static str) -> Opt {
    Opt { name, kind, default, help }
}

const FORMAT_JSON: Opt = opt("format", Kind::Choice(&["json", "csv"]), Some("json"), "Output format; csv prints a table instead of JSON");
const ROWS: Opt = opt("rows", Kind::Switch, None, "Include every reading in the JSON output");

pub const COMMANDS: &[Command] = &[
    Command {
        name: "describe",
        summary: "List the tools, models, parameters, constraints and command options, or describe one model",
        args: &[Arg { name: "model", help: "A model name, to describe only that model", required: false, many: false }],
        options: &[],
        example: "pk describe fvcb",
        check_args: None,
    },
    Command {
        name: "fit",
        summary: "Fit a model to one or more data files",
        args: &[
            Arg { name: "tool", help: "photosynthesis, stomatal, hydraulics or soil", required: true, many: false },
            Arg { name: "file", help: "One or more data files (.csv, .xlsx, LI-6800 logs, or - for stdin), each fitted separately", required: true, many: true },
        ],
        options: &[
            opt("model", Kind::Text("model"), None, "Model to fit (run \"pk describe\" for each tool's models)"),
            opt("fit", Kind::Names, None, "Fit only these parameters"),
            opt("fix", Kind::Names, None, "Hold these parameters fixed"),
            opt("set", Kind::Pairs, None, "Starting value, or the value a fixed parameter is held at"),
            opt("max-iter", Kind::Count(1), Some("200"), "Iteration limit"),
            opt("no-constraint", Kind::Names, None, "Turn off one soft constraint"),
            opt("constraints", Kind::Choice(&["none"]), None, "Turn off every soft constraint"),
            opt("weight", Kind::Pairs, None, "Weight for one soft constraint (key=value)"),
            opt("units", Kind::Choice(&["MPa", "kPa", "bar", "m", "cm"]), Some("MPa"), "Units of water potential; soil only"),
            opt("group-by", Kind::Text("column"), None, "Split one file into a fit per value of this column"),
            opt("survey", Kind::Files, None, "Rescale mode: survey points to rescale A to; photosynthesis only"),
            opt("stat", Kind::Choice(&["mean", "median", "max"]), Some("mean"), "Statistic for rescaling"),
            opt("ref-ci", Kind::Number(Range::Positive), None, "Reference Ci for rescaling; defaults to the highest Ci in the curve"),
            opt("predicted", Kind::Switch, None, "Include predicted and observed values in the output"),
            FORMAT_JSON,
        ],
        example: "pk fit photosynthesis examples/aci.csv --model fvcb --fix gm",
        check_args: Some(crate::fit::check_args),
    },
    Command {
        name: "li600 correct",
        summary: "Energy-balance correction of LI-600 stomatal conductance (Rizzo & Bailey 2025)",
        args: &[Arg { name: "file", help: "An LI-600 export (.csv, .xlsx, or - for stdin)", required: true, many: false }],
        options: &[
            opt("sidedness", Kind::Number(Range::Positive), Some("1"), "1 for stomata on one side of the leaf, 2 for both"),
            opt("thermal-conductance", Kind::Number(Range::Positive), Some("0.007"), "Instrument heat exchange, W °C⁻¹"),
            ROWS,
            opt("out", Kind::Text("file"), None, "Also write the input file with the corrected columns added"),
            FORMAT_JSON,
        ],
        example: "pk li600 correct examples/li600.csv --sidedness 1 --out corrected.csv",
        check_args: None,
    },
    Command {
        name: "li600 estimate",
        summary: "Solve A, Ci and Cc for each LI-600 reading from its gsw and a photosynthesis parameter set",
        args: &[Arg { name: "file", help: "An LI-600 export, ideally already corrected (.csv, .xlsx, or - for stdin)", required: true, many: false }],
        options: &[
            opt("model", Kind::Choice(&["fvcb", "fvcbC4", "empiricalA"]), Some("fvcb"), "Photosynthesis model"),
            opt("params", Kind::Text("file"), None, "Parameter file: pk fit JSON, a CSV of fitted parameters, a Parameter/Value table, or name = value lines"),
            opt("set", Kind::Pairs, None, "Override one parameter"),
            opt("ca", Kind::Number(Range::Positive), Some("420"), "Ambient CO₂, µmol mol⁻¹"),
            opt("absorptance", Kind::Number(Range::Fraction), Some("0.85"), "Leaf absorptance applied to Qamb"),
            opt("min-gsw", Kind::Number(Range::NonNegative), Some("0.001"), "Conductance floor; 0 leaves lower readings unsolved"),
            ROWS,
            opt("out", Kind::Text("file"), None, "Also write the input file with the solved columns added"),
            FORMAT_JSON,
        ],
        example: "pk li600 estimate corrected.csv --params fvcb_fit.json --out with_A.csv",
        check_args: None,
    },
    Command {
        name: "concat",
        summary: "Merge LI-6800 logs or CSVs under one header, with a CurveID per file",
        args: &[Arg { name: "file", help: "Two or more LI-6800 logs or CSVs to merge", required: true, many: true }],
        options: &[
            opt("out", Kind::Text("file"), None, "Write the merged CSV to a file and print a JSON summary"),
            opt("format", Kind::Choice(&["json", "csv"]), Some("csv"), "Default csv without --out, json with it"),
        ],
        example: "pk concat leaf1 leaf2 leaf3 --out merged.csv",
        check_args: None,
    },
];

pub fn command(name: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.name == name)
}

// ---- "Did you mean" ----

/// Edit distance counting a swap of neighbouring letters as one edit.
fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}

/// The candidate the input most likely meant, if any is close: same letters ignoring case,
/// a small typo, or one containing the other.
pub fn suggest<'a>(input: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let input_l = input.to_lowercase();
    let n = input_l.chars().count();
    let limit = match n {
        0..=4 => 1,
        5..=8 => 2,
        _ => 3,
    };
    let mut best: Option<(usize, &'a str)> = None;
    for c in candidates {
        let cl = c.to_lowercase();
        let score = if cl == input_l {
            0
        } else {
            let d = distance(&input_l, &cl);
            if d <= limit {
                d
            } else if n >= 2 && (cl.contains(&input_l) || (cl.len() >= 3 && input_l.contains(&cl))) {
                limit + 1
            } else {
                continue;
            }
        };
        if best.is_none_or(|(s, _)| score < s) {
            best = Some((score, c));
        }
    }
    best.map(|(_, c)| c)
}

/// "Did you mean X? " or "", to start a hint.
pub fn did_you_mean(s: Option<&str>) -> String {
    s.map_or(String::new(), |s| format!("Did you mean {s}? "))
}

/// `a, b or c`.
pub fn either(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} or {last}", rest.join(", ")),
    }
}

/// A parameter the model doesn't have. `label` is how it was given: `--fix`, `init`.
pub fn unknown_parameter(label: &str, name: &str, model: &str, known: &[&str]) -> CliError {
    CliError::usage("unknown_parameter", format!("{label} {name}: {model} has no parameter \"{name}\".")).hint(format!("{}Parameters: {}.", did_you_mean(suggest(name, known.iter().copied())), known.join(", ")))
}

/// A soft constraint the model doesn't have.
pub fn unknown_constraint(label: &str, key: &str, model: &str, keys: &[&str]) -> CliError {
    if keys.is_empty() {
        return CliError::usage("not_applicable", format!("{label} {key}: {model} has no soft constraints."));
    }
    CliError::usage("unknown_constraint", format!("{label} {key}: {model} has no constraint \"{key}\".")).hint(format!("{}Constraints: {}.", did_you_mean(suggest(key, keys.iter().copied())), keys.join(", ")))
}

/// A model name that doesn't exist.
pub fn unknown_model(name: &str, known: &[&str]) -> CliError {
    CliError::usage("unknown_model", format!("Unknown model \"{name}\".")).hint(format!("{}Models: {}.", did_you_mean(suggest(name, known.iter().copied())), known.join(", ")))
}

// ---- Parsing ----

fn parse_switch(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Some(true),
        "false" | "no" | "off" | "0" => Some(false),
        _ => None,
    }
}

fn looks_like_number(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}

fn unknown_option(cmd: &Command, written: &str, name: &str) -> CliError {
    let names: Vec<String> = cmd.options.iter().map(|o| format!("--{}", o.name)).collect();
    let guess = suggest(name, cmd.options.iter().map(|o| o.name)).map(|n| format!("--{n}"));
    let elsewhere: Vec<&str> = COMMANDS.iter().filter(|c| c.name != cmd.name && c.options.iter().any(|o| o.name == name)).map(|c| c.name).collect();
    let mut message = format!("Unknown option {written} for pk {}.", cmd.name);
    if !elsewhere.is_empty() {
        message = format!("{written} is an option of pk {}, not pk {}.", either(&elsewhere), cmd.name);
    }
    let hint = if names.is_empty() {
        format!("pk {} takes no options.", cmd.name)
    } else {
        format!("{}Options: {}.", did_you_mean(guess.as_deref()), names.join(", "))
    };
    CliError::usage("unknown_option", message).hint(hint)
}

/// Check one option value, returning it in canonical form.
fn check_value(o: &Opt, raw: &str) -> Result<String> {
    let bad = |why: String| CliError::usage("invalid_value", format!("--{} {why}", o.name));
    let v = raw.trim();
    if v.is_empty() {
        return Err(bad("needs a value.".into()).hint(format!("Write --{} {}.", o.name, o.value_hint().unwrap_or_default())));
    }
    match o.kind {
        Kind::Switch => unreachable!("switches take no value"),
        Kind::Text(_) | Kind::Files => Ok(v.to_string()),
        Kind::Choice(choices) => match choices.iter().find(|c| c.eq_ignore_ascii_case(v)) {
            Some(c) => Ok(c.to_string()),
            None => Err(bad(format!("must be {}; got \"{v}\".", either(choices))).hint(did_you_mean(suggest(v, choices.iter().copied())).trim_end().to_string())),
        },
        Kind::Number(range) => {
            let n = numbers::parse(v);
            if !n.is_finite() {
                return Err(bad(format!("must be a number; got \"{v}\".")));
            }
            match range.check(n) {
                Some(why) => Err(bad(format!("must be {why}; got {v}."))),
                None => Ok(v.to_string()),
            }
        }
        Kind::Count(min) => match v.parse::<u64>() {
            Ok(n) if n >= min => Ok(v.to_string()),
            _ => Err(bad(format!("must be a whole number of at least {min}; got \"{v}\"."))),
        },
        Kind::Pairs => {
            for item in v.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                let Some((name, value)) = item.split_once('=').filter(|(n, x)| !n.trim().is_empty() && !x.trim().is_empty()) else {
                    return Err(bad(format!("expects name=value; got \"{item}\".")).hint(format!("For example --{} Vcmax25=90.", o.name)));
                };
                if !numbers::parse(value).is_finite() {
                    return Err(bad(format!("{}: \"{}\" is not a number.", name.trim(), value.trim())));
                }
            }
            Ok(v.to_string())
        }
        Kind::Names => Ok(v.to_string()),
    }
}

/// Parse a command's arguments against its table entry. Every problem is a usage error
/// that names what was wrong and what would be right.
pub fn parse(cmd: &Command, argv: &[String]) -> Result<ParsedArgs> {
    let mut out = ParsedArgs::default();
    let mut i = 0;
    let mut only_positional = false;
    while i < argv.len() {
        let a = &argv[i];
        i += 1;
        if only_positional || a == "-" || !a.starts_with('-') || looks_like_number(a) {
            out.positional.push(a.clone());
            continue;
        }
        if a == "--" {
            only_positional = true;
            continue;
        }
        if !a.starts_with("--") {
            let name = a.trim_start_matches('-');
            let guess = cmd.options.iter().find(|o| o.name == name).map(|o| o.name).or_else(|| suggest(name, cmd.options.iter().map(|o| o.name)));
            let hint = match guess {
                Some(g) => format!("Options start with two dashes: --{g}."),
                None => format!("Options start with two dashes. Run \"pk {} --help\" for the options.", cmd.name),
            };
            return Err(CliError::usage("unknown_option", format!("Unknown option {a} for pk {}.", cmd.name)).hint(hint));
        }
        let body = &a[2..];
        let (name, inline) = match body.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (body, None),
        };
        let (o, negated) = match cmd.options.iter().find(|o| o.name == name) {
            Some(o) => (o, false),
            None => match name.strip_prefix("no-").and_then(|n| cmd.options.iter().find(|o| o.name == n)) {
                Some(o) if matches!(o.kind, Kind::Switch) && inline.is_none() => (o, true),
                Some(o) => {
                    return Err(CliError::usage("invalid_value", format!("--no-{} isn't valid: --{} is not an on/off option.", o.name, o.name)).hint(format!("Leave --{} out instead.", o.name)))
                }
                None => return Err(unknown_option(cmd, &format!("--{name}"), name)),
            },
        };
        let value = if let Kind::Switch = o.kind {
            let on = match inline {
                Some(v) => parse_switch(v).ok_or_else(|| {
                    CliError::usage("invalid_value", format!("--{} is an on/off option; got \"{v}\".", o.name)).hint(format!("Write --{} or --no-{}.", o.name, o.name))
                })?,
                None => {
                    // `--rows true` reads naturally; accept it rather than treat `true` as a file.
                    let explicit = argv.get(i).and_then(|n| ["true", "false"].contains(&n.to_lowercase().as_str()).then(|| parse_switch(n)).flatten());
                    if explicit.is_some() {
                        i += 1;
                    }
                    explicit.unwrap_or(true) != negated
                }
            };
            Flag::Bool(on)
        } else {
            let raw = match inline {
                Some(v) => v.to_string(),
                None => match argv.get(i) {
                    Some(next) if !next.starts_with("--") => {
                        i += 1;
                        next.clone()
                    }
                    _ => {
                        return Err(CliError::usage("missing_value", format!("--{} needs a value.", o.name))
                            .hint(format!("Write --{} {}{}.", o.name, o.value_hint().unwrap_or_default(), o.default.map_or(String::new(), |d| format!(" (default {d})")))))
                    }
                },
            };
            Flag::Str(check_value(o, &raw)?)
        };
        match (out.flags.get_mut(o.name), value) {
            (None, Flag::Str(s)) if o.repeatable() => {
                out.flags.insert(o.name.to_string(), Flag::List(vec![s]));
            }
            (None, v) => {
                out.flags.insert(o.name.to_string(), v);
            }
            (Some(Flag::List(l)), Flag::Str(s)) => l.push(s),
            (Some(prev), v) => {
                let show = |f: &Flag| match f {
                    Flag::Str(s) => s.clone(),
                    Flag::Bool(b) => b.to_string(),
                    Flag::List(l) => l.join(","),
                };
                if show(prev) != show(&v) {
                    return Err(CliError::usage("duplicate_option", format!("--{} was given twice ({} and {}).", o.name, show(prev), show(&v))).hint(format!("Give --{} once.", o.name)));
                }
            }
        }
    }
    if let Some(check) = cmd.check_args {
        check(&out.positional)?;
    }
    check_positionals(cmd, &out.positional)?;
    Ok(out)
}

fn check_positionals(cmd: &Command, got: &[String]) -> Result<()> {
    let required = cmd.args.iter().filter(|a| a.required).count();
    if got.len() < required {
        let missing = &cmd.args[got.len()];
        return Err(CliError::usage("missing_argument", format!("pk {} needs a {}: {}.", cmd.name, missing.name, lower_first(missing.help))).hint(format!("For example: {}", cmd.example)));
    }
    let open_ended = cmd.args.last().is_some_and(|a| a.many);
    if !open_ended && got.len() > cmd.args.len() {
        let extra = &got[cmd.args.len()..];
        let what = match cmd.args {
            [] => "no arguments".to_string(),
            [one] => format!("one {}", one.name),
            many => format!("{} arguments", many.len()),
        };
        let mut hint = format!("Run pk {} once per file.", cmd.name);
        if let Some(o) = extra.iter().find_map(|e| cmd.options.iter().find(|o| o.name == e.trim_start_matches('-'))) {
            hint = format!("To set an option, write --{}.", o.name);
        } else if cmd.name == "describe" {
            hint = "Describe one model at a time.".into();
        }
        return Err(CliError::usage("too_many_arguments", format!("pk {} takes {what}; also got {}.", cmd.name, extra.iter().map(|e| format!("\"{e}\"")).collect::<Vec<_>>().join(", "))).hint(hint));
    }
    Ok(())
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_lowercase().collect::<String>() + c.as_str())
}

// ---- Help and description ----

pub fn usage_line(cmd: &Command) -> String {
    let mut s = format!("pk {}", cmd.name);
    for a in cmd.args {
        let name = if a.many { format!("{}...", a.name) } else { a.name.to_string() };
        s += &if a.required { format!(" <{name}>") } else { format!(" [{name}]") };
    }
    if !cmd.options.is_empty() {
        s += " [options]";
    }
    s
}

pub fn command_help(cmd: &Command) -> String {
    let mut s = format!("{}\n\n{}.\n", usage_line(cmd), cmd.summary);
    if !cmd.args.is_empty() {
        s += "\nArguments\n";
        for a in cmd.args {
            s += &format!("  {:<28}{}\n", format!("<{}>", a.name), a.help);
        }
    }
    if !cmd.options.is_empty() {
        s += "\nOptions\n";
        for o in cmd.options {
            let left = format!("--{}{}", o.name, o.value_hint().map_or(String::new(), |h| format!(" {h}")));
            let mut right = o.help.to_string();
            if let Some(d) = o.default {
                right += &format!(" (default {d})");
            }
            if o.repeatable() {
                right += " [repeatable]";
            }
            s += &format!("  {left:<28}{right}\n");
        }
    }
    s + &format!("\nExample\n  {}\n", cmd.example)
}

pub fn overview() -> String {
    let mut s = String::from("pk — PhytKit: plant physiology model fitting from the command line\n\nCommands\n");
    for c in COMMANDS {
        s += &format!("  {:<40}{}\n", usage_line(c), c.summary);
    }
    s + "
Inputs may be .csv, .xlsx (first sheet), LI-6800 logs (no extension), or - for stdin.
Output is one JSON object. Errors print {\"ok\": false, \"error\": {\"code\", \"message\", \"hint\", ...}}
and exit with code 1.
Run \"pk <command> --help\" for a command's options, or \"pk describe\" for every model.
"
}

/// The `commands` section of `pk describe`.
pub fn describe_commands() -> Value {
    Value::Array(
        COMMANDS
            .iter()
            .map(|c| {
                let args: Vec<Value> = c.args.iter().map(|a| json!({ "name": a.name, "required": a.required, "many": a.many, "description": a.help })).collect();
                let options: Vec<Value> = c
                    .options
                    .iter()
                    .map(|o| {
                        let (ty, choices) = match o.kind {
                            Kind::Switch => ("switch", None),
                            Kind::Text(_) => ("text", None),
                            Kind::Choice(ch) => ("choice", Some(ch)),
                            Kind::Number(_) => ("number", None),
                            Kind::Count(_) => ("integer", None),
                            Kind::Pairs => ("name=value", None),
                            Kind::Names => ("names", None),
                            Kind::Files => ("files", None),
                        };
                        json!({ "name": format!("--{}", o.name), "type": ty, "choices": choices, "default": o.default, "repeatable": o.repeatable(), "description": o.help })
                    })
                    .collect();
                json!({ "name": c.name, "usage": usage_line(c), "summary": c.summary, "arguments": args, "options": options, "example": c.example })
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }
    fn err(cmd: &str, line: &str) -> CliError {
        parse(command(cmd).unwrap(), &argv(line)).expect_err(line)
    }

    #[test]
    fn suggestions() {
        assert_eq!(suggest("sidednes", ["sidedness", "rows"]), Some("sidedness"));
        assert_eq!(suggest("modle", ["model", "fit"]), Some("model"));
        assert_eq!(suggest("c4", ["fvcb", "fvcbC4"]), Some("fvcbC4"));
        assert_eq!(suggest("Vcmax", ["Vcmax25", "Jmax25"]), Some("Vcmax25"));
        assert_eq!(suggest("zzz", ["fvcb", "medlyn"]), None);
    }

    #[test]
    fn rejects_unknown_and_misspelled_options() {
        let e = err("li600 correct", "a.csv --sidednes 2");
        assert_eq!(e.code, "unknown_option");
        assert!(e.hint.unwrap().starts_with("Did you mean --sidedness?"));
        let e = err("fit", "photosynthesis a.csv --rows");
        assert!(e.message.contains("option of pk li600 correct or li600 estimate"), "{}", e.message);
        assert_eq!(err("li600 correct", "a.csv -rows").hint.unwrap(), "Options start with two dashes: --rows.");
    }

    #[test]
    fn checks_values_and_counts() {
        assert_eq!(err("fit", "soil a.csv --max-iter 2.5").code, "invalid_value");
        assert_eq!(err("li600 estimate", "a.csv --ca -10").code, "invalid_value");
        assert_eq!(err("li600 correct", "a.csv --format xml").code, "invalid_value");
        assert_eq!(err("fit", "soil a.csv --set gm").code, "invalid_value");
        assert_eq!(err("li600 correct", "a.csv --sidedness").code, "missing_value");
        assert_eq!(err("fit", "soil a.csv --model x --model y").code, "duplicate_option");
        assert_eq!(err("li600 correct", "a.csv b.csv").code, "too_many_arguments");
        assert_eq!(err("li600 correct", "").code, "missing_argument");
    }

    #[test]
    fn switches_do_not_swallow_arguments() {
        let p = parse(command("li600 correct").unwrap(), &argv("--rows a.csv")).unwrap();
        assert_eq!(p.positional, ["a.csv"]);
        assert!(p.bool("rows", false));
        let p = parse(command("li600 correct").unwrap(), &argv("a.csv --rows false")).unwrap();
        assert!(!p.bool("rows", true));
        let p = parse(command("li600 correct").unwrap(), &argv("a.csv --no-rows")).unwrap();
        assert!(!p.bool("rows", true));
    }

    #[test]
    fn values_can_be_negative_numbers_and_choices_ignore_case() {
        let p = parse(command("fit").unwrap(), &argv("soil a.csv --set psi0=-1.5 --units kpa --format=CSV")).unwrap();
        assert_eq!(p.str("units").unwrap().as_deref(), Some("kPa"));
        assert_eq!(p.str("format").unwrap().as_deref(), Some("csv"));
    }
}

#[cfg(test)]
mod error_code_tests {
    /// Every code used in the source is documented in `ERROR_CODES`.
    #[test]
    fn every_code_is_listed() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        for entry in std::fs::read_dir(dir).unwrap() {
            let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
            for marker in ["usage(\"", "failed(\""] {
                for part in text.split(marker).skip(1) {
                    let code = &part[..part.find('"').unwrap()];
                    if code.chars().all(|c| c.is_ascii_lowercase() || c == '_') && !code.is_empty() {
                        assert!(crate::args::CliError::is_known_code(code), "{code} is not in ERROR_CODES");
                    }
                }
            }
        }
    }
}
