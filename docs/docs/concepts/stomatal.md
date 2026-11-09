# Stomatal conductance models

The `stomatal` tool fits stomatal conductance to water vapour, **gs** (mol
m⁻² s⁻¹), as measured by the LI-600 or LI-6800 (`gsw`). If the file has a
`gsw_corrected` column from [the LI-600 correction](li600.md), PhytKit uses it
instead of `gsw`.

## Buckley–Turnbull–Adams (2012)

`--model bta` (default). A compact form of the Buckley–Mott–Farquhar
hydromechanical model, driven by light and the vapour pressure deficit:

\[
g_s = \frac{E_m\,(Q + i_0)}{k + b\,Q + (Q + i_0)\,D_s}
\]

- **Q:** photosynthetic photon flux density.
- **Ds:** leaf-to-air vapour pressure deficit as a mole fraction, in mmol mol⁻¹.

If your file has VPD in kPa, PhytKit converts it with
\(D_s = 1000\,\mathrm{VPD}/P_{atm}\). It uses a `Patm` column if present, and
101.325 kPa otherwise. The parameters `Em`, `i0`, `k` and `b` are empirical.

## Medlyn et al. (2011)

`--model medlyn`. The optimal stomatal model: the conductance that maximizes
carbon gain per unit water lost.

\[
g_s = g_{s0} + 1.6\left(1 + \frac{g_1}{\sqrt{D}}\right)\frac{A}{C_s}
\]

- **A:** net assimilation.
- **Cs:** CO₂ at the leaf surface.
- **D:** vapour pressure deficit, in kPa.

The 1.6 converts a CO₂ conductance to water vapour. D is floored at 1e-5 kPa,
because the model diverges as D approaches zero.

The LI-600 doesn't measure CO₂, so a file without a CO₂ column uses
Cs = 420 µmol mol⁻¹. That's the same ambient CO₂ the
[A estimate](li600.md#estimating-a-from-conductance) assumes, so you can fit
Medlyn to the output of `pk li600 estimate`, which writes `A_solved`.

## Units

Conductance above 10 is taken to be in mmol m⁻² s⁻¹ and divided by 1000,
because leaf conductance in mol m⁻² s⁻¹ rarely exceeds 2. PhytKit reports
every conversion it makes in `warnings`.

See [Fit stomatal conductance](../workflows/fit-stomatal.md).
