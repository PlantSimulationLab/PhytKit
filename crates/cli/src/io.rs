//! Read CSV, .xlsx (first sheet) or an LI-6800 native log.

use crate::args::CliError;
use phytkit_core::licor::{licor_text_to_csv, xlsx_to_csv};
use std::io::Read;

pub struct LoadedFile {
    pub name: String,
    pub text: String,
    pub converted: Option<&'static str>,
}

fn cwd_hint() -> String {
    std::env::current_dir().map_or(String::new(), |d| {
        format!("Relative paths are read from {}.", d.display())
    })
}

/// Read a file, with an error that says what went wrong in plain terms.
pub fn read_bytes(path: &str) -> Result<Vec<u8>, CliError> {
    let p = std::path::Path::new(path);
    if p.is_dir() {
        return Err(
            CliError::failed("not_a_file", format!("{path} is a folder, not a file.")).hint(
                format!(
                    "Pass the files inside it, for example {}/*.csv.",
                    path.trim_end_matches('/')
                ),
            ),
        );
    }
    std::fs::read(p).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => {
            CliError::failed("file_not_found", format!("File not found: {path}")).hint(cwd_hint())
        }
        std::io::ErrorKind::PermissionDenied => CliError::failed(
            "permission_denied",
            format!("Permission denied reading {path}."),
        ),
        _ => CliError::failed("read_failed", format!("Could not read {path}: {e}.")),
    })
}

fn basename(p: &str) -> String {
    p.rsplit('/').next().unwrap_or(p).to_string()
}

pub fn load_file(path: &str) -> Result<LoadedFile, CliError> {
    if path == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).map_err(|e| {
            CliError::failed(
                "read_failed",
                format!("Could not read standard input: {e}."),
            )
        })?;
        let raw = String::from_utf8_lossy(&buf).into_owned();
        let li = licor_text_to_csv(&raw);
        let converted = li.as_ref().map(|_| "li6800-log");
        return Ok(LoadedFile {
            name: "stdin".into(),
            text: li.unwrap_or(raw),
            converted,
        });
    }
    if path.to_lowercase().ends_with(".xlsx") {
        let text = xlsx_to_csv(&read_bytes(path)?).map_err(CliError::data)?;
        return Ok(LoadedFile {
            name: basename(path),
            text,
            converted: Some("xlsx"),
        });
    }
    let buf = read_bytes(path)?;
    let raw = String::from_utf8_lossy(&buf).into_owned();
    let li = licor_text_to_csv(&raw);
    let converted = li.as_ref().map(|_| "li6800-log");
    Ok(LoadedFile {
        name: basename(path),
        text: li.unwrap_or(raw),
        converted,
    })
}

pub fn write_out(path: &str, text: &str) -> Result<(), CliError> {
    let body = if text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    };
    if path == "-" {
        print!("{body}");
        Ok(())
    } else {
        std::fs::write(path, body).map_err(|e| {
            let hint = match e.kind() {
                std::io::ErrorKind::NotFound => {
                    "The folder it would go in doesn't exist.".to_string()
                }
                std::io::ErrorKind::PermissionDenied => {
                    "Choose a location you can write to.".to_string()
                }
                _ => String::new(),
            };
            CliError::failed("write_failed", format!("Could not write {path}: {e}.")).hint(hint)
        })
    }
}
