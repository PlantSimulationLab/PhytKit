//! Python bindings for phytkit. Commands run the same Rust code as the `pk` CLI and return
//! the same structure as its JSON; the array functions fit data already in memory.

use phytkit_cli::options::{unknown_constraint, unknown_model, unknown_parameter};
use phytkit_cli::{envelope, run as run_cli, Error as CliError, Output};
use phytkit_core::model::{Columns, Params};
use phytkit_core::models::{builtin_models, model_by_name};
use phytkit_core::{fit, FitOptions, FitResult};
use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rayon::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

create_exception!(
    _phytkit,
    PhytKitError,
    PyException,
    "A phytkit command failed. `.code` names the kind of failure (see `describe()[\"errors\"]`), `.hint` says what to do about it, and `.usage` is True when the arguments were wrong."
);

/// A JSON value from the command layer as the Python object `json.loads` would give.
fn to_py(py: Python<'_>, v: &Value) -> PyResult<Py<PyAny>> {
    Ok(match v {
        Value::Null => py.None(),
        Value::Bool(b) => b.into_pyobject(py)?.to_owned().into_any().unbind(),
        Value::Number(n) => match n.as_i64() {
            Some(i) => i.into_pyobject(py)?.into_any().unbind(),
            None => n
                .as_f64()
                .unwrap_or(f64::NAN)
                .into_pyobject(py)?
                .into_any()
                .unbind(),
        },
        Value::String(s) => s.into_pyobject(py)?.into_any().unbind(),
        Value::Array(a) => {
            let l = PyList::empty(py);
            for x in a {
                l.append(to_py(py, x)?)?;
            }
            l.into_any().unbind()
        }
        Value::Object(o) => {
            let d = PyDict::new(py);
            for (k, x) in o {
                d.set_item(k, to_py(py, x)?)?;
            }
            d.into_any().unbind()
        }
    })
}

/// Run a `pk` command (arguments as on the command line, without `pk`).
/// Returns a dict for JSON output or a str for CSV and help text; raises PhytKitError on failure.
#[pyfunction]
fn run(py: Python<'_>, argv: Vec<String>) -> PyResult<Py<PyAny>> {
    let result = py.detach(|| run_cli(&argv));
    match result {
        Ok(Output::Raw(s)) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        Ok(Output::Json(obj)) => to_py(
            py,
            &Value::Object(envelope(phytkit_cli::command_name(&argv).as_deref(), obj)),
        ),
        Err(e) => {
            let help = phytkit_cli::options::command(
                &phytkit_cli::command_name(&argv).unwrap_or_default(),
            )
            .filter(|_| e.usage)
            .map(phytkit_cli::options::usage_line);
            Err(py_error(py, e, help))
        }
    }
}

/// The `pk` command line: run `argv` and return what `pk` prints and its exit code.
#[pyfunction]
fn main(py: Python<'_>, argv: Vec<String>) -> (String, i32) {
    py.detach(|| phytkit_cli::render(&argv, run_cli(&argv)))
}

/// Names of the built-in models.
#[pyfunction]
fn models() -> Vec<String> {
    builtin_models()
        .iter()
        .map(|m| m.name().to_string())
        .collect()
}

/// A `PhytKitError` carrying the error's code, hint and usage flag.
fn py_error(py: Python<'_>, e: CliError, help: Option<String>) -> PyErr {
    let err = PhytKitError::new_err(e.message);
    let v = err.value(py);
    let set = || -> PyResult<()> {
        v.setattr("code", e.code)?;
        v.setattr("usage", e.usage)?;
        v.setattr("hint", e.hint)?;
        v.setattr("help", help)
    };
    set().map_or_else(|failed| failed, |_| err)
}

fn fail(e: CliError) -> PyErr {
    Python::attach(|py| py_error(py, e, None))
}

fn model(name: &str) -> PyResult<&'static dyn phytkit_core::Model> {
    model_by_name(name).ok_or_else(|| {
        let known: Vec<&str> = builtin_models().iter().map(|m| m.name()).collect();
        fail(unknown_model(name, &known))
    })
}

/// Reject parameter and constraint names the model doesn't have, rather than ignore them.
fn check_names(
    m: &dyn phytkit_core::Model,
    label: &str,
    names: impl IntoIterator<Item = String>,
) -> PyResult<()> {
    let known: Vec<&str> = m.params().iter().map(|p| p.name).collect();
    for n in names {
        if !known.contains(&n.as_str()) {
            return Err(fail(unknown_parameter(label, &n, m.name(), &known)));
        }
    }
    Ok(())
}

fn check_constraints(
    m: &dyn phytkit_core::Model,
    label: &str,
    keys: impl IntoIterator<Item = String>,
) -> PyResult<()> {
    let known: Vec<&str> = m.constraints().iter().map(|c| c.key).collect();
    for k in keys {
        if !known.contains(&k.as_str()) {
            return Err(fail(unknown_constraint(label, &k, m.name(), &known)));
        }
    }
    Ok(())
}

fn columns(x: &Bound<'_, PyDict>) -> PyResult<Columns> {
    let mut c = Columns::new();
    for (k, v) in x.iter() {
        c.insert(k.extract::<String>()?, v.extract::<Vec<f64>>()?);
    }
    Ok(c)
}

fn params(p: Option<&Bound<'_, PyDict>>) -> PyResult<Params> {
    let mut out = Params::new();
    if let Some(p) = p {
        for (k, v) in p.iter() {
            out.insert(k.extract::<String>()?, v.extract::<f64>()?);
        }
    }
    Ok(out)
}

fn result_dict<'py>(
    py: Python<'py>,
    m: &dyn phytkit_core::Model,
    x: &Columns,
    r: &FitResult,
) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    let p = PyDict::new(py);
    for (k, v) in &r.params {
        p.set_item(k, v)?;
    }
    d.set_item("params", p)?;
    d.set_item("fitted", r.fitted.clone())?;
    d.set_item("predicted", r.predicted.clone())?;
    d.set_item("rmse", r.rmse)?;
    d.set_item("r2", r.r2)?;
    d.set_item("iterations", r.iterations)?;
    d.set_item("converged", r.converged)?;
    let w = phytkit_core::warn::to_json(&phytkit_cli::fit_warnings(m, x, r));
    d.set_item("warnings", to_py(py, &w)?)?;
    Ok(d)
}

#[allow(clippy::too_many_arguments)]
fn options(
    m: &dyn phytkit_core::Model,
    fit_params: Option<Vec<String>>,
    init: Option<&Bound<'_, PyDict>>,
    max_iter: Option<usize>,
    curves: Option<Vec<Vec<usize>>>,
    constraints: Option<Vec<String>>,
    constraint_weights: Option<HashMap<String, f64>>,
) -> PyResult<FitOptions> {
    let init = params(init)?;
    check_names(m, "fit_params", fit_params.iter().flatten().cloned())?;
    check_names(m, "init", init.keys().cloned())?;
    let fitted: Vec<String> = match &fit_params {
        Some(f) => f.clone(),
        None => m
            .params()
            .iter()
            .filter(|p| !p.fixed)
            .map(|p| p.name.to_string())
            .collect(),
    };
    for (k, v) in &init {
        if fitted.contains(k) {
            if let Some(e) = phytkit_cli::options::start_out_of_bounds(
                m,
                "init",
                k,
                *v,
                "leave it out of fit_params",
            ) {
                return Err(fail(e));
            }
        }
    }
    check_constraints(m, "constraints", constraints.iter().flatten().cloned())?;
    check_constraints(
        m,
        "constraint_weights",
        constraint_weights.iter().flat_map(|w| w.keys().cloned()),
    )?;
    if max_iter == Some(0) {
        return Err(fail(CliError::usage(
            "invalid_value",
            "max_iter must be at least 1.",
        )));
    }
    Ok(FitOptions {
        fit_params,
        init,
        max_iter,
        curves,
        constraints,
        constraint_weights,
        ..Default::default()
    })
}

/// Every model input is present, the input columns have the same length, and (with `y`)
/// they have one value per observation and every value is finite. `at` prefixes messages
/// (for example "datasets[3]: ") so a bad dataset in `fit_many` can be found.
fn check_inputs(
    m: &dyn phytkit_core::Model,
    x: &Columns,
    y: Option<&[f64]>,
    at: &str,
) -> PyResult<()> {
    let inputs = m.inputs();
    let mut first: Option<(&str, usize)> = None;
    for name in inputs {
        match x.get(*name) {
            None => {
                let given: Vec<&str> = x.keys().map(String::as_str).collect();
                let guess = phytkit_cli::options::suggest(name, given.iter().copied());
                let hint = match guess {
                    Some(g) => format!(
                        "Rename \"{g}\" to \"{name}\". {} reads: {}.",
                        m.name(),
                        inputs.join(", ")
                    ),
                    None => format!("{} reads: {}.", m.name(), inputs.join(", ")),
                };
                return Err(fail(
                    CliError::usage(
                        "missing_input",
                        format!("x has no \"{name}\" column, which {} needs.", m.name()),
                    )
                    .hint(hint),
                ));
            }
            Some(c) => {
                if let Some(y) = y.filter(|y| y.len() != c.len()) {
                    return Err(fail(
                        CliError::usage(
                            "invalid_value",
                            format!(
                                "{at}x[\"{name}\"] has {} values but y has {}.",
                                c.len(),
                                y.len()
                            ),
                        )
                        .hint("Give one value per observation in every column and in y."),
                    ));
                }
                match first {
                    Some((f, n)) if n != c.len() => {
                        return Err(fail(
                            CliError::usage(
                                "invalid_value",
                                format!(
                                    "{at}x[\"{name}\"] has {} values but x[\"{f}\"] has {n}.",
                                    c.len()
                                ),
                            )
                            .hint("Give every input column the same number of values."),
                        ));
                    }
                    None => first = Some((name, c.len())),
                    _ => {}
                }
                if y.is_some() {
                    if let Some(i) = c.iter().position(|v| !v.is_finite()) {
                        return Err(fail(not_finite(at, &format!("x[\"{name}\"]"), i, c[i])));
                    }
                }
            }
        }
    }
    if let Some(y) = y {
        if let Some(i) = y.iter().position(|v| !v.is_finite()) {
            return Err(fail(not_finite(at, "y", i, y[i])));
        }
    }
    Ok(())
}

fn not_finite(at: &str, what: &str, i: usize, v: f64) -> CliError {
    CliError::usage(
        "invalid_value",
        format!("{at}{what}[{i}] is {v}; every value must be a finite number."),
    )
    .hint("Drop the rows with missing or infinite values before fitting.")
}

/// Every index in `curves` points at a row of the data.
fn check_curves(curves: Option<&Vec<Vec<usize>>>, n: usize) -> PyResult<()> {
    for (c, rows) in curves.into_iter().flatten().enumerate() {
        if let Some(&i) = rows.iter().find(|&&i| i >= n) {
            return Err(fail(
                CliError::usage(
                    "invalid_value",
                    format!("curves[{c}] has row index {i}, but there are only {n} rows."),
                )
                .hint("Row indices in curves are 0-based and must be less than len(y)."),
            ));
        }
    }
    Ok(())
}

/// Fit a model to data in memory.
///
/// `x` maps each of the model's inputs (see `describe(model)`) to a sequence of values;
/// temperatures for the photosynthesis models are in Kelvin. Returns params, fitted names,
/// predicted values, rmse, r2, iterations, converged and warnings (not_converged,
/// parameter_at_bound).
#[pyfunction]
#[pyo3(signature = (model_name, x, y, *, fit_params=None, init=None, max_iter=None, curves=None, constraints=None, constraint_weights=None))]
#[allow(clippy::too_many_arguments)]
fn fit_arrays<'py>(
    py: Python<'py>,
    model_name: &str,
    x: &Bound<'py, PyDict>,
    y: Vec<f64>,
    fit_params: Option<Vec<String>>,
    init: Option<&Bound<'py, PyDict>>,
    max_iter: Option<usize>,
    curves: Option<Vec<Vec<usize>>>,
    constraints: Option<Vec<String>>,
    constraint_weights: Option<HashMap<String, f64>>,
) -> PyResult<Bound<'py, PyDict>> {
    let m = model(model_name)?;
    let cols = columns(x)?;
    check_inputs(m, &cols, Some(&y), "")?;
    check_curves(curves.as_ref(), y.len())?;
    let opts = options(
        m,
        fit_params,
        init,
        max_iter,
        curves,
        constraints,
        constraint_weights,
    )?;
    let r = py.detach(|| fit(m, &cols, &y, &opts));
    result_dict(py, m, &cols, &r)
}

/// Fit the same model to many datasets in parallel. `datasets` is a list of `(x, y)` pairs.
/// `threads` limits the worker threads (default: all cores). Results keep the input order.
#[pyfunction]
#[pyo3(signature = (model_name, datasets, *, fit_params=None, init=None, max_iter=None, constraints=None, constraint_weights=None, threads=None))]
#[allow(clippy::too_many_arguments)]
fn fit_many<'py>(
    py: Python<'py>,
    model_name: &str,
    datasets: Vec<(Bound<'py, PyDict>, Vec<f64>)>,
    fit_params: Option<Vec<String>>,
    init: Option<&Bound<'py, PyDict>>,
    max_iter: Option<usize>,
    constraints: Option<Vec<String>>,
    constraint_weights: Option<HashMap<String, f64>>,
    threads: Option<usize>,
) -> PyResult<Bound<'py, PyList>> {
    let m = model(model_name)?;
    let mut data = Vec::with_capacity(datasets.len());
    for (i, (x, y)) in datasets.iter().enumerate() {
        let cols = columns(x)?;
        check_inputs(m, &cols, Some(y), &format!("datasets[{i}]: "))?;
        data.push((cols, y.clone()));
    }
    let opts = options(
        m,
        fit_params,
        init,
        max_iter,
        None,
        constraints,
        constraint_weights,
    )?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.unwrap_or(0))
        .build()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let results: Vec<FitResult> =
        py.detach(|| pool.install(|| data.par_iter().map(|(x, y)| fit(m, x, y, &opts)).collect()));
    let out = PyList::empty(py);
    for (r, (x, _)) in results.iter().zip(&data) {
        out.append(result_dict(py, m, x, r)?)?;
    }
    Ok(out)
}

/// Model predictions for a parameter set (missing parameters take their defaults).
#[pyfunction]
fn predict(
    model_name: &str,
    params_: &Bound<'_, PyDict>,
    x: &Bound<'_, PyDict>,
) -> PyResult<Vec<f64>> {
    let m = model(model_name)?;
    let mut p: Params = m
        .params()
        .iter()
        .map(|s| (s.name.to_string(), s.default))
        .collect();
    let given = params(Some(params_))?;
    check_names(m, "params", given.keys().cloned())?;
    p.extend(given);
    let cols = columns(x)?;
    check_inputs(m, &cols, None, "")?;
    Ok(m.predict(&p, &cols))
}

/// Which build of the native code this is: version, git commit, whether the tree had
/// uncommitted changes, release wheel or local build, and the Rust profile and target.
#[pyfunction]
fn build_info(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("version", env!("CARGO_PKG_VERSION"))?;
    d.set_item("commit", env!("PHYTKIT_GIT_COMMIT"))?;
    d.set_item("dirty", env!("PHYTKIT_GIT_DIRTY") == "true")?;
    d.set_item("build", env!("PHYTKIT_BUILD_KIND"))?;
    d.set_item(
        "profile",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
    )?;
    d.set_item(
        "target",
        format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
    )?;
    Ok(d)
}

#[pymodule]
fn _phytkit(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(models, m)?)?;
    m.add_function(wrap_pyfunction!(fit_arrays, m)?)?;
    m.add_function(wrap_pyfunction!(fit_many, m)?)?;
    m.add_function(wrap_pyfunction!(predict, m)?)?;
    m.add_function(wrap_pyfunction!(build_info, m)?)?;
    m.add("PhytKitError", m.py().get_type::<PhytKitError>())?;
    m.add_function(wrap_pyfunction!(main, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
