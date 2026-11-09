# Using the command line

`pk` has four commands:

| Command | What it does |
|---|---|
| `pk describe [model]` | Lists tools, models, parameters, bounds and constraints |
| `pk fit <tool> <file...>` | Fits a model: `photosynthesis`, `stomatal`, `hydraulics` or `soil` |
| `pk li600 correct <file>` | Corrects LI-600 stomatal conductance for evaporative cooling |
| `pk li600 estimate <file>` | Solves A, Ci and Cc from LI-600 conductance |
| `pk concat <file...>` | Merges LI-6800 logs or CSVs, with a CurveID per file |

`pk --help` prints this list, and `pk <command> --help` prints a command's
options. [Commands](../reference/commands.md) documents every option.

## Input files

`pk` reads:

- **CSV files** with one header row of column names.
- **`.xlsx` workbooks**, using the first sheet. For photosynthesis, the
  LI-6800's `Measurements` sheet.
- **LI-6800 native logs**, the files with no extension that the instrument
  writes. PhytKit finds the `[Data]` block and the column header itself.
- **Standard input**, if you give `-` as the file name.

PhytKit finds the columns it needs by name and accepts common variants, such
as `Psi (MPa)` or `water_potential` for water potential. It also converts
common units, such as °C to Kelvin, bar to MPa, and a percent to a fraction.
It reports each choice in `warnings`. See
[Input files](../reference/input-files.md).

## Options

Options follow the command: `--name value`, `--name=value`, or `--flag` alone
for a yes/no option. A few options can be repeated, such as `--set` and
`--survey`. Lists can be comma-separated: `--fix gm,alphaG`.

`pk` checks every option before it runs anything. A misspelled or unknown
option, a value of the wrong type, or an extra argument is an error, never
silently ignored. Run `pk <command> --help` to see a command's options.

The options you'll use most with `pk fit`:

| Option | Example | Effect |
|---|---|---|
| `--model` | `--model fvcbC4` | Model to fit (each tool has a default) |
| `--fit` | `--fit Vcmax25,Jmax25` | Fit only these parameters |
| `--fix` | `--fix gm,alphaG` | Hold these parameters fixed |
| `--set` | `--set Vcmax25=80` | Starting value, or the fixed value of a fixed parameter |
| `--max-iter` | `--max-iter 400` | Iteration limit (default 200) |
| `--group-by` | `--group-by Leaf` | Split one file into one fit per value of a column |
| `--format` | `--format csv` | Print CSV instead of JSON |

## Output and errors

Every command prints **one JSON object** to standard output. It has
`"ok": true` and the results, unless you asked for `--format csv`.

When something goes wrong, `pk` prints an error object and exits with code 1:

```json
{
  "ok": false,
  "command": "li600 correct",
  "error": {
    "code": "unknown_option",
    "message": "Unknown option --sidednes for pk li600 correct.",
    "usage": true,
    "hint": "Did you mean --sidedness? Options: --sidedness, --thermal-conductance, --rows, --out, --format.",
    "help": "pk li600 correct <file> [options]"
  }
}
```

- `code` names the kind of error, such as `unknown_option` or
  `file_not_found`. Codes don't change between versions, so scripts can
  branch on them. They're all listed in [Commands](../reference/commands.md#errors).
- `message` says what was wrong.
- `hint`, when present, says what to do: the option, model, parameter or
  tool you probably meant, and the valid choices.
- `usage` is `true` when the command line was wrong, so fixing the arguments
  and retrying will help. It's `false` when a file or its data was the
  problem.
- `help`, on usage errors, is the command's usage line.

## For scripts and coding agents

`pk` is designed to be driven by other programs:

- It never asks questions or waits for input.
- `pk describe` and `pk describe <model>` say what each model takes, so a
  script doesn't need to read these docs.
- Output is always JSON, or CSV when you ask for it.
- Errors have a stable `code`, and a `hint` naming the likely fix. Nothing is
  silently ignored: an unknown option, a stray argument or an out-of-range
  value stops the command before it runs.
- `pk describe` also lists every command's options, with types, defaults and
  choices, and every error code.
- `--format csv` and `--out` write tables you can open directly.
