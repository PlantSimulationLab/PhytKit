# How fitting works

Every `pk fit` and `fit_arrays` call uses the same optimizer: a bounded
**Levenberg–Marquardt** least-squares fit. It finds the parameter values that
minimize the sum of squared differences between the model and your
measurements, the same objective as MATLAB's `lsqcurvefit`.

## The optimizer

At each iteration the fitter estimates how the residuals change with each
parameter (the **Jacobian**), then takes a step that blends Gauss–Newton (fast
near a minimum) with gradient descent (safe far from one). A damping factor
sets the blend. It adapts by comparing how much each step actually improved
the fit with how much it was predicted to. This is Nielsen's update, from
Madsen, Nielsen & Tingleff, *Methods for Non-Linear Least Squares Problems*.

- **Scaled parameters.** Internally, each parameter is divided by the size of
  its starting value, so parameters of very different magnitudes (a `Vcmax25`
  of 100 next to an `alpha` of 0.3) are treated evenly.
- **Bounds.** Each parameter has a lower and upper bound, listed in
  [Models and parameters](../reference/models.md). A step that would cross a
  bound is clipped to it. Starting values outside the bounds are clipped too.
- **Derivatives.** Most models use finite differences: each parameter is
  nudged slightly and the change measured. The `lightResponse` model has exact
  derivatives.

## Free and fixed parameters

Each model has a default set of parameters to fit. Others, such as the
Rubisco kinetic constants in FvCB, are **fixed by default** at literature
values. You can change this:

| `pk fit` | Python | Effect |
|---|---|---|
| `--fit a,b` | `fit_params=["a", "b"]` | Fit only these |
| `--fix a,b` | (leave out of `fit_params`) | Hold these at their starting value |
| `--set a=1.5` | `init={"a": 1.5}` | Starting value, or the value a fixed parameter is held at |

The data sets some starting values. For example, soil models start θs and θr
at the wettest and driest measured water contents, and BTA starts `Em` at
least as high as the largest measured conductance.

## Soft constraints

Some models add **penalties** to the objective that pull a fit away from
results that are physically implausible but would otherwise fit the data. An
example is FvCB's requirement that TPU limitation should only appear at high
Ci. A penalty is zero when the condition holds and grows as it's violated. Each
has a weight, and you can turn penalties off or reweight them. See
[Photosynthesis models](photosynthesis.md#soft-constraints).

`r2` and `rmse` measure only how well the model fits your data; they never
include penalties.

## When the fit stops

The fitter stops when one of these happens:

1. The gradient is effectively zero: the fit is at a minimum.
2. The step becomes negligibly small.
3. No step in any direction improves the fit (the fit has **stalled**).
4. It reaches the iteration limit: 200 by default, set with `--max-iter`.

`converged` is `true` for the first three and `false` for the last. A fit
that stops at the limit isn't finished. Its parameters may still be moving, so
raise `--max-iter` and fit again before you use them.

!!! note "Converged doesn't mean correct"
    `converged: true` means the fitter can't improve the fit from where it
    is. It doesn't mean the result is the best possible fit, or that every
    parameter is well determined. Check `r2`, `rmse` and the parameters
    themselves, and compare fits from different starting values (`--set`)
    when it matters.

## Parameters the data can't pin down

A model with many parameters can often fit a dataset almost equally well with
quite different parameter values. When that happens:

- Small changes, such as a slightly different starting value or one extra data
  point, can move the fit to a different combination with the same `rmse`.
- Parameters can end up at their bounds.

This is a property of the model and the data, not an error. It's most common
with the empirical photosynthesis model and with small datasets. To make
parameters better determined:

- **Fix what you know.** For example, `--fix gm` with a measured or assumed
  value, or keep the kinetic constants at their defaults.
- **Fit fewer parameters** with `--fit`.
- **Pool curves** that should share parameters into one fit, using CurveID.
- **Measure over a wider range**, such as more temperatures or more light
  levels.

A parameter reported exactly at its bound (see
[Models and parameters](../reference/models.md)) is a sign the data doesn't
constrain it.
