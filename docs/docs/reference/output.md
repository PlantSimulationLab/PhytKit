# Output

## Common fields

Every successful JSON output starts with `"ok": true` and `"command"`, the
command name (`"fit"`, `"li600 correct"`, ...). Failures have `"ok": false` and
an `error` object (see [Commands](commands.md)). `NaN` and infinite values
are written as `null`.

## Warnings

Every result that reads data has a `warnings` list. Each entry is an object:

```json
{"code": "parameter_at_bound", "level": "warning", "message": "gm ended on its upper bound (5): the bound, not the data, sets its value."}
```

- `code` is stable: codes are never renamed or removed, like the error codes.
- `level` is `info` for a choice PhytKit made (the columns it used, a documented
  conversion), and `warning` for something that may make the result wrong. Read
  every `warning` before you use the result.
- `message` is for people and may change between versions; match on `code`.

In Python, each `warning`-level entry is also raised as a `phytkit.PhytKitWarning`
whose message starts with `[code]`. `info` entries are only in the list.
`pk describe` lists the codes under `warnings`.

| Code | Level | Meaning |
|---|---|---|
| `columns_mapped` | info | Which file columns were used for each model input (also in `mapping`). |
| `temperature_celsius` | info | Leaf temperatures below 200 were read as °C and converted to kelvin. |
| `curves_detected` | info | Rows were grouped into curves by a CurveID column. |
| `single_curve` | info | No CurveID column: every row is treated as one curve. |
| `sign_convention` | info | Positive water or matric potentials were negated (read as tension or suction). |
| `relative_conductance` | info | Relative conductance K = 1 − PLC/100 was fitted from a PLC column. |
| `unit_converted_by_name` | info | A column was converted because its name states its unit (bar, kPa, cm, ...). |
| `vpd_converted` | info | VPD (kPa) and Ds (mmol mol⁻¹) were converted into each other, using Patm or 101.325 kPa. |
| `observations_parsed` | info | How many LI-600 observations were read, and from which header row. |
| `gsw_corrected_used` | info | The LI-600 gsw_corrected column was used. |
| `labels_disambiguated` | info | Files sharing a label were labelled by their full names so their curves stay apart. |
| `rows_dropped` | warning | Rows with missing or non-numeric values were left out of the fit. |
| `unit_guessed_from_values` | warning | A column's unit was guessed from its values (mmol→mol, mol/mol→mmol/mol, percent→fraction) and converted. |
| `co2_assumed` | warning | No CO2 column: leaf-surface CO2 was assumed to be 420 µmol mol⁻¹. |
| `soil_units_assumed` | warning | No --units given and the ψ column name states no unit: ψ was read as MPa. |
| `soil_units_overridden` | warning | The ψ column name states one unit but --units gave another; --units was used. |
| `gsw_uncorrected_used` | warning | No gsw_corrected column: the uncorrected LI-600 gsw was used. |
| `rows_skipped` | warning | LI-600 rows with a blank gsw_corrected were skipped. |
| `readings_unsolved` | warning | Some LI-600 readings could not be solved and are left out of the summary. |
| `rows_would_drop` | warning | Some merged files lack the column the fitter prefers; their rows would be dropped by pk fit. |
| `not_converged` | warning | The fit stopped before converging; the parameters may not be the best fit. |
| `parameter_at_bound` | warning | A fitted parameter ended on its bound: its value is set by the bound, not the data. |
| `temperature_response_unconstrained` | warning | Temperature-response parameters (*_dHa, *_dHd, *_Topt) were fitted, but leaf temperature spans less than 5 K, so the data cannot determine them, and the 25 °C values depend on what they settle at. |

!!! note "Changed in 0.2.0"
    Before 0.2.0, `warnings` was a list of plain strings, and the fit warnings
    `not_converged` and `parameter_at_bound` did not exist. The old string is now the
    `message` field.

## `pk fit`

### One fit

| Field | Meaning |
|---|---|
| `file` | File name, or `<column> <value>` for a `--group-by` group |
| `converted` | `"xlsx"` or `"li6800-log"` if the file was converted, else `null` |
| `tool`, `model` | What was fitted |
| `n` | Rows fitted |
| `mapping` | File column used for each input |
| `r2`, `rmse` | Fit quality on the data (6 decimals, or 6 significant figures for small values) |
| `iterations`, `converged` | How the fit ended |
| `fitted` | Optimized parameter names |
| `params` | Every parameter's value (6 decimals, or 6 significant figures for small values) |
| `derived` | Values computed from the parameters, such as `psi_tlp` (pressure–volume only) |
| `rescale` | Rescale details, or `null`; see [Rescale curves](../workflows/rescale-to-survey.md) |
| `warnings` | See [Warnings](#warnings): columns used, conversions, dropped rows, no convergence, parameters on a bound |
| `elapsedMs` | Time taken |
| `predicted`, `observed` | With `--predicted`: model and measured values, point by point |

### Batch fits

With more than one file or `--group-by` group (one file, or a `--group-by`
that finds a single group, gives the single-fit object above):

| Field | Meaning |
|---|---|
| `tool` | Tool name |
| `files` | Number of files or groups |
| `groupBy` | The `--group-by` column, or `null` |
| `fitted`, `failed` | How many succeeded and failed |
| `results` | One fit object per success, as above |
| `errors` | `{file, code, error, hint}` for each failure |

### CSV

`--format csv` prints one row per fit. The columns are:

- `file`, then `model` when the files were fitted with more than one model,
  then `n_points`, `r2`, `rmse`, `iterations`, `converged`, `elapsed_ms` and
  `rescale_factor`
- one column per model parameter, covering every model in the batch (cells for
  another model's parameters are empty)
- one column per derived value
- `error`, filled in for files that failed

## `pk li600 correct`

| Field | Meaning |
|---|---|
| `file`, `converted`, `n` | Input and number of readings |
| `solved` | Readings where the correction converged |
| `settings` | `sidedness`, `thermalConductance` |
| `summary` | `meanOverestimatePct`, `maxOverestimatePct`, `maxMeasuredGsw` over solved readings |
| `warnings` | See [Warnings](#warnings) |
| `out` | The `--out` file, or `null` |
| `rows` | With `--rows`: `gsw_measured`, `gsw_corrected`, `overestimate_pct`, `E_corrected_mmol`, `Ta_chamb_corrected`, `T_out_corrected`, `converged` for each reading |

The `--out` file and `--format csv` add these columns, with units:

| Column | Units |
|---|---|
| `gsw_corrected` | mol m⁻² s⁻¹ |
| `E_corrected` | mmol m⁻² s⁻¹ |
| `T_in_corrected`, `Ta_chamb_corrected`, `T_out_corrected` | °C |
| `W_chamb_corrected` | mol mol⁻¹ |
| `stomatal_sidedness` | — |

## `pk li600 estimate`

| Field | Meaning |
|---|---|
| `file`, `converted`, `n`, `solved` | Input, readings, and readings solved |
| `usedCorrectedGsw` | Whether `gsw_corrected` was used |
| `model`, `params` | The model and the full parameter set used |
| `paramsSource`, `ignoredParams` | The `--params` file, and names in it the model doesn't use |
| `settings` | `Ca`, `absorptance`, `minGsw`, `gm` (or `null`), `series` (whether gm was used) |
| `clamped` | Readings raised to `--min-gsw` |
| `summary` | `meanA`, `maxA`, `meanCi`, `meanCiCa` over solved readings |
| `warnings` | See [Warnings](#warnings) |
| `out` | The `--out` file, or `null` |
| `rows` | With `--rows`: `A`, `Ci`, `Cc`, `gsw`, `Qabs`, `Tleaf`, `ETR`, `ok`, `clamped` for each reading |

The `--out` file and `--format csv` add these columns:

| Column | Units |
|---|---|
| `A_solved` | µmol m⁻² s⁻¹ |
| `Ci_solved`, `Cc_solved` | µmol mol⁻¹ |
| `gsw_used` | mol m⁻² s⁻¹ |
| `Qabs` | µmol m⁻² s⁻¹ |

## `pk concat`

With `--out` or `--format json`:

| Field | Meaning |
|---|---|
| `files` | Each input's `name` and `converted` |
| `curves` | Number of distinct CurveIDs in the result |
| `namespaced` | Whether CurveIDs were prefixed with file labels to keep them apart |
| `warnings` | See [Warnings](#warnings); for example, mixed `Qabs`/`Q` columns |
| `out` | The output file, or `null` |
| `rows` | Rows written |
