# Adding a model

A model is a type that implements `phytkit_core::Model`. `models/bta.rs` is a
short example to copy.

## 1. Implement the trait

```rust
use crate::model::{col, get, param, Columns, Model, ParamSpec, Params};

pub struct MyModel { params: Vec<ParamSpec> }

impl MyModel {
    pub fn new() -> Self {
        MyModel {
            params: vec![
                param("a", 1.0, 0.0, 10.0).describe("scale"),
                param("b", 0.5, 0.0, 5.0),
                param("c", 0.0, -1.0, 1.0).fixed(),   // fixed by default; --fit lists exactly what to fit
            ],
        }
    }
}

impl Default for MyModel {
    fn default() -> Self { Self::new() }
}

impl Model for MyModel {
    fn name(&self) -> &str { "myModel" }
    fn inputs(&self) -> &[&'static str] { &["Q"] }
    fn params(&self) -> &[ParamSpec] { &self.params }
    fn predict(&self, p: &Params, x: &Columns) -> Vec<f64> {
        // Read parameters once, outside the loop: looking them up per point is slow.
        let (a, b, c) = (get(p, "a"), get(p, "b"), get(p, "c"));
        col(x, "Q").iter().map(|q| a * q / (q + b) + c).collect()
    }
}
```

Optional methods:

| Method | Use it for |
|---|---|
| `jacobian` and `has_jacobian` | Exact derivatives, instead of finite differences (not used when the model also has penalties) |
| `penalties` and `has_penalties`, `constraints` | Soft constraints (each `ConstraintSpec` sets `default_on`; when the caller doesn't choose, `penalties` should apply only those) |
| `derived` | Values computed from the parameters, reported with each fit (like `psi_tlp`) |
| `latex` | The equation, shown by `pk describe` and in these docs |

## 2. Register it

Add the module to `models/mod.rs`, and add an instance to `builtin_models()`.
The model is then available to `pk describe`, `fit_arrays`, `fit_many` and
`predict`.

## 3. Add it to a tool (optional)

To fit it from files with `pk fit`, add a `ToolModel` to a tool in
`crates/cli/src/tools.rs`. That entry sets:

- the label,
- the parser that reads its inputs from a file, and
- an optional seed function that sets starting values from the data.

## 4. Test and document it

Registering a model makes `cargo test --release` fail until the model
reference test knows about it:

- add the model's name to the expected list in
  `crates/core/tests/model_reference.rs`;
- add an entry for it (inputs, parameter sets, fits) to
  `crates/core/tests/data/model_reference.json`, then run
  `UPDATE_MODEL_REFERENCE=1 cargo test --release --test model_reference` to
  fill in the stored values. The update only rewrites entries that already
  exist; it doesn't create one.

Add tests in the model's file, run `python3 docs/gen_models.py` to add it to
[Models and parameters](../reference/models.md), and describe it in
[Concepts](../concepts/index.md).
