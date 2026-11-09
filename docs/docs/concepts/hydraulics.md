# Plant hydraulics models

The `hydraulics` tool fits two kinds of curve: xylem **vulnerability** to
embolism, and leaf **pressure–volume** curves.

Water potentials are negative MPa throughout. PhytKit converts the common
alternatives and says so in `warnings`:

- Potentials reported as positive tension are made negative.
- Columns named in bar are converted to MPa.
- Percentages are converted to fractions.

## Vulnerability curves

`--model vulnerability`. The sigmoidal model of Pammenter & Van der Willigen
(1998), fitted in conductance space so the maximum conductance is a free
parameter:

\[
K(\Psi) = \frac{K_{max}}{1 + \exp\!\left[-\frac{S}{25}(\Psi - P_{50})\right]}
\]

| Parameter | Meaning |
|---|---|
| `Kmax` | Conductance at full hydration |
| `P50` | Water potential at 50% loss of conductivity (MPa) |
| `S` | Slope at P50 (% MPa⁻¹) |

The equivalent percent loss of conductivity is
\(PLC = 100\,(1 - K/K_{max})\), the "sigmoidal" model of the fitplc package.

Your file can give either:

- **PLC**, which is converted to relative conductance \(1 - PLC/100\), so
  `Kmax` is near 1. Fractions (0–1) are recognized as well as percentages.
- **A conductance column** (`K`, `Kleaf`, `Kstem` and similar), used as is.
  `Kmax` then starts at the largest measured value.

## Pressure–volume curves

`--model pvcurve`. Leaf water potential as a function of relative water
content \(w\) (Sack, John & Buckley 2018; Tyree & Hammel 1972):

\[
\Psi(w) = \pi_o\left[\max\!\left(0,\ \frac{w - w_{tlp}}{1 - w_{tlp}}\right)\right]^{\varepsilon} - \frac{\pi_o}{w}
\]

The first term is **turgor pressure**, which falls to zero at the turgor loss
point \(w_{tlp}\). The second is the **osmotic potential**. At full hydration
(\(w = 1\)) they cancel, and \(\Psi = 0\).

| Parameter | Meaning |
|---|---|
| `πo` | Osmotic pressure at full turgor (MPa, positive) |
| `wtlp` | Relative water content at turgor loss |
| `ε` | How turgor relaxes as the leaf dries above the turgor loss point (a proxy for bulk wall elasticity) |

The result also reports a **derived** value: the water potential at turgor
loss, \(\Psi_{tlp} = -\pi_o / w_{tlp}\), as `psi_tlp`.

Relative water content may be a fraction or a percentage.

See [Fit pressure–volume curves](../workflows/fit-pv-curves.md) and
[Fit a vulnerability curve](../workflows/fit-vulnerability.md).
