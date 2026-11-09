# Merge LI-6800 logs

Pool several LI-6800 logs or CSVs into one file, so you can fit them together
with a CurveID per curve.

```bash
pk concat log1 log2 log3 --out merged.csv
pk fit photosynthesis merged.csv --model fvcb
```

LI-6800 native logs, `.xlsx` exports and plain CSVs can be mixed. Each is
reduced to a plain table first (the LI-6800 header block and units row are
removed). The rows are then written under one header containing every column
from every file.

## Curve IDs

- **No file has a CurveID column:** one is added, set to each file's label
  (its name without the LI-6800 date-and-time prefix or extension).
- **Files already have CurveIDs:** they're kept, unless two files use the same
  value. LI-6800 logs often all start at 1. In that case every value is
  prefixed with its file's label, so curves from different files stay apart,
  and the result reports `"namespaced": true`.

## Output

Without `--out`, the merged CSV is printed. With `--out`, it's written to the
file, and `pk` prints a summary with:

- `files`: each input and whether it was converted
- `curves`: distinct CurveIDs
- `rows`: rows written
- `warnings`

```bash
pk concat examples/aci.csv examples/messy_aci.csv --format json
```

!!! warning "Mixed light columns"
    If some files log `Qabs` and others only `Q`, the merged file has both
    columns. The fitter prefers `Qabs`, so rows that only have `Q` would be
    dropped as incomplete. `pk concat` warns when this would happen. Merge
    logs recorded the same way.
