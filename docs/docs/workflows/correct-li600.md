# Correct LI-600 conductance

Apply the Rizzo & Bailey (2025) energy-balance correction to LI-600 porometer
readings. See [LI-600 porometry](../concepts/li600.md) for why the correction
is needed.

## Inputs

An LI-600 CSV export. PhytKit finds the column-name row itself, below the
instrument's category row, and skips the units row. It needs these columns:
`gsw`, `Tref`, `Tleaf`, `rh_r`, `rh_s`, `flow`, `P_atm` and `E_apparent`.

## Steps

1. Correct, with the sidedness that fits your leaves: 1 for stomata on one
   side, 2 for both sides.

    ```bash
    pk li600 correct examples/li600.csv --sidedness 1
    ```

    The `summary` gives the mean and largest overestimate, in percent, over
    readings that solved:

    ```json
    "summary": { "meanOverestimatePct": 8.637, "maxOverestimatePct": 14.787, "maxMeasuredGsw": 0.5856 }
    ```

2. Write the original file with the corrected columns added. Use this file for
   the next steps:

    ```bash
    pk li600 correct examples/li600.csv --sidedness 1 --out li600_corrected.csv
    ```

    `li600_corrected.csv` keeps every original row and column and adds
    `gsw_corrected`, `E_corrected`, `T_in_corrected`, `Ta_chamb_corrected`,
    `T_out_corrected`, `W_chamb_corrected` and `stomatal_sidedness`, with
    their units in the units row.

3. Optionally, look at individual readings with `--rows`, or print only the
   new columns with `--format csv`:

    ```bash
    pk li600 correct examples/li600.csv --rows
    pk li600 correct examples/li600.csv --format csv
    ```

The correction's overestimate grows with conductance. In the sample file it
runs from about 2% at gsw 0.1 to about 11–13% near 0.5 and nearly 15% at the
highest readings (about 0.55–0.59 mol m⁻² s⁻¹).

## Next

[Estimate A from porometry](estimate-assimilation.md) uses `gsw_corrected`
automatically when the file has it.
