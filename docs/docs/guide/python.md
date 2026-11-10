# Scripting with Python

The `phytkit` Python package runs the same code as `pk`. Use it to fit many
files in a loop, to feed results into pandas or plots, or to fit data you
already have in memory. Install it as described in
[Install PhytKit](install.md#the-python-package).

## The same commands as `pk`

Each `pk` command has a function. It returns a dictionary with the same
contents as `pk`'s JSON output:

```python
import phytkit as pk

fit = pk.fit("photosynthesis", "examples/aci.csv", model="fvcb", set={"gm": 1e6}, fix=["gm", "alphaG"])
print(fit["r2"], fit["params"]["Vcmax25"])
```

Options are `pk`'s options as keyword arguments, with `_` in place of `-`:

| `pk` | Python |
|---|---|
| `--model fvcb` | `model="fvcb"` |
| `--max-iter 400` | `max_iter=400` |
| `--fix gm,alphaG` | `fix=["gm", "alphaG"]` |
| `--set Vcmax25=80 --set gm=1000000` | `set={"Vcmax25": 80, "gm": 1e6}` |
| `--survey a.csv --survey b.csv` | `survey=["a.csv", "b.csv"]` |
| `--predicted` | `predicted=True` |
| `--format csv` | `format="csv"` (returns the CSV text as a string) |

The other commands work the same way:

```python
pk.describe("fvcb")
pk.fit("hydraulics", "examples/pv.csv", model="pvcurve", group_by="Leaf")
pk.li600_correct("examples/li600.csv", sidedness=1)
pk.li600_estimate("examples/li600.csv", set={"Vcmax25": 90, "Jmax25": 170})
pk.concat("examples/aci.csv", "examples/messy_aci.csv", format="json")
```

`pk.run([...])` runs any `pk` command line given as a list, for example
`pk.run(["fit", "soil", "examples/soil.csv", "--group-by", "sample", "--units", "kPa"])`.

## Data already in memory

`fit_arrays` fits a model to arrays you pass in directly: lists, NumPy arrays,
or pandas columns.

```python
import phytkit as pk

x = {"Q": [0, 100, 200, 400, 800, 1200, 1600, 2000]}
y = [-1.4, 4.6, 8.4, 12.9, 17.1, 18.6, 19.5, 20.0]
result = pk.fit_arrays("lightResponse", x, y)
print(result["params"], result["rmse"])
```

`x` maps each of the model's inputs to a column of values. `pk.describe(model)`
lists the inputs, and [Models and parameters](../reference/models.md) gives
their units. **Temperatures are in Kelvin** here. The file commands convert °C
for you, but `fit_arrays` takes values as they are.

The result has `params`, `fitted`, `predicted`, `rmse`, `r2`, `iterations`
and `converged`. Optional arguments match the fitter's options:

```python
pk.fit_arrays("fvcb", {"Ci": ci, "Q": q, "T": t_kelvin}, a,
              fit_params=["Vcmax25", "Jmax25", "TPU25", "Rd25"],   # what to optimize
              init={"gm": 1e6},                                    # starting or fixed values
              max_iter=400,
              curves=[[0, 1, 2], [3, 4, 5]])                       # row indices of each curve
```

`predict` evaluates a model. Parameters you leave out take their defaults:

```python
pk.predict("fvcb", {"Vcmax25": 90, "Jmax25": 170}, {"Ci": [200, 400, 800], "Q": [1500] * 3, "T": [298.15] * 3})
```

## Fit many datasets in parallel

`fit_many` fits the same model to a list of `(x, y)` datasets on all your CPU
cores. It returns the results in the same order:

```python
results = pk.fit_many("fvcb", [(x1, y1), (x2, y2), (x3, y3)], threads=8)
```

`init` sets starting values for fitted parameters, which must lie within
their bounds (a value outside is an error), and the values that fixed
parameters are held at, which may be anything. To hold a parameter such as
`gm`, leave it out of `fit_params`, which lists exactly the parameters to fit.

See [Fit many curves at once](../workflows/batch-fitting.md).

## Errors

Every function raises `phytkit.PhytKitError` on failure, with the same
fields as `pk`'s error object:

```python
try:
    pk.fit("photosynthesis", "examples/aci.csv", fix=["Vcmax"])
except pk.PhytKitError as e:
    print(e.code)   # unknown_parameter
    print(e)        # --fix Vcmax: fvcb has no parameter "Vcmax".
    print(e.hint)   # Did you mean Vcmax25? Parameters: Vcmax25, Jmax25, ...
    print(e.usage)  # True
```

Keyword arguments are checked like `pk`'s options, so a misspelled one
(`sidednes=2`) raises `unknown_option` rather than being ignored.
`fit_arrays`, `fit_many` and `predict` check their inputs the same way:
unknown parameter or constraint names, a missing input column, and input
columns of different lengths are all errors. `fit_arrays` and `fit_many` also
reject a column whose length differs from `y`, any NaN or infinite value in
the data, and a `curves` row index past the end of `y`. Drop incomplete rows
before fitting.

## Warnings

Results carry a `warnings` list of `{code, level, message}` objects (see
[Output](../reference/output.md#warnings)). Each entry whose level is `warning`
(something that may make the result wrong, such as a parameter that ended on its
bound) is also raised as a `phytkit.PhytKitWarning`:

```python
import warnings
with warnings.catch_warnings():
    warnings.simplefilter("error", pk.PhytKitWarning)   # stop on any of them
    res = pk.fit("photosynthesis", "examples/aci.csv")
```

The full list of functions and arguments is in [Python API](../reference/python-api.md).
