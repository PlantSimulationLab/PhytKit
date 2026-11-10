# Python API

```python
import phytkit as pk
```

## Commands

These run `pk` commands and return what `pk` would print: a `dict` for JSON
output, or a `str` for CSV output (`format="csv"`, or `concat` without `out`
or `format="json"`). Options are `pk`'s options as keyword
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
| `y` | Sequence of measured responses, the same length as each input. Every value in `x` and `y` must be finite |
| `fit_params` | Exactly the parameters to optimize (default: those not fixed by default) |
| `init` | Dict of starting values (within the bounds for parameters being fitted), or the values fixed parameters are held at |
| `max_iter` | Iteration limit (default 200) |
| `curves` | List of 0-based row-index lists, one per curve, for per-curve constraints |
| `constraints` | Exactly the constraint keys to apply (default: those on by default) |
| `constraint_weights` | Dict of constraint key → weight |

Returns a dict:

| Key | Meaning |
|---|---|
| `params` | Every parameter's value |
| `fitted` | Names of the optimized parameters |
| `predicted` | Model prediction for each point |
| `rmse`, `r2` | Fit quality on `y` |
| `iterations`, `converged` | How the fit finished; see [How fitting works](../concepts/fitting.md#when-the-fit-stops) |
| `warnings` | `not_converged` and `parameter_at_bound` entries, as in [Output](output.md#warnings) |

### `fit_many`

```python
pk.fit_many(model_name, datasets, *, fit_params=None, init=None, max_iter=None,
            constraints=None, constraint_weights=None, threads=None) -> list[dict]
```

Fits each `(x, y)` pair in `datasets` in parallel and returns one result per
dataset, in order, each as from `fit_arrays`. `threads` sets the number of worker
threads. When it is `None`, the `RAYON_NUM_THREADS` environment variable sets it,
and without that every core is used. `fit_many` is the only function that starts
threads; the others run on the calling thread.

Results don't depend on the thread count: the parameters are bit-identical at
any `threads`, and identical to fitting each dataset with `fit_arrays`.

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

## Warnings

`PhytKitWarning`
:   A `UserWarning` subclass. Every function that returns a result with
    `warnings` also raises each entry whose level is `warning` through Python's
    `warnings` module, with the message `[code] file: message`. Entries whose level
    is `info` are only in the result. See [Output](output.md#warnings) for the codes.
    To turn them into errors, use `warnings.simplefilter("error", pk.PhytKitWarning)`.

## Version and build

`pk.__version__` is the package version.

`pk.build_info()` returns which build of the native code is running:

```python
{"version": "0.2.0", "commit": "<40-character git hash, or unknown>", "dirty": False,
 "build": "release", "profile": "release", "target": "x86_64-linux"}
```

`build` is `"release"` for a wheel from PyPI and `"local"` for a wheel you built
yourself; `dirty` is `True` when a local build had uncommitted changes.

## Reproducibility

The same inputs give bit-identical results on the same build: starting values come
from the data, never from a random number generator, and `fit_many` gives the same
parameters at every thread count. Only `elapsedMs` differs from run to run. PhytKit
reads only the files you give it and writes only `out=` files: no cache, no
configuration, nothing under your home directory, and no network access.
