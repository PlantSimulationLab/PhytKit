# Python API

```python
import phytkit as pk
```

## Commands

These run `pk` commands and return what `pk` would print: a `dict` for JSON
output, or a `str` for `format="csv"`. Options are `pk`'s options as keyword
arguments, with `_` in place of `-` (see
[Scripting with Python](../guide/python.md#the-same-commands-as-pk)). They
raise `PhytKitError` on failure.

| Function | `pk` command |
|---|---|
| `pk.describe(model=None)` | `pk describe [model]` |
| `pk.fit(tool, *files, **options)` | `pk fit <tool> <file...>` |
| `pk.li600_correct(file, **options)` | `pk li600 correct <file>` |
| `pk.li600_estimate(file, **options)` | `pk li600 estimate <file>` |
| `pk.concat(*files, **options)` | `pk concat <file...>` |
| `pk.run(argv)` | Any `pk` command line, as a list of strings |

How keyword values become options:

| Python value | Becomes |
|---|---|
| `True` | the option alone: `predicted=True` → `--predicted` |
| `False` or `None` | nothing |
| a dict | one `--name key=value` per entry: `set={"gm": 1e6}` |
| a list | for `fit` and `fix`, one comma-separated option; for others, the option repeated: `survey=["a", "b"]` |
| anything else | `--name value`, with paths converted to strings |

## Fitting data in memory

### `fit_arrays`

```python
pk.fit_arrays(model_name, x, y, *, fit_params=None, init=None, max_iter=None,
              curves=None, constraints=None, constraint_weights=None) -> dict
```

| Argument | Meaning |
|---|---|
| `model_name` | A model from `pk.models()` |
| `x` | Dict of input name → sequence of values; see [Models and parameters](models.md) for each model's inputs and units (temperature in Kelvin) |
| `y` | Sequence of measured responses, the same length as each input |
| `fit_params` | Parameters to optimize (default: those not fixed by default) |
| `init` | Dict of starting values, or the values fixed parameters are held at |
| `max_iter` | Iteration limit (default 200) |
| `curves` | List of row-index lists, one per curve, for per-curve constraints |
| `constraints` | Constraint keys to apply (default: all) |
| `constraint_weights` | Dict of constraint key → weight |

Returns a dict:

| Key | Meaning |
|---|---|
| `params` | Every parameter's value |
| `fitted` | Names of the optimized parameters |
| `predicted` | Model prediction for each point |
| `rmse`, `r2` | Fit quality on `y` |
| `iterations`, `converged` | How the fit finished; see [How fitting works](../concepts/fitting.md#when-the-fit-stops) |

### `fit_many`

```python
pk.fit_many(model_name, datasets, *, fit_params=None, init=None, max_iter=None,
            constraints=None, constraint_weights=None, threads=None) -> list[dict]
```

Fits each `(x, y)` pair in `datasets` in parallel and returns one result per
dataset, in order, each as from `fit_arrays`. `threads` limits the worker
threads (default: every core).

### `predict`

```python
pk.predict(model_name, params, x) -> list[float]
```

The model's prediction at `x`. Parameters missing from `params` take their
defaults.

### `models`

```python
pk.models() -> list[str]
```

The names of every built-in model.

## Errors

`PhytKitError`
:   Raised by every function. It has the fields of `pk`'s error object:

    | Attribute | |
    |---|---|
    | `str(e)` | The message |
    | `e.code` | The kind of error, such as `unknown_option` (see [Commands](commands.md#errors)) |
    | `e.hint` | What to do about it, or `None` |
    | `e.usage` | `True` when the call was wrong, `False` when a file or its data was |
    | `e.help` | The command's usage line, on usage errors from command functions, or `None` |

## Version

`pk.__version__` is the package version.
