//! `pk describe`: the tools, models, parameters and constraints.

use crate::args::{ParsedArgs, Result};
use crate::options::{describe_commands, unknown_model};
use crate::tools::{poro_models, TOOLS};
use crate::{object, Obj};
use phytkit_core::models::{builtin_models, model_by_name};
use serde_json::{json, Value};

pub fn describe_model(name: &str) -> Result<Value> {
    let m = model_by_name(name).ok_or_else(|| {
        let known: Vec<&str> = builtin_models().iter().map(|m| m.name()).collect();
        unknown_model(name, &known)
    })?;
    let params: Vec<Value> = m
        .params()
        .iter()
        .map(|p| json!({ "name": p.name, "units": p.units, "default": p.default, "lb": p.lb, "ub": p.ub, "fixedByDefault": p.fixed, "description": p.description }))
        .collect();
    let io = |i: phytkit_core::model::InputSpec| json!({ "name": i.name, "units": i.units, "description": i.description });
    let inputs: Vec<Value> = m.input_specs().into_iter().map(io).collect();
    let constraints: Vec<Value> = m
        .constraints()
        .iter()
        .map(|c| json!({ "key": c.key, "label": c.label, "weight": c.weight, "description": c.description, "perCurve": c.per_curve, "defaultOn": c.default_on }))
        .collect();
    Ok(
        json!({ "name": m.name(), "inputs": inputs, "response": io(m.response()), "params": params, "constraints": constraints, "latex": m.latex() }),
    )
}

pub fn describe(args: &ParsedArgs) -> Result<Obj> {
    if let Some(which) = args.positional.first() {
        return Ok(object(json!({ "model": describe_model(which)? })));
    }
    let tools: Vec<Value> = TOOLS
        .iter()
        .map(|t| {
            let models: Vec<Value> =
                t.models.iter().enumerate().map(|(i, m)| json!({ "name": m.model, "label": m.label, "default": i == 0 })).collect();
            let units = t.unit_options.map(|opts| json!({ "column": t.unit_column, "options": opts.iter().map(|(l, _)| *l).collect::<Vec<_>>() }));
            json!({ "name": t.name, "description": t.description, "models": models, "units": units, "batch": true, "rescale": t.photosynthesis })
        })
        .collect();
    let estimate_models: Vec<Value> = poro_models()
        .iter()
        .map(|(n, l)| json!({ "name": n, "label": l }))
        .collect();
    Ok(object(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "tools": tools,
        "li600": {
            "correct": {
                "description": "Rizzo & Bailey (2025) energy-balance correction of LI-600 gsw",
                "options": option_summary("li600 correct")
            },
            "estimate": {
                "description": "Solve A, Ci, Cc from LI-600 gsw + a photosynthesis parameter set",
                "models": estimate_models,
                "options": option_summary("li600 estimate")
            }
        },
        "concat": { "description": "Merge LI-6800 logs / CSVs under one header with a CurveID per file" },
        "models": builtin_models().iter().map(|m| m.name()).collect::<Vec<_>>(),
        "commands": describe_commands(),
        "errors": {
            "shape": "{\"ok\": false, \"command\", \"error\": {\"code\", \"message\", \"usage\", \"hint\"?, \"help\"?}}",
            "usage": "true when the command line was wrong (fix the arguments and retry); false when a file or its data was",
            "codes": crate::args::ERROR_CODES.iter().map(|(c, d)| json!({ "code": c, "description": d })).collect::<Vec<_>>()
        }
    })))
}

/// A command's options as "name (default)" or "name", from the COMMANDS table (the full
/// detail is in the `commands` section).
fn option_summary(name: &str) -> Vec<String> {
    crate::options::command(name)
        .map(|c| {
            c.options
                .iter()
                .map(|o| match o.default {
                    Some(d) => format!("{} ({d})", o.name),
                    None => o.name.to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}
