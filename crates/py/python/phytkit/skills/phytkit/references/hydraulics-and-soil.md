# Hydraulics and soil water retention fits

Part of the `phytkit` skill. `pk describe <model>` gives each model's equation, units and
bounds.

## Vulnerability curves (`pk fit hydraulics`, model `vulnerability`)

K(Ψ) = Kmax / (1 + exp(−S/25·(Ψ − P50))), with Ψ and P50 in MPa (negative) and S in % MPa⁻¹.

- **With a PLC column:** PLC (percent or 0–1 fraction) is converted to relative conductance
  1 − PLC/100 (`relative_conductance`), so `Kmax` is near 1 and dimensionless.
- **With a conductance column** (`K`, `Kleaf`, `Kstem`, ...): it is used as is, and `Kmax`
  takes its units.
- Positive potentials are read as tension and negated (`sign_convention`). Columns named in bar
  are converted to MPa (`unit_converted_by_name`).

## Pressure–volume curves (`pk fit hydraulics --model pvcurve`)

Ψ(w) = πo·max(0, (w − wtlp)/(1 − wtlp))^ε − πo/w, with relative water content w as a fraction
(a percentage above 1.5 is scaled, with `unit_guessed_from_values`). The result's `derived`
section gives `psi_tlp`, the water potential at turgor loss, in MPa.

## Soil water retention (`pk fit soil`)

Models: `vanGenuchten` (default), `modifiedVanGenuchten`, `raats`, `brooksCorey` and
`fractalSWRC`. θ is a volumetric fraction (percentages are scaled).

- **ψ units** come from `--units` (`MPa`, `kPa`, `hPa`, `bar`, `m`, `cm` of water), else from
  the ψ column's name (`Psi (kPa)`, `h_cm`), else MPa with the `soil_units_assumed` warning.
  If the name and `--units` disagree, `--units` wins (`soil_units_overridden`).
- PhytKit converts ψ to MPa before fitting, so α is in MPa⁻¹ and ψb and b\* are in MPa,
  whatever unit the file used.
- Several samples in one file: `--group-by sample` (or the sample column's name) fits each one
  separately.
