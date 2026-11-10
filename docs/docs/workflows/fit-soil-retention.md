# Fit soil water retention

Fit a soil water retention curve, θ against matric potential ψ. See
[Soil water retention models](../concepts/soil.md).

## Inputs

A matric potential or suction column (`psi`, `suction`, `h`, `head`, `kPa`,
and similar) and a water content column (`theta`, `θ`, `VWC`, and similar).

ψ's unit comes from `--units` (`MPa`, `kPa`, `hPa`, `bar`, `m` or `cm` of
water); without it, from the unit the ψ column's name states, or else MPa.
Suction reported as positive is fine. Water content as a
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
    ψ is converted to MPa before fitting, so α is reported in MPa⁻¹ and ψb
    and b* in MPa, whatever units the file used. Without `--units`, PhytKit
    uses the unit the ψ column's name states (`Psi (bar)`, `kPa`, `h_cm`, ...)
    and says so in `warnings`. If the name doesn't state one, ψ is read as MPa
    and a warning says so: kPa data in a column named `psi` then fits in the
    wrong units. Pass `--units` whenever your data aren't in MPa.
