# Using PhytKit parameters in Helios

Part of the `phytkit` skill. Read it before handing any fitted value to Helios's
`photosynthesis` or `stomatalconductance` plugin, from C++ or through PyHelios.

PhytKit fits; Helios simulates. They share most equations but not all of their
conventions. Convert in the code that builds the Helios coefficients, not by editing
the fitted values.

## FvCB C3 (`fvcb` → `FarquharModelCoefficients`)

Fit with `--ci-based` (`ci_based=True`). Helios uses Cc = Ci unless its mesophyll
conductance is set, and it has no `alphaG`; `--ci-based` fits in that form:

```bash
pk fit photosynthesis curves.csv --model fvcb --ci-based
```

Then pass every value through the four-argument setters, converting `*_Topt` from
kelvin to °C:

| PhytKit parameter (units) | Helios call | Conversion |
|---|---|---|
| `Vcmax25`, `Vcmax_dHa`, `Vcmax_Topt`, `Vcmax_dHd` | `setVcmax(Vcmax25, dHa, Topt_C, dHd)` | `Topt_C = Vcmax_Topt − 273.15` |
| `Jmax25`, `Jmax_dHa`, `Jmax_Topt`, `Jmax_dHd` | `setJmax(Jmax25, dHa, Topt_C, dHd)` | `Topt_C = Jmax_Topt − 273.15` |
| `TPU25`, `TPU_dHa`, `TPU_Topt`, `TPU_dHd` | `setTPU(TPU25, dHa, Topt_C, dHd)` | `Topt_C = TPU_Topt − 273.15`. Without `setTPU`, Helios has no TPU limit at all |
| `Rd25`, `Rd_dHa` | `setRd(Rd25, Rd_dHa)` | none (both are Arrhenius) |
| `alpha` | `setQuantumEfficiency_alpha(alpha)` | none |
| `theta` | `setLightResponseCurvature_theta(theta)` | none. **Always pass it**: Helios's C++ default is 0 and PyHelios's is 0.7 |
| `gm` (`null` after `--ci-based`) | leave unset | Helios's default gm is infinite |
| `alphaG` | — | must be 0 (Helios's TPU limit is 3·TPU − Rd); `--ci-based` holds it there |

Units are the same on both sides: rates in µmol m⁻² s⁻¹, dHa and dHd in kJ mol⁻¹, gm in
mol m⁻² s⁻¹ bar⁻¹. Topt is kelvin in PhytKit and °C in Helios. Helios rejects a Topt
outside 0–100 °C ("provided in Kelvin instead of Celsius") and a dHd that isn't above dHa.

```cpp
FarquharModelCoefficients c;
c.setVcmax(p["Vcmax25"], p["Vcmax_dHa"], p["Vcmax_Topt"] - 273.15, p["Vcmax_dHd"]);
c.setJmax(p["Jmax25"], p["Jmax_dHa"], p["Jmax_Topt"] - 273.15, p["Jmax_dHd"]);
c.setTPU(p["TPU25"], p["TPU_dHa"], p["TPU_Topt"] - 273.15, p["TPU_dHd"]);
c.setRd(p["Rd25"], p["Rd_dHa"]);
c.setQuantumEfficiency_alpha(p["alpha"]);
c.setLightResponseCurvature_theta(p["theta"]);
photosynthesis.setModelType_Farquhar();
photosynthesis.setModelCoefficients(c);
```

PyHelios's `FarquharModelCoefficients` setters take the same arguments in the same
order, with Topt in °C. Check two things in the PyHelios version you have:

- whether it passes TPU to Helios at all;
- whether a one-argument setter really gives a constant. Some versions attach dHa = 60,
  so pass every argument.

### If the curves were fitted one at a time

A curve measured at one leaf temperature cannot determine the temperature parameters.
PhytKit warns `temperature_response_unconstrained`. Fit only
`--fit Vcmax25,Jmax25,TPU25,Rd25,alpha,theta`, then pass the defaults that were held
(see `pk describe fvcb`) to the Helios setters along with the fitted values. A Vcmax25
means something only together with the temperature response that scaled it to 25 °C.

### Temperature response: identical form

Both evaluate a peaked Arrhenius normalized to 25 °C, with R = 0.0083144598 kJ mol⁻¹ K⁻¹
and T in kelvin:

    k(T) = k25 · exp(dHa/R · (1/298.15 − 1/T))
               · [1 + exp(dHd/R · (1/Topt − 1/298.15) − ln(dHd/dHa − 1))]
               / [1 + exp(dHd/R · (1/Topt − 1/T)      − ln(dHd/dHa − 1))]

Helios's two-argument `setRd(Rd25, dHa)` sets dHd = 10·dHa and Topt = 10000 K, which is
plain Arrhenius to float precision, the same as PhytKit's Rd.

### What differs, and by how much

- **Γ\*, Kc and Ko.** Helios hard-codes Bernacchi et al. (2001): Γ\* = 42.93 µmol mol⁻¹,
  Kc = 406.79 µmol mol⁻¹ and Ko = 277.12 mmol mol⁻¹ at 25 °C, with the same activation
  energies and O = 213.5 mmol mol⁻¹. It ignores the struct's `c_*`, `dH_*` and `O` fields.
  PhytKit's fixed defaults are 42.75, 404.9 and 278.4, within 0.5 %. To fit with exactly
  Helios's values, add `--set GammaS25=42.93 --set Kc25=406.79 --set Ko25=277.12`. These
  parameters are fixed, so any value is accepted.
- **Light.** PhytKit's `Q` is absorbed light (`Qabs`); Helios multiplies alpha by the PAR
  each leaf absorbs. Fit against incident light and alpha absorbs the leaf's absorptance.
- **Co-limitation.** PhytKit blends the Rubisco, RuBP and TPU rates with a hyperbolic
  smooth minimum (curvature 0.999). Helios uses a quadratic blend over a 0.99 µmol window.
  A agrees closely except near the transitions between limitations; compare predictions
  with a tolerance, not exactly.
- **Precision.** Helios computes in single precision.
- **gm with a finite value.** If the user does want a mesophyll conductance, fit without
  `--ci-based` and pass `setMesophyllConductance_gm(gm)`. Both use Cc = Ci − A/gm, and PhytKit's
  gm has no temperature response. Check first that gm did not end on its bound.

## Stomatal conductance

### BTA (`bta` → `BMFcoefficients`)

Both use gs = Em·(Q + i0) / (k + b·Q + (Q + i0)·Ds), with gs in mol m⁻² s⁻¹ and Ds in
mmol mol⁻¹. The four parameters carry over **unchanged**:

| PhytKit | Helios `BMFcoefficients` | Units (both) |
|---|---|---|
| `Em` | `Em` | mmol m⁻² s⁻¹ |
| `i0` | `i0` | µmol m⁻² s⁻¹ |
| `k` | `k` | µmol m⁻² s⁻¹ · mmol mol⁻¹ |
| `b` | `b` | mmol mol⁻¹ |

- **Light.** Helios drives BMF with absorbed PAR. `pk fit stomatal` takes incident `Qin`
  when the file has it. Fit against `Qabs` (drop or rename the `Qin` column) when the values
  go to Helios, and check `mapping`.
- **Ds.** Helios computes Ds from the leaf to the leaf surface, not to the bulk air. In a
  well-stirred chamber the two are close.

### Medlyn (`medlyn` → `MOPTcoefficients`)

Both use gs = gs0 + 1.6·(1 + g1/√D)·A/Cs, with D in kPa and gs in mol m⁻² s⁻¹. Carry over
`gs0` (mol m⁻² s⁻¹) and `g1` (kPa^0.5) unchanged. Helios's soil-moisture factor β multiplies
inside the square root; PhytKit has none, which is β = 1.

## What does not transfer

- **`empiricalA`** is a different model from Helios's `EmpiricalModelCoefficients`. No
  parameter maps across.
- **`fvcbC4`** has not been checked against Helios's C4 model. Don't assume its parameters
  transfer.
- **Vulnerability, pressure–volume and soil retention** have no counterpart in these two
  plugins.
