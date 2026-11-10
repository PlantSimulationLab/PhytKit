//! `pk fit <tool> <file...>`.

use crate::args::{CliError, ParsedArgs, Result};
use crate::io::{load_file, LoadedFile};
use crate::options::{did_you_mean, suggest, unknown_constraint, unknown_parameter};
use crate::tools::{tool_by_name, FitData, Parser, Tool, ToolModel, TOOLS};
use crate::{object, Obj, Output};
use phytkit_core::model::{Model, Params};
use phytkit_core::models::model_by_name;
use phytkit_core::numbers::{self, json as num};
use phytkit_core::rescale::{
    predict_at_ref, rescale_factor, saturating_ci, survey_conditions, Stat,
};
use phytkit_core::warn::Warning;
use phytkit_core::{fit, FitOptions, FitResult};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;

const DATA_HINT: &str = "Run \"pk describe <model>\" to see the inputs each model reads.";

/// A model name the tool doesn't have, with the nearest real one.
fn unknown_model(tool: &Tool, name: &str) -> CliError {
    let names: Vec<&str> = tool.models.iter().map(|x| x.model).collect();
    let mut hint = did_you_mean(suggest(name, names.iter().copied()));
    if let Some(other) = TOOLS
        .iter()
        .find(|t| t.name != tool.name && t.models.iter().any(|x| x.model == name))
    {
        hint = format!(
            "{name} is a {} model; run pk fit {} instead. ",
            other.name, other.name
        );
    }
    CliError::usage(
        "unknown_model",
        format!("Unknown model \"{name}\" for pk fit {}.", tool.name),
    )
    .hint(format!("{hint}Models: {}.", names.join(", ")))
}

fn basename(p: &str) -> String {
    p.rsplit('/').next().unwrap_or(p).to_string()
}

/// One thing to fit: a file, or a group split out of one file.
struct Source {
    name: String,
    path: String,
    text: Option<String>,
}

fn load_photosynthesis(path: &str) -> Result<(String, FitData, Option<&'static str>)> {
    let data_error = |e: String| CliError::data(e).hint(DATA_HINT);
    if path.to_lowercase().ends_with(".xlsx") {
        let buf = crate::io::read_bytes(path)?;
        return Ok((
            basename(path),
            crate::tools::parse_licor_xlsx_data(&buf).map_err(data_error)?,
            Some("xlsx"),
        ));
    }
    let f = load_file(path)?;
    Ok((
        f.name,
        crate::tools::parse_with(Parser::Photosynthesis, &f.text).map_err(data_error)?,
        f.converted,
    ))
}

struct Loaded {
    name: String,
    data: FitData,
    entry: &'static ToolModel,
    converted: Option<&'static str>,
}

/// The outer error stops the whole command (a bad model name); the inner one fails only this source.
fn load_for(tool: &'static Tool, src: &Source, model_name: Option<&str>) -> Result<Result<Loaded>> {
    let unknown = |m: &str| unknown_model(tool, m);
    let data_error = |e: String| CliError::data(e).hint(DATA_HINT);
    if tool.photosynthesis {
        let entry = match model_name {
            Some(m) => tool
                .models
                .iter()
                .find(|x| x.model == m)
                .ok_or_else(|| unknown(m))?,
            None => &tool.models[0],
        };
        if let Some(text) = &src.text {
            return Ok(crate::tools::parse_with(Parser::Photosynthesis, text)
                .map(|data| Loaded {
                    name: src.name.clone(),
                    data,
                    entry,
                    converted: None,
                })
                .map_err(data_error));
        }
        return Ok(
            load_photosynthesis(&src.path).map(|(name, data, converted)| Loaded {
                name,
                data,
                entry,
                converted,
            }),
        );
    }
    let f: LoadedFile = match &src.text {
        Some(t) => LoadedFile {
            name: src.name.clone(),
            text: t.clone(),
            converted: None,
        },
        None => match load_file(&src.path) {
            Ok(f) => f,
            Err(e) => return Ok(Err(e)),
        },
    };
    if let Some(m) = model_name {
        let entry = tool
            .models
            .iter()
            .find(|x| x.model == m)
            .ok_or_else(|| unknown(m))?;
        return Ok(crate::tools::parse_with(entry.parser, &f.text)
            .map(|data| Loaded {
                name: f.name.clone(),
                data,
                entry,
                converted: f.converted,
            })
            .map_err(data_error));
    }
    let mut errors = Vec::new();
    for entry in tool.models {
        match crate::tools::parse_with(entry.parser, &f.text) {
            Ok(data) => {
                return Ok(Ok(Loaded {
                    name: f.name.clone(),
                    data,
                    entry,
                    converted: f.converted,
                }))
            }
            Err(e) => errors.push(format!("{}: {}", entry.model, e)),
        }
    }
    Ok(Err(CliError::data(format!(
        "No {} model could read {}:\n{}",
        tool.name,
        f.name,
        errors.join("\n")
    ))
    .hint(DATA_HINT)))
}

struct FitSpec {
    fit_params: Vec<String>,
    init: Params,
    constraints: Option<Vec<String>>,
    weights: Option<HashMap<String, f64>>,
    max_iter: Option<usize>,
}

fn build_spec(
    entry: &ToolModel,
    m: &dyn Model,
    data: &FitData,
    args: &ParsedArgs,
) -> Result<FitSpec> {
    let names: Vec<&str> = m.params().iter().map(|p| p.name).collect();
    let check = |k: &str, flag: &str| -> Result<()> {
        if names.contains(&k) {
            Ok(())
        } else {
            Err(unknown_parameter(&format!("--{flag}"), k, m.name(), &names))
        }
    };
    let only = args.list("fit");
    let fixed = args.list("fix");
    for k in &only {
        check(k, "fit")?;
    }
    for k in &fixed {
        check(k, "fix")?;
    }
    let mut fit_params: Vec<String> = if !only.is_empty() {
        only.clone()
    } else {
        m.params()
            .iter()
            .filter(|p| !p.fixed)
            .map(|p| p.name.to_string())
            .collect()
    };
    fit_params.retain(|k| !fixed.contains(k));
    if fit_params.is_empty() {
        return Err(CliError::usage(
            "nothing_to_fit",
            "Nothing left to fit: every parameter is fixed.",
        )
        .hint("Remove a name from --fix, or list the parameters to fit with --fit."));
    }
    let mut init: Params = m
        .params()
        .iter()
        .map(|p| (p.name.to_string(), p.default))
        .collect();
    if let Some(seed) = entry.seed {
        for (k, v) in seed(data) {
            init.insert(k, v);
        }
    }
    let set = args.kv("set")?;
    for (k, v) in &set {
        check(k, "set")?;
        if fit_params.contains(k) {
            if let Some(e) =
                crate::options::start_out_of_bounds(m, "--set", k, *v, &format!("add --fix {k}"))
            {
                return Err(e);
            }
        }
    }
    for (k, v) in set {
        init.insert(k, v);
    }
    let cons = m.constraints();
    let (mut constraints, mut weights) = (None, None);
    if cons.is_empty() {
        if let Some(flag) = ["no-constraint", "constraint", "weight", "constraints"]
            .into_iter()
            .find(|f| args.flags.contains_key(*f))
        {
            return Err(CliError::usage(
                "not_applicable",
                format!("--{flag}: {} has no soft constraints.", m.name()),
            )
            .hint(format!("Leave --{flag} out.")));
        }
    } else {
        let off = args.list("no-constraint");
        let extra = args.list("constraint");
        let all = args.str("constraints")?;
        let keys: Vec<&str> = cons.iter().map(|c| c.key).collect();
        for (flag, list) in [("--no-constraint", &off), ("--constraint", &extra)] {
            for k in list {
                if !keys.contains(&k.as_str()) {
                    return Err(unknown_constraint(flag, k, m.name(), &keys));
                }
            }
        }
        if let Some(k) = extra.iter().find(|k| off.contains(k)) {
            return Err(CliError::usage(
                "invalid_combination",
                format!("--constraint {k} and --no-constraint {k} contradict each other."),
            )
            .hint("Give the constraint to only one of them."));
        }
        let none = all.as_deref() == Some("none");
        constraints = Some(
            cons.iter()
                .filter(|c| {
                    let k = c.key.to_string();
                    extra.contains(&k) || (!none && c.default_on && !off.contains(&k))
                })
                .map(|c| c.key.to_string())
                .collect(),
        );
        let w = args.kv("weight")?;
        for (k, _) in &w {
            if !keys.contains(&k.as_str()) {
                return Err(unknown_constraint("--weight", k, m.name(), &keys));
            }
        }
        weights = Some(
            cons.iter()
                .map(|c| {
                    (
                        c.key.to_string(),
                        w.iter()
                            .find(|(k, _)| k == c.key)
                            .map_or(c.weight, |(_, v)| *v),
                    )
                })
                .collect(),
        );
    }
    let max_iter = args.str("max-iter")?.and_then(|s| s.parse::<usize>().ok());
    Ok(FitSpec {
        fit_params,
        init,
        constraints,
        weights,
        max_iter,
    })
}

fn run_fit(m: &dyn Model, data: &FitData, spec: &FitSpec) -> FitResult {
    fit(
        m,
        &data.x,
        &data.y,
        &FitOptions {
            fit_params: Some(spec.fit_params.clone()),
            init: spec.init.clone(),
            max_iter: spec.max_iter,
            curves: data.curves.clone(),
            constraints: spec.constraints.clone(),
            constraint_weights: spec.weights.clone(),
            ..Default::default()
        },
    )
}

fn params_json(p: &Params) -> Value {
    Value::Object(p.iter().map(|(k, v)| (k.clone(), num(*v))).collect())
}

struct Summary {
    obj: Obj,
    file: String,
    model: String,
    n: usize,
    r2: f64,
    rmse: f64,
    iterations: usize,
    converged: bool,
    elapsed: f64,
    rescale_factor: Option<f64>,
    params: Params,
    derived: Option<Params>,
}

#[allow(clippy::too_many_arguments)]
fn summarize(
    name: &str,
    tool: &str,
    m: &dyn Model,
    data: &FitData,
    r: &FitResult,
    converted: Option<&str>,
    predicted: bool,
    rescale: Option<Obj>,
    rescale_factor: Option<f64>,
    ms: f64,
) -> Summary {
    let rounded: Params = r
        .params
        .iter()
        .map(|(k, v)| (k.clone(), numbers::round_fit(*v)))
        .collect();
    let derived: Option<Params> = m.derived(&r.params).map(|d| {
        d.into_iter()
            .map(|(k, v)| (k, numbers::round_fit(v)))
            .collect()
    });
    let mut o = object(json!({
        "file": name,
        "converted": converted,
        "tool": tool,
        "model": m.name(),
        "n": data.y.len(),
        "mapping": data.mapping.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<Obj>(),
        "r2": num(numbers::round_fit(r.r2)),
        "rmse": num(numbers::round_fit(r.rmse)),
        "iterations": r.iterations,
        "converged": r.converged,
        "fitted": r.fitted,
        "params": params_json(&rounded),
    }));
    if let Some(d) = &derived {
        o.insert("derived".into(), params_json(d));
    }
    o.insert("rescale".into(), rescale.map_or(Value::Null, Value::Object));
    let mut warnings = data.warnings.clone();
    warnings.extend(fit_warnings(m, r));
    o.insert("warnings".into(), phytkit_core::warn::to_json(&warnings));
    o.insert("elapsedMs".into(), json!(ms.round() as u64));
    if predicted {
        o.insert(
            "predicted".into(),
            Value::Array(
                r.predicted
                    .iter()
                    .map(|v| num(numbers::round_to(*v, 5)))
                    .collect(),
            ),
        );
        o.insert(
            "observed".into(),
            Value::Array(data.y.iter().map(|v| num(*v)).collect()),
        );
    }
    Summary {
        obj: o,
        file: name.to_string(),
        model: m.name().to_string(),
        n: data.y.len(),
        r2: numbers::round_fit(r.r2),
        rmse: numbers::round_fit(r.rmse),
        iterations: r.iterations,
        converged: r.converged,
        elapsed: ms.round(),
        rescale_factor,
        params: rounded,
        derived,
    }
}

/// A CSV cell, quoted when it holds a quote, comma or newline.
fn cell(s: String) -> String {
    if s.contains(['"', ',', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}
fn num_cell(v: f64) -> String {
    cell(numbers::text(v))
}

/// The batch table. Its parameter and derived columns cover every model in the batch (in the
/// order the models first appear), and a `model` column is added when the files were fitted
/// with more than one model, so no file's results are dropped.
fn batch_csv(rows: &[Summary], errors: &[(String, CliError)], entry: &ToolModel) -> String {
    let mut models: Vec<String> = Vec::new();
    for r in rows {
        if !models.contains(&r.model) {
            models.push(r.model.clone());
        }
    }
    if models.is_empty() {
        models.push(entry.model.to_string());
    }
    let mixed = models.len() > 1;
    let (mut keys, mut dkeys): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for name in &models {
        let m = model_by_name(name).expect("registered model");
        for p in m.params() {
            if !keys.iter().any(|k| k == p.name) {
                keys.push(p.name.to_string());
            }
        }
        let defaults: Params = m
            .params()
            .iter()
            .map(|p| (p.name.to_string(), p.default))
            .collect();
        for k in m
            .derived(&defaults)
            .map(|d| d.into_keys().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            if !dkeys.contains(&k) {
                dkeys.push(k);
            }
        }
    }
    let mut head: Vec<String> = [
        "file",
        "n_points",
        "r2",
        "rmse",
        "iterations",
        "converged",
        "elapsed_ms",
        "rescale_factor",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    if mixed {
        head.insert(1, "model".into());
    }
    head.extend(keys.iter().cloned());
    head.extend(dkeys.iter().cloned());
    head.push("error".into());
    let mut lines = vec![head.join(",")];
    for r in rows {
        let mut c = vec![
            cell(r.file.clone()),
            num_cell(r.n as f64),
            num_cell(r.r2),
            num_cell(r.rmse),
            num_cell(r.iterations as f64),
            cell(r.converged.to_string()),
            num_cell(r.elapsed),
            r.rescale_factor.map_or(String::new(), num_cell),
        ];
        if mixed {
            c.insert(1, cell(r.model.clone()));
        }
        c.extend(
            keys.iter()
                .map(|k| r.params.get(k).map_or(String::new(), |v| num_cell(*v))),
        );
        c.extend(dkeys.iter().map(|k| {
            r.derived
                .as_ref()
                .and_then(|d| d.get(k))
                .map_or(String::new(), |v| num_cell(*v))
        }));
        c.push(String::new());
        lines.push(c.join(","));
    }
    for (file, err) in errors {
        let mut c = vec![cell(file.clone())];
        c.extend(std::iter::repeat_n(
            String::new(),
            7 + usize::from(mixed) + keys.len() + dkeys.len(),
        ));
        c.push(cell(err.message.clone()));
        lines.push(c.join(","));
    }
    lines.join("\n") + "\n"
}

fn find_tool(name: &str) -> Result<&'static Tool> {
    tool_by_name(name).ok_or_else(|| {
        let names: Vec<&str> = TOOLS.iter().map(|t| t.name).collect();
        let mut hint = did_you_mean(suggest(name, names.iter().copied()));
        if let Some(t) = TOOLS
            .iter()
            .find(|t| t.models.iter().any(|x| x.model == name))
        {
            hint = format!(
                "{name} is a model; write pk fit {} <file> --model {name}. ",
                t.name
            );
        } else if std::path::Path::new(name).exists() {
            hint = format!("The tool comes before the files: pk fit <tool> {name}. ");
        }
        CliError::usage("unknown_tool", format!("Unknown tool \"{name}\"."))
            .hint(format!("{hint}Tools: {}.", names.join(", ")))
    })
}

/// The first argument must be a tool.
pub(crate) fn check_args(positional: &[String]) -> Result<()> {
    positional
        .first()
        .map_or(Ok(()), |t| find_tool(t).map(|_| ()))
}

pub fn fit_command(args: &ParsedArgs) -> Result<Output> {
    let tool = find_tool(&args.positional[0])?;
    let files: Vec<String> = args.positional[1..].to_vec();
    let model_name = args.str("model")?;
    let mut unit_given: Option<(&'static str, f64)> = None;
    if let Some(unit) = args.str("units")? {
        let opts = tool.unit_options.ok_or_else(|| {
            let with_units: Vec<&str> = TOOLS
                .iter()
                .filter(|t| t.unit_options.is_some())
                .map(|t| t.name)
                .collect();
            CliError::usage(
                "not_applicable",
                format!("--units doesn't apply to pk fit {}.", tool.name),
            )
            .hint(format!(
                "--units sets the water-potential units for pk fit {}. Leave it out here.",
                with_units.join(" and ")
            ))
        })?;
        let u = opts
            .iter()
            .find(|(l, _)| l.eq_ignore_ascii_case(&unit))
            .ok_or_else(|| {
                CliError::usage(
                    "invalid_value",
                    format!(
                        "--units must be {} for pk fit {}; got \"{unit}\".",
                        crate::options::either(&opts.iter().map(|(l, _)| *l).collect::<Vec<_>>()),
                        tool.name
                    ),
                )
            })?;
        unit_given = Some(*u);
    }
    let survey_paths = args.list("survey");
    let stat_s = args.str("stat")?.unwrap_or_else(|| "mean".into());
    let stat = Stat::parse(&stat_s)
        .ok_or_else(|| CliError::usage("invalid_value", "--stat must be mean, median or max."))?;
    let mut surveys: Vec<FitData> = Vec::new();
    if !survey_paths.is_empty() {
        if !tool.photosynthesis {
            return Err(CliError::usage(
                "not_applicable",
                format!(
                    "--survey (rescale mode) applies only to pk fit photosynthesis, not pk fit {}.",
                    tool.name
                ),
            )
            .hint("Leave --survey out."));
        }
        for p in &survey_paths {
            surveys.push(
                load_photosynthesis(p)
                    .map_err(|e| CliError {
                        message: format!("--survey {p}: {}", e.message),
                        ..e
                    })?
                    .1,
            );
        }
    }
    let ref_ci = if args.flags.contains_key("ref-ci") {
        Some(args.num("ref-ci", f64::NAN)?)
    } else {
        None
    };
    if ref_ci.is_some() && survey_paths.is_empty() {
        return Err(
            CliError::usage("not_applicable", "--ref-ci only applies in rescale mode.")
                .hint("Add --survey <file>, or leave --ref-ci out."),
        );
    }
    let predicted = args.bool("predicted", false);

    let group_by = args.str("group-by")?;
    let mut sources: Vec<Source> = files
        .iter()
        .map(|p| Source {
            name: basename(p),
            path: p.clone(),
            text: None,
        })
        .collect();
    if let Some(col) = &group_by {
        if files.len() != 1 {
            return Err(CliError::usage(
                "invalid_combination",
                format!(
                    "--group-by splits one file, but {} files were given.",
                    files.len()
                ),
            )
            .hint("Give one file with --group-by, or leave it out to fit each file separately."));
        }
        if tool.photosynthesis && files[0].to_lowercase().ends_with(".xlsx") {
            return Err(CliError::usage(
                "invalid_combination",
                "--group-by needs a CSV or LI-6800 log, not .xlsx.",
            )
            .hint("Save the sheet as CSV first."));
        }
        let text = load_file(&files[0])?.text;
        let groups = phytkit_core::group_csv::split_csv_by_column(&text, col)
            .map_err(|e| CliError::usage("invalid_value", format!("--group-by {col}: {e}")))?;
        sources = groups
            .into_iter()
            .map(|g| Source {
                name: format!("{col} {}", g.name),
                path: files[0].clone(),
                text: Some(g.csv),
            })
            .collect();
    }

    let mut results: Vec<Summary> = Vec::new();
    let mut errors: Vec<(String, CliError)> = Vec::new();
    let mut last_entry: &ToolModel = &tool.models[0];
    for src in &sources {
        let t0 = Instant::now();
        let attempt = (|| -> Result<Result<Summary>> {
            let loaded = match load_for(tool, src, model_name.as_deref())? {
                Ok(l) => l,
                Err(e) => return Ok(Err(e)),
            };
            last_entry = loaded.entry;
            let m = model_by_name(loaded.entry.model).expect("registered model");
            let mut data = loaded.data;
            if let Some(c) = tool.unit_column {
                // One unit factor, applied once: --units if given, else the unit the
                // column's name states, else MPa.
                let named = data
                    .mapping
                    .iter()
                    .find(|(k, _)| k == c)
                    .and_then(|(_, field)| {
                        phytkit_core::hydraulics::soil_unit_in_name(field)
                            .map(|u| (field.clone(), u))
                    });
                let (_, factor) = match (unit_given, &named) {
                    (Some(u), Some((field, n))) if n.0 != u.0 => {
                        data.warnings.push(Warning::new(
                            "soil_units_overridden",
                            format!(
                                "Column \"{field}\" names {}, but --units {} was given; read it as {}.",
                                n.0, u.0, u.0
                            ),
                        ));
                        u
                    }
                    (Some(u), _) => u,
                    (None, Some((field, n))) => {
                        if n.1 != 1.0 {
                            data.warnings.push(Warning::new(
                                "unit_converted_by_name",
                                format!(
                                    "Column \"{field}\" is in {} by its name; converted to MPa.",
                                    n.0
                                ),
                            ));
                        }
                        *n
                    }
                    (None, None) => {
                        data.warnings.push(Warning::new(
                            "soil_units_assumed",
                            "No --units given and the column name doesn't state a unit; ψ read as MPa.",
                        ));
                        ("MPa", 1.0)
                    }
                };
                if factor != 1.0 {
                    if let Some(v) = data.x.get_mut(c) {
                        for x in v.iter_mut() {
                            *x *= factor;
                        }
                    }
                }
            }
            let spec = build_spec(loaded.entry, m, &data, args)?;
            let mut result = run_fit(m, &data, &spec);
            if let Some(starts) = loaded.entry.starts {
                // Try the extra starting points for parameters that are fitted and weren't
                // given with --set; keep the best fit (the seeded fit wins ties).
                let set: Vec<String> = args.kv("set")?.into_iter().map(|(k, _)| k).collect();
                for start in starts(&data) {
                    let mut alt = FitSpec {
                        fit_params: spec.fit_params.clone(),
                        init: spec.init.clone(),
                        constraints: spec.constraints.clone(),
                        weights: spec.weights.clone(),
                        max_iter: spec.max_iter,
                    };
                    for (k, v) in start {
                        if spec.fit_params.contains(&k) && !set.contains(&k) {
                            alt.init.insert(k, v);
                        }
                    }
                    let r = run_fit(m, &data, &alt);
                    let better = r.rmse.is_finite()
                        && (!result.rmse.is_finite()
                            || r.rmse < result.rmse * (1.0 - 1e-9)
                            || (!result.converged && r.converged && r.rmse <= result.rmse));
                    if better {
                        result = r;
                    }
                }
            }
            let mut rescale = None;
            let mut factor_out = None;
            if !surveys.is_empty() {
                let q: Vec<f64> = surveys
                    .iter()
                    .flat_map(|s| s.x.get("Q").cloned().unwrap_or_default())
                    .collect();
                let t: Vec<f64> = surveys
                    .iter()
                    .flat_map(|s| s.x.get("T").cloned().unwrap_or_default())
                    .collect();
                let survey_a: Vec<f64> = surveys.iter().flat_map(|s| s.y.clone()).collect();
                let Some((cq, ct)) = survey_conditions(&q, &t) else {
                    return Ok(Err(CliError::failed(
                        "rescale_failed",
                        "Survey files have no usable light/temperature values.",
                    )));
                };
                let ci = match ref_ci
                    .or_else(|| saturating_ci(data.x.get("Ci").map_or(&[][..], |v| v)))
                {
                    Some(c) => c,
                    None => {
                        return Ok(Err(CliError::failed(
                            "rescale_failed",
                            "No usable Ci values in the curve data.",
                        )))
                    }
                };
                let reference = predict_at_ref(m, &result.params, ci, cq, ct);
                let factor = rescale_factor(&survey_a, &[reference], stat);
                if !factor.is_finite() {
                    return Ok(Err(CliError::failed("rescale_failed", "No rescale factor: the survey points or the curve's A at the reference condition are unusable.")));
                }
                let first = result;
                crate::tools::scale_response(&mut data, factor);
                result = run_fit(m, &data, &spec);
                rescale = Some(object(json!({
                    "factor": num(numbers::round_to(factor, 6)),
                    "refCi": num(numbers::round_to(ci, 3)),
                    "refQ": num(numbers::round_to(cq, 3)),
                    "refT": num(numbers::round_to(ct, 3)),
                    "stat": stat.as_str(),
                    "surveyPoints": survey_a.len(),
                    "firstPass": {
                        "r2": num(numbers::round_to(first.r2, 6)),
                        "rmse": num(numbers::round_to(first.rmse, 6)),
                        "params": params_json(&first.params)
                    }
                })));
                factor_out = Some(numbers::round_to(factor, 6));
            }
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            Ok(Ok(summarize(
                &loaded.name,
                tool.name,
                m,
                &data,
                &result,
                loaded.converted,
                predicted,
                rescale,
                factor_out,
                ms,
            )))
        })()?;
        match attempt {
            Ok(s) => results.push(s),
            Err(e) => errors.push((src.name.clone(), e)),
        }
    }

    let format = args.str("format")?.unwrap_or_else(|| "json".into());
    if format == "csv" {
        return Ok(Output::Raw(batch_csv(&results, &errors, last_entry)));
    }
    if sources.len() == 1 {
        if let Some((_, e)) = errors.into_iter().next() {
            return Err(e);
        }
        return Ok(Output::Json(
            results.into_iter().next().expect("one result").obj,
        ));
    }
    let errors: Vec<Value> = errors
        .into_iter()
        .map(
            |(file, e)| json!({ "file": file, "code": e.code, "error": e.message, "hint": e.hint }),
        )
        .collect();
    let o = object(json!({
        "tool": tool.name,
        "files": sources.len(),
        "groupBy": group_by,
        "fitted": results.len(),
        "failed": errors.len(),
        "results": results.into_iter().map(|s| Value::Object(s.obj)).collect::<Vec<_>>(),
        "errors": errors,
    }));
    Ok(Output::Json(o))
}

/// Warnings about the fit itself: it did not converge, or a fitted parameter ended on a
/// bound (so the bound, not the data, set its value).
pub fn fit_warnings(m: &dyn Model, r: &FitResult) -> Vec<Warning> {
    let mut out = Vec::new();
    if !r.converged {
        out.push(Warning::new(
            "not_converged",
            format!(
                "The fit stopped after {} iteration(s) without converging; the parameters may not be the best fit. Raise --max-iter, or check the data and the starting values.",
                r.iterations
            ),
        ));
    }
    for name in &r.fitted {
        let (Some(spec), Some(&v)) = (
            m.params().iter().find(|p| p.name == name.as_str()),
            r.params.get(name),
        ) else {
            continue;
        };
        let (lb, ub) = (
            spec.lb.unwrap_or(f64::NEG_INFINITY),
            spec.ub.unwrap_or(f64::INFINITY),
        );
        // Within 1e-6 of the bound, relative to the bound's size (or to 1 near zero).
        let near = |b: f64| b.is_finite() && (v - b).abs() <= 1e-6 * b.abs().max(1.0);
        let side = if near(lb) {
            Some(("lower", lb))
        } else if near(ub) {
            Some(("upper", ub))
        } else {
            None
        };
        if let Some((which, b)) = side {
            out.push(Warning::new(
                "parameter_at_bound",
                format!(
                    "{name} ended on its {which} bound ({}): the bound, not the data, sets its value.",
                    numbers::text(b)
                ),
            ));
        }
    }
    out
}
