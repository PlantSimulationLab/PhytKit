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
                param("c", 0.0, -1.0, 1.0).fixed(),   // held fixed unless freed with --fit
            ],
        }
    }
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
| `jacobian` and `has_jacobian` | Exact derivatives, instead of finite differences |
| `penalties` and `has_penalties`, `constraints` | Soft constraints |
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

Add tests in the model's file, run `python3 docs/gen_models.py` to add it to
[Models and parameters](../reference/models.md), and describe it in
[Concepts](../concepts/index.md).
