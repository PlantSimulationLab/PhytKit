//! `pk concat <files...>`: merge LI-6800 logs or CSVs under one header with a CurveID per file.

use crate::args::{ParsedArgs, Result};
use crate::io::{load_file, write_out};
use crate::{object, Output};
use phytkit_core::concat::concat_csvs;
use serde_json::{json, Value};

pub fn concat(args: &ParsedArgs) -> Result<Output> {
    let files = args
        .positional
        .iter()
        .map(|p| load_file(p))
        .collect::<Result<Vec<_>>>()?;
    let res = concat_csvs(
        &files
            .iter()
            .map(|f| (f.name.as_str(), f.text.as_str()))
            .collect::<Vec<_>>(),
    );
    let out = args.str("out")?;
    if let Some(o) = &out {
        write_out(o, &res.csv)?;
    }
    let format = args.str("format")?.unwrap_or_else(|| {
        if out.is_some() {
            "json".into()
        } else {
            "csv".into()
        }
    });
    if format == "csv" {
        return Ok(Output::Raw(res.csv));
    }
    let files: Vec<Value> = files
        .iter()
        .map(|f| json!({ "name": f.name, "converted": f.converted }))
        .collect();
    let rows = res.csv.trim().lines().count().saturating_sub(1);
    Ok(Output::Json(object(json!({
        "files": files, "curves": res.curves, "namespaced": res.namespaced, "warnings": res.warnings, "out": out, "rows": rows
    }))))
}
