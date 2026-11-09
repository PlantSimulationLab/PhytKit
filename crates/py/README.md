# PhytKit

Plant physiology model fitting, written in Rust: photosynthesis (FvCB C3 and C4, empirical light
response), stomatal conductance (BTA, Medlyn), xylem vulnerability, pressure–volume curves, soil
water retention, and LI-600 porometer tools.

```bash
pip install phytkit
```

This installs the `phytkit` Python package and the `pk` command-line tool. Both return the same
JSON.

```bash
pk describe fvcb
pk fit photosynthesis leaf.csv --model fvcb
pk fit soil soil.csv --group-by sample --units kPa
pk li600 correct porometer.csv --sidedness 1.5
```

```python
import phytkit as pk

pk.fit("photosynthesis", "leaf.csv", model="fvcb", fix=["gm"], max_iter=400)  # same as `pk fit`
pk.fit_arrays("fvcb", {"Ci": ci, "Q": q, "T": t_kelvin}, a)                   # data in memory
pk.fit_many("fvcb", [(x1, y1), (x2, y2)], threads=8)                        # parallel batch
pk.predict("fvcb", {"Vcmax25": 90}, {"Ci": ci, "Q": q, "T": t_kelvin})
```

Failures raise `phytkit.PhytKitError`, with a stable `.code` and a `.hint`.

- Documentation: <https://plantsimulationlab.github.io/PhytKit/>
- Source and issues: <https://github.com/PlantSimulationLab/PhytKit>

Originally developed primarily by Kyle Rizzo ([@ktrizzo](https://github.com/ktrizzo)); maintained by the PlantSimulationLab. MIT licensed.
