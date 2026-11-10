# Stomatal conductance fits

Part of the `phytkit` skill. `pk describe bta` and `pk describe medlyn` give each model's
inputs, units and bounds.

## Input

`pk fit stomatal` reads a conductance column (`gsw_corrected`, `gsw`, `gs`, ... in
mol m⁻² s⁻¹; values above 10 are read as mmol and divided by 1000, with the
`unit_guessed_from_values` warning) plus:

- **BTA (`bta`):** light (`Qin` first, then `Qabs`, `Q`, `PARi`, ...) and either `Ds` in
  mmol mol⁻¹ or VPD in kPa. VPD is converted with Ds = 1000·VPD/Patm, using a `Patm` column
  or 101.325 kPa (`vpd_converted`).
- **Medlyn (`medlyn`):** `A`, `Cs` (or `Ca`, `CO2`) and VPD in kPa. Without a CO₂ column,
  Cs is assumed to be 420 µmol mol⁻¹ (`co2_assumed`).

Without `--model`, the first model whose reader accepts the file is used: `bta`, then
`medlyn`. Check `model` and `mapping` in the result.

## BTA

gs = Em·(Q + i0) / (k + b·Q + (Q + i0)·Ds). `Em` is a transpiration rate in mmol m⁻² s⁻¹,
`i0` is in µmol m⁻² s⁻¹, `k` in µmol m⁻² s⁻¹ · mmol mol⁻¹, and `b` in mmol mol⁻¹. Em, k and b
trade off against one another, so check `parameter_at_bound` and look at `--predicted` across
the light and VPD range.

## Medlyn

gs = gs0 + 1.6·(1 + g1/√D)·A/Cs, with D in kPa (floored at 1e-5), gs0 in mol m⁻² s⁻¹ and g1 in
kPa^0.5.

## LI-600 data

Correct LI-600 conductance first with `pk li600 correct`; the corrected file's
`gsw_corrected` column is then used automatically. See [li600.md](li600.md).
