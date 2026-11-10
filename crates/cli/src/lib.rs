//! The `pk` command layer, shared by the `pk` binary and the Python bindings.

pub mod agent_files;
pub mod args;
mod concat;
mod describe;
mod fit;
pub use fit::fit_warnings;
mod hydraulics_adapter;
mod io;
mod li600;
pub mod options;
mod tools;

pub use args::CliError as Error;
use args::{CliError, Result};
use options::{command, command_help, did_you_mean, overview, suggest, usage_line, COMMANDS};
use serde_json::{json, Map, Value};

/// A JSON object, in insertion order.
pub type Obj = Map<String, Value>;

pub enum Output {
    /// Printed as-is (help text, CSV).
    Raw(String),
    /// Printed as `{ ok: true, command, ...obj }`.
    Json(Obj),
}

fn unknown_command(written: &str) -> CliError {
    let names: Vec<&str> = COMMANDS.iter().map(|c| c.name).collect();
    if written == "li600" || written.starts_with("li600 ") {
        let sub = written.trim_start_matches("li600").trim();
        let subs = ["correct", "estimate"];
        let (message, guess) = if sub.is_empty() {
            (
                "pk li600 needs a subcommand: correct or estimate.".to_string(),
                None,
            )
        } else {
            (
                format!("Unknown command pk li600 {sub}."),
                suggest(sub, subs).map(|s| format!("pk li600 {s}")),
            )
        };
        return CliError::usage("unknown_command", message).hint(format!(
            "{}Commands: pk li600 correct (fix gsw), pk li600 estimate (solve A).",
            did_you_mean(guess.as_deref())
        ));
    }
    if written == "skills" || written.starts_with("skills ") {
        let sub = written.trim_start_matches("skills").trim();
        let message = if sub.is_empty() {
            "pk skills needs a subcommand: path or install.".to_string()
        } else {
            format!("Unknown command pk skills {sub}.")
        };
        let guess = suggest(sub, ["path", "install"]).map(|s| format!("pk skills {s}"));
        return CliError::usage("unknown_command", message).hint(format!(
            "{}Commands: pk skills path (where the agent skill is), pk skills install (copy it into .claude/skills).",
            did_you_mean(guess.as_deref())
        ));
    }
    let guess = suggest(written, names.iter().copied()).map(|s| format!("pk {s}"));
    CliError::usage("unknown_command", format!("Unknown command \"{written}\".")).hint(format!(
        "{}Commands: {}. Run \"pk --help\" for an overview.",
        did_you_mean(guess.as_deref()),
        names
            .iter()
            .map(|n| format!("pk {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

/// Run a command line (without the program name).
pub fn run(argv: &[String]) -> Result<Output> {
    let Some(first) = argv.first() else {
        return Ok(Output::Raw(overview()));
    };
    if ["--help", "-h", "help"].contains(&first.as_str()) {
        // `pk help fit` is the same as `pk fit --help`.
        return match argv.get(1..).filter(|r| !r.is_empty()) {
            Some(rest) => run(&[rest, &["--help".to_string()]].concat()),
            None => Ok(Output::Raw(overview())),
        };
    }
    let key = command_name(argv).unwrap_or_default();
    let tail = &argv[key.split(' ').count().min(argv.len())..];
    let cmd = command(&key).ok_or_else(|| unknown_command(&key))?;
    if tail.iter().any(|a| a == "--help" || a == "-h") {
        return Ok(Output::Raw(command_help(cmd)));
    }
    let args = options::parse(cmd, tail)?;
    match cmd.name {
        "describe" => describe::describe(&args).map(Output::Json),
        "li600 correct" => li600::correct(&args),
        "li600 estimate" => li600::estimate(&args),
        "fit" => fit::fit_command(&args),
        "concat" => concat::concat(&args),
        "agent-files" => agent_files::agent_files(&args),
        "skills path" => agent_files::skills_path(&args),
        "skills install" => agent_files::skills_install(&args),
        _ => unreachable!("every command in the table is dispatched"),
    }
}

/// The command name used in output (`"li600 correct"` for `li600 correct ...`).
pub fn command_name(argv: &[String]) -> Option<String> {
    match argv.first().map(String::as_str) {
        Some(group @ ("li600" | "skills")) => Some(match argv.get(1) {
            Some(sub) if !sub.starts_with('-') => format!("{group} {sub}"),
            _ => group.to_string(),
        }),
        other => other.map(str::to_string),
    }
}

/// What `pk` prints: the text and the exit code.
pub fn render(argv: &[String], result: Result<Output>) -> (String, i32) {
    let cmd = command_name(argv);
    let (value, code) = match result {
        Ok(Output::Raw(raw)) => return (if raw.ends_with('\n') { raw } else { raw + "\n" }, 0),
        Ok(Output::Json(obj)) => (Value::Object(envelope(cmd.as_deref(), obj)), 0),
        Err(e) => (
            json!({ "ok": false, "command": cmd, "error": error_json(cmd.as_deref(), &e) }),
            1,
        ),
    };
    (
        serde_json::to_string_pretty(&value).unwrap_or_default() + "\n",
        code,
    )
}

/// The `error` object: `code`, `message`, `usage`, and `hint` and `help` when there are any.
pub fn error_json(cmd: Option<&str>, e: &CliError) -> Value {
    let mut o = object(json!({ "code": e.code, "message": e.message, "usage": e.usage }));
    if let Some(h) = &e.hint {
        o.insert("hint".into(), json!(h));
    }
    if let Some(c) = cmd.and_then(command).filter(|_| e.usage) {
        o.insert("help".into(), json!(usage_line(c)));
    }
    Value::Object(o)
}

/// A `json!({...})` object literal as an `Obj`.
pub fn object(v: Value) -> Obj {
    match v {
        Value::Object(o) => o,
        _ => Obj::new(),
    }
}

/// `{ "ok": true, "command": ..., ...obj }`.
pub fn envelope(cmd: Option<&str>, obj: Obj) -> Obj {
    let mut out = Obj::new();
    out.insert("ok".into(), Value::Bool(true));
    out.insert("command".into(), json!(cmd));
    out.extend(obj);
    out
}
