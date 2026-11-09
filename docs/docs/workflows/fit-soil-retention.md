# Fit soil water retention

Fit a soil water retention curve, θ against matric potential ψ. See
[Soil water retention models](../concepts/soil.md).

## Inputs

A matric potential or suction column (`psi`, `suction`, `h`, `head`, `kPa`,
and similar) and a water content column (`theta`, `θ`, `VWC`, and similar).

ψ is read in MPa unless you give `--units`: `MPa`, `kPa`, `bar`, `m` or `cm`
(of water). Suction reported as positive is fine. Water content as a
percentage is converted to a fraction.

## Steps

`examples/soil.csv` holds two soils with ψ in kPa:

```bash
pk fit soil examples/soil.csv --group-by sample --units kPa
```

That fits van Genuchten, the default. To compare models:

```bash
pk fit soil examples/soil.csv --group-by sample --units kPa --model brooksCorey
pk fit soil examples/soil.csv --group-by sample --units kPa --model raats
pk fit soil examples/soil.csv --group-by sample --units kPa --model fractalSWRC
pk fit soil examples/soil.csv --group-by sample --units kPa --model modifiedVanGenuchten
```

Compare `rmse` across models, but prefer the simplest model that fits well.
Models with more shape parameters can fit closely while leaving those
parameters poorly determined.

!!! warning "Get the units right"
    α, ψb and b* are in units of the potential. If you give kPa data without
    `--units kPa`, the fit runs in the wrong units without any error. The
    `warnings` don't say which unit was assumed, so always pass `--units`
    when your data aren't in MPa.
