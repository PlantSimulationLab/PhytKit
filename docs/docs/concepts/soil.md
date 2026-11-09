# Soil water retention models

The `soil` tool fits volumetric water content **θ** as a function of matric
potential **ψ**. All five models share two parameters:

- **θr:** the residual water content.
- **θs:** the saturated water content.

PhytKit starts them at the driest and wettest measured θ. ψ is in MPa unless
you say otherwise with `--units` (`kPa`, `bar`, `m` or `cm` of water).
Suction given as positive is treated as negative ψ, and water content given as
a percentage is converted to a fraction.

| Model | `--model` | Shape |
|---|---|---|
| van Genuchten (1980) | `vanGenuchten` (default) | \(\theta = \theta_r + (\theta_s - \theta_r)\,[1 + (\alpha\lvert\psi\rvert)^n]^{-m}\), with \(m = 1 - 1/n\) |
| Modified van Genuchten (Troch 1993) | `modifiedVanGenuchten` | The same form with \(m = 1 + 1/n_1\) |
| Raats superclass (Raats 1992; Heinen & Bakker 2016) | `raats` | \(\lvert\psi\rvert = b^*\,[(1 - S^\alpha)/S^\beta]^\gamma\), solved numerically for \(S\) |
| Brooks–Corey (1964) | `brooksCorey` | \(S_e = (\psi_b/\lvert\psi\rvert)^\lambda\) drier than air entry, saturated wetter |
| Fractal (Li et al. 2025) | `fractalSWRC` | Suction from fractal pore structure, with an adsorbed film |

The Raats superclass contains van Genuchten, Brooks–Corey and others as
special cases. The fractal model has a fractal dimension `D` between 1 and 2,
a maximum pore radius `r_max` in µm, and an adsorbed film fraction `δ`.

Some models also carry conductivity parameters that retention data alone
can't determine. Examples are `α2` and `n2` in the modified van Genuchten
model, `λ` in the Raats model, and the tortuosity `L`. These are fixed by default.

See [Fit soil water retention](../workflows/fit-soil-retention.md) and every
parameter in [Models and parameters](../reference/models.md#vangenuchten).
