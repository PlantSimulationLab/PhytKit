# Fit pressure–volume curves

Fit leaf pressure–volume curves to get the osmotic pressure at full turgor,
the turgor loss point, and the water potential at turgor loss. See
[Plant hydraulics models](../concepts/hydraulics.md#pressurevolume-curves).

## Inputs

A file with relative water content (`RWC`, as a fraction or a percentage) and
water potential (`Psi (MPa)`, `Psi`, `WP`, and similar). A column named in bar,
such as `Psi (bar)`, is converted to MPa. Positive tension is made negative.

## Steps

`examples/pv.csv` holds three leaves, with a `Leaf` column:

```bash
pk fit hydraulics examples/pv.csv --model pvcurve --group-by Leaf
```

`--group-by Leaf` fits each leaf separately. The result lists one fit per leaf
under `results`. Each fit has the parameters `πo`, `wtlp` and `ε`, and under
`derived`, the water potential at turgor loss, `psi_tlp`:

```json
{ "file": "Leaf 1", "r2": 0.999985, "params": { "πo": 1.597251, "wtlp": 0.871015, "ε": 1.244022 }, "derived": { "psi_tlp": -1.83378 } }
```

For a table with one row per leaf:

```bash
pk fit hydraulics examples/pv.csv --model pvcurve --group-by Leaf --format csv
```

To fit every leaf as one curve, leave out `--group-by`.
