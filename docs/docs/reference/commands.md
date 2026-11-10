# Commands

All commands print one JSON object to standard output, unless noted. On
failure they print an error object and exit with code 1 (see [Errors](#errors)).
Add `--help` to any command, or run `pk help <command>`, for its arguments and
options.

Options take a value as `--name value` or `--name=value`. Yes/no options are
given alone (`--rows`), and `--no-rows` or `--rows=false` turns one off.
Options marked *repeatable* can be given several times, and list options
accept comma-separated values. Unknown options, values of the wrong type or
out of range, extra arguments, and an option given twice with different
values are all errors.

## `pk describe`

```bash
pk describe            # every tool, model and the li600 options
pk describe <model>    # one model: inputs, response and parameters (with units), bounds, constraints, equation
```

## `pk fit`

```bash
pk fit <tool> <file...> [options]
```

`<tool>` is `photosynthesis`, `stomatal`, `hydraulics` or `soil`. With several
files, or with `--group-by`, each file or group is fitted separately (see
[Output](output.md#batch-fits)).

| Option | Value | Meaning |
|---|---|---|
| `--model` | model name | Model to fit. Defaults: `fvcb`, `bta`, `vulnerability`, `vanGenuchten`. If you leave it out for `stomatal`, `hydraulics` or `soil`, the first model whose parser reads the file is used. |
| `--fit` | names, repeatable | Fit exactly these parameters (replaces the default set) |
| `--fix` | names, repeatable | Hold these parameters fixed |
| `--set` | `name=value`, repeatable | Starting value (must be within the parameter's bounds), or the value a fixed parameter is held at (any value) |
| `--max-iter` | whole number ≥ 1 | Iteration limit (default 200) |
| `--no-constraint` | key, repeatable | Turn off one soft constraint |
| `--constraint` | key, repeatable | Turn on a soft constraint that is off by default |
| `--constraints` | `none` | Turn off every soft constraint |
| `--weight` | `key=value`, repeatable | Weight for one soft constraint |
| `--units` | `MPa`, `kPa`, `hPa`, `bar`, `m`, `cm` | Units of ψ (`soil` only). Without it: the unit the ψ column's name states, else MPa |
| `--group-by` | column | Split one file into a fit per value of the column |
| `--survey` | file, repeatable | Rescale mode: survey points to rescale A to (`photosynthesis` only) |
| `--stat` | `mean`, `median`, `max` | Statistic for rescaling (default `mean`) |
| `--ref-ci` | number | Reference Ci for rescaling (default: the highest Ci in the curve) |
| `--predicted` | — | Include predicted and observed values in the output |
| `--format` | `json`, `csv` | `csv` prints one row per fit |

## `pk li600 correct`

```bash
pk li600 correct <file|-> [options]
```

| Option | Value | Meaning |
|---|---|---|
| `--sidedness` | number | 1 (default) for stomata on one side, 2 for both |
| `--thermal-conductance` | number | Instrument heat exchange, W °C⁻¹ (default 0.007) |
| `--rows` | — | Include every reading in the JSON |
| `--out` | file | Also write the original file with the corrected columns added |
| `--format` | `json`, `csv` | `csv` prints only the corrected columns |

## `pk li600 estimate`

```bash
pk li600 estimate <file|-> [options]
```

| Option | Value | Meaning |
|---|---|---|
| `--model` | `fvcb`, `fvcbC4`, `empiricalA` | Photosynthesis model. Default: the model a `--params` fit JSON was fitted with, else `fvcb` |
| `--params` | file | Parameter file: a single `pk fit` JSON result (a batch result is rejected; pass one entry of its `results`), a CSV of fitted parameters, a Parameter/Value table, or `name = value` lines |
| `--set` | `name=value`, repeatable | Override one parameter |
| `--ca` | number | Ambient CO₂, µmol mol⁻¹ (default 420) |
| `--absorptance` | number | Leaf absorptance applied to `Qamb` (default 0.85) |
| `--min-gsw` | number | Conductance floor (default 0.001); `0` leaves lower readings unsolved |
| `--rows` | — | Include every reading in the JSON |
| `--out` | file | Also write the original file with the solved columns added |
| `--format` | `json`, `csv` | `csv` prints only the solved columns |

## `pk concat`

```bash
pk concat <file...> [options]
```

| Option | Value | Meaning |
|---|---|---|
| `--out` | file | Write the merged CSV to a file, and print a JSON summary |
| `--format` | `json`, `csv` | Default `csv` (the merged file) without `--out`, `json` with it |

## Errors

```json
{
  "ok": false,
  "command": "fit",
  "error": {
    "code": "unknown_parameter",
    "message": "--fix Vcmax: fvcb has no parameter \"Vcmax\".",
    "usage": true,
    "hint": "Did you mean Vcmax25? Parameters: Vcmax25, Jmax25, ...",
    "help": "pk fit <tool> <file...> [options]"
  }
}
```

`hint` is present when there's a specific fix to suggest, and `help` (the
command's usage line) is present on usage errors for a known command (not on
`unknown_command`). `usage` is `true` when the command line was wrong
and `false` when a file or its data was. `pk describe` lists these codes too,
under `errors`.

| Code | `usage` | Meaning |
|---|---|---|
| `unknown_command` | true | No such command |
| `unknown_option` | true | The command has no such option |
| `missing_argument` | true | A required argument, such as the input file, is missing |
| `too_many_arguments` | true | More arguments than the command takes |
| `missing_value` | true | An option that needs a value was given without one |
| `invalid_value` | true | An option's value is the wrong type or out of range |
| `duplicate_option` | true | An option that takes one value was given twice with different values |
| `invalid_combination` | true | Options or arguments that can't be used together |
| `not_applicable` | true | An option that doesn't apply to this tool or model |
| `unknown_tool` | true | `pk fit` was given a tool it doesn't have |
| `unknown_model` | true | No such model, or not one for this tool |
| `unknown_parameter` | true | The model has no parameter by that name |
| `unknown_constraint` | true | The model has no soft constraint by that name |
| `nothing_to_fit` | true | Every parameter is fixed |
| `missing_input` | true | Data passed from Python lacks one of the model's input columns |
| `file_not_found` | false | An input file doesn't exist |
| `not_a_file` | false | An input path is a folder |
| `permission_denied` | false | An input file can't be read |
| `read_failed` | false | An input file couldn't be read for another reason |
| `write_failed` | false | An `--out` file couldn't be written |
| `invalid_data` | false | A file was read but doesn't hold the data the command needs |
| `rescale_failed` | false | Rescale mode couldn't compute a factor from the survey and curve data |

## Input from standard input

Give `-` as the file to read from standard input, for example
`cat readings.csv | pk li600 correct -`.
