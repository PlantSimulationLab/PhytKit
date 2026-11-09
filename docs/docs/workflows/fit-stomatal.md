# Fit stomatal conductance

Fit the Buckley–Turnbull–Adams or Medlyn stomatal conductance model. See
[Stomatal conductance models](../concepts/stomatal.md).

## BTA: conductance, light and VPD

The file needs conductance (`gsw`, `gs`, `gsw_corrected`, and similar), light
(`Qin`, `Qabs`, `Q`, `PARi`, and similar) and VPD (`VPDleaf`, `VPD`, and
similar, in kPa) or `Ds` (mmol mol⁻¹):

```bash
pk fit stomatal examples/bta.csv
```

## Medlyn: conductance, assimilation, CO₂ and VPD

The file needs conductance, net assimilation (`A`, `A_solved`, `Anet`, and
similar) and VPD. A CO₂ column (`Cs`, `Ca`, `CO2`, and similar) is optional;
without one, Cs = 420 µmol mol⁻¹.

```bash
pk fit stomatal examples/medlyn.csv --model medlyn --predicted
```

## From porometry

Correct the LI-600 file, estimate A, and fit Medlyn to the result. The file
then has `gsw_corrected`, `A_solved` and `VPDleaf`. See
[Estimate A from porometry](estimate-assimilation.md).
