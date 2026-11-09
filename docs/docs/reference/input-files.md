# Input files

Every tool reads CSV files with one header row, `.xlsx` workbooks (first
sheet), LI-6800 native logs, and standard input (`-`). Columns are found by
name, so files from different sources work without renaming. Each tool reports
the columns it used, and any conversion it made, in `warnings`.

## Photosynthesis

| Input | Column names tried, in order | Notes |
|---|---|---|
| A | `A` | µmol m⁻² s⁻¹ |
| Ci | `Ci` | µmol mol⁻¹ |
| Light | `Qabs`, `Q` | Absorbed PPFD; `Qabs` wins when both are present |
| Temperature | `Tleaf`, `T` | Values below 200 are read as °C and converted to Kelvin |
| Curve | `CurveID`, `curve` | Optional; groups rows into curves for per-curve constraints |

Names are matched ignoring case. Rows missing any of A, Ci, light or
temperature are dropped and counted in `warnings`.

LI-6800 files are recognized automatically:

- **`.xlsx` exports:** Phytkit reads the `Measurements` sheet and finds the row
  holding both `A` and `Ci`.
- **Native logs:** it reads the `[Data]` block.

In both, it skips the category and units rows and keeps the data rows.

## Stomatal conductance

| Input | Column names tried | Notes |
|---|---|---|
| Conductance | `gsw_corrected`, `gsw`, `gs`, `gtw`, `g_sw`, `gvw`, `stomatal_conductance`, `conductance`, `cond`, `gs_w` | mol m⁻² s⁻¹; values above 10 are read as mmol and divided by 1000 |
| Light (BTA) | `Qin`, `Qabs`, `Q`, `PARi`, `PPFD`, `PAR`, `Qamb`, `PFD`, `light`, `Qi`, `PARin` | |
| VPD | `VPDleaf`, `VPDl`, `VpdL`, `VPD`, `VpdA`, `SVPleaf`, `leaf_vpd`, `vpd_leaf`, `VPDL`, `D`, `Dleaf` | kPa |
| Ds | `Ds`, `D_s` | mmol mol⁻¹; used directly by BTA |
| Pressure | `Pa`, `Patm`, `P_atm`, `pressure`, `atm_pressure`, `press`, `Patm_kPa` | kPa; 101.325 if absent |
| A (Medlyn) | `A`, `A_solved`, `Anet`, `A_net`, `Photo`, `assimilation`, `An` | |
| CO₂ (Medlyn) | `Cs`, `C_s`, `CO2_s`, `Ca`, `CO2_a`, `CO2`, `Ca_s` | 420 µmol mol⁻¹ if absent |

BTA converts VPD to Ds with \(D_s = 1000\,\mathrm{VPD}/P_{atm}\); Medlyn
converts Ds to kPa the other way. Files from the LI-600 and LI-6800 are read
directly: Phytkit finds the column-name row and skips the units row.

## Plant hydraulics

| Input | Column names tried | Notes |
|---|---|---|
| Water potential | `P`, `Psi (MPa)`, `Psi(MPa)`, `WP (MPa)`, `WP(MPa)`, `Psi_MPa`, `Psi`, `psi`, `WP`, `water_potential`, `potential`, `MPa`, `Pressure`, `P_MPa`, `Pa`, then `Psi (bar)`, `Psi(bar)`, `WP (bar)`, `Psi_bar` | MPa. Columns named in bar are converted. Positive tension is made negative. |
| PLC | `PLC`, `plc`, `loss`, `percent_loss`, `PLC_pct`, `PLCpercent`, `pld` | Percent, or a 0–1 fraction |
| Conductance | `K`, `k`, `Kleaf`, `Kstem`, `Kroot`, `Kplant`, `Kmax`, ... with a space, `_` or `-` separator; `conductance`, `conductivity` | Used when there's no PLC column |
| Relative water content | `RWC`, `rwc`, `RWC (%)`, `relative_water_content`, `water_content`, `RWC_pct`, `w` | Fraction, or a percentage (above 1.5) |

Names are matched loosely, ignoring case, spaces, underscores and unit
punctuation. So `water potential`, `water_potential` and `WaterPotential` are
the same.

## Soil water retention

| Input | Column names tried | Notes |
|---|---|---|
| Matric potential | the water-potential names above, plus `matric_potential`, `matric`, `suction`, `tension`, `pressure_head`, `head`, `h`, `kPa`, `hPa`, `cm`, `psi_MPa` | MPa unless `--units` says otherwise. Positive suction is accepted. |
| Water content | `theta`, `θ`, `VWC`, `vwc`, `theta_v`, `SWC`, `swc`, `volumetric_water_content`, `water_content`, `moisture`, `VMC`, `WC` | Fraction, or a percentage (above 1.5) |

## LI-600 files

`pk li600 correct` needs `gsw`, `Tref`, `Tleaf`, `rh_r`, `rh_s`, `flow`,
`P_atm` and `E_apparent`. `pk li600 estimate` needs `gsw` (or
`gsw_corrected`), `Qamb` and `Tleaf`, and keeps `ETR` if present. Both find the
column-name row within the first five rows and skip a units row below it.
