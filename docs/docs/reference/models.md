# Models and parameters

Every built-in model, with its inputs, parameters and default bounds. This page is
generated from `pk describe` by `docs/gen_models.py`; `pk describe <model>` prints the
same information as JSON.

**Fixed by default** parameters are held at their default. To fit one, list it with `--fit`
(or `fit_params` in Python), but `--fit` replaces the whole set of fitted parameters, so also
list every other parameter you want fitted. Bounds are hard limits the fitter never crosses.

| Model | Tool | Name in the tool |
|---|---|---|
| [`empiricalA`](#empiricala) | photosynthesis | Empirical |
| [`fvcb`](#fvcb) | photosynthesis | Farquhar–von Caemmerer–Berry (C3) |
| [`fvcbC4`](#fvcbc4) | photosynthesis | von Caemmerer (C4) |
| [`vulnerability`](#vulnerability) | hydraulics | Sigmoidal vulnerability curve |
| [`pvcurve`](#pvcurve) | hydraulics | Pressure–volume curve |
| [`vanGenuchten`](#vangenuchten) | soil | van Genuchten |
| [`modifiedVanGenuchten`](#modifiedvangenuchten) | soil | modified van Genuchten |
| [`raats`](#raats) | soil | Raats superclass |
| [`brooksCorey`](#brookscorey) | soil | Brooks–Corey |
| [`fractalSWRC`](#fractalswrc) | soil | fractal (Li et al. 2025) |
| [`bta`](#bta) | stomatal | Buckley–Turnbull–Adams 2012 |
| [`medlyn`](#medlyn) | stomatal | Medlyn et al. 2011 (optimality) |
| [`lightResponse`](#lightresponse) | Python only | — |

## empiricalA

**Empirical** — `pk fit photosynthesis <file> --model empiricalA`

\[
A = A_{max}(T)\,\frac{Q}{Q + K_q(T)}\cdot\frac{C_c - \Gamma^*(T)}{C_c + K_{co}(T)} - R_d(T),\quad C_c = C_i - A/g_m\ (C_c = C_i \text{ when } g_m \ge 99)
\]

**Inputs:** `Ci` (intercellular CO2, µmol mol⁻¹), `Q` (absorbed light (PPFD), µmol m⁻² s⁻¹), `T` (leaf temperature in kelvin; pk fit converts a °C column, the array functions do not, K)

**Predicts:** `A` (net CO2 assimilation, µmol m⁻² s⁻¹)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `Amax25` | µmol m⁻² s⁻¹ | 25 | 1 | 200 |  | max assimilation at 25C |
| `Rd25` | µmol m⁻² s⁻¹ | 2.5 | 0 | 10 |  | dark respiration at 25C |
| `GammaS25` | µmol mol⁻¹ | 42.75 | 10 | 120 |  | CO2 compensation point at 25C |
| `Kco25` | µmol mol⁻¹ | 700 | 50 | 3000 |  | CO2 half-saturation at 25C |
| `Kq25` | µmol m⁻² s⁻¹ | 50 | 5 | 2000 |  | light half-saturation at 25C |
| `Amax_dHa` | kJ mol⁻¹ | 50 | 1 | 200 |  | Amax activation energy |
| `Amax_dHd` | kJ mol⁻¹ | 200 | 100 | 500 |  | Amax deactivation energy |
| `Amax_Topt` | K | 308.12 | 288 | 330 |  | temperature optimum of Amax, in kelvin (°C + 273.15; a value below 200 is read as °C) |
| `Rd_dHa` | kJ mol⁻¹ | 46.39 | 1 | 200 |  | Rd activation energy |
| `GammaS_dHa` | kJ mol⁻¹ | 37.83 | 1 | 200 |  | activation energy of Γ* (Arrhenius) |
| `Kq_dHa` | kJ mol⁻¹ | 70 | 1 | 200 |  | activation energy of Kq (Arrhenius) |
| `Kco_dHa` | kJ mol⁻¹ | 79.43 | 1 | 200 |  | activation energy of Kco (Arrhenius) |
| `gm` | mol m⁻² s⁻¹ bar⁻¹ | 0.25 | 0.01 | 99 |  | mesophyll conductance (Cc = Ci − A/gm); 99 or more means none (Cc = Ci) |

## fvcb

**Farquhar–von Caemmerer–Berry (C3)** — `pk fit photosynthesis <file> --model fvcb`

\[
A = \left(1-\tfrac{\Gamma^*}{C_c}\right)\min\{\,w_c,\ w_j,\ w_p\,\} - R_d,\quad C_c = C_i - A/g_m
\]

**Inputs:** `Ci` (intercellular CO2, µmol mol⁻¹), `Q` (absorbed light (PPFD), µmol m⁻² s⁻¹), `T` (leaf temperature in kelvin; pk fit converts a °C column, the array functions do not, K)

**Predicts:** `A` (net CO2 assimilation, µmol m⁻² s⁻¹)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `Vcmax25` | µmol m⁻² s⁻¹ | 100 | 5 | 400 |  | max Rubisco carboxylation at 25C |
| `Jmax25` | µmol m⁻² s⁻¹ | 200 | 5 | 700 |  | max electron transport at 25C |
| `TPU25` | µmol m⁻² s⁻¹ | 25 | 1 | 100 |  | triose-phosphate utilization at 25C |
| `Rd25` | µmol m⁻² s⁻¹ | 1.5 | 0 | 15 |  | day respiration at 25C |
| `gm` | mol m⁻² s⁻¹ bar⁻¹ | 0.25 | 0.01 | 5 |  | mesophyll conductance (Cc = Ci − A/gm) |
| `alpha` | mol mol⁻¹ | 0.5 | 0.1 | 0.9 |  | quantum yield of electron transport |
| `theta` | dimensionless | 0.7 | 0.1 | 0.99 |  | curvature of J(Q) |
| `alphaG` | dimensionless | 0 | 0 | 1 |  | glycolate carbon returned (TPU) |
| `Vcmax_dHa` | kJ mol⁻¹ | 65.33 | 1 | 200 |  | activation energy of Vcmax (peaked Arrhenius) |
| `Vcmax_dHd` | kJ mol⁻¹ | 200 | 100 | 500 |  | deactivation energy of Vcmax |
| `Vcmax_Topt` | K | 311.15 | 288 | 333 |  | temperature optimum of Vcmax, in kelvin (°C + 273.15; a value below 200 is read as °C) |
| `Jmax_dHa` | kJ mol⁻¹ | 43.9 | 1 | 200 |  | activation energy of Jmax (peaked Arrhenius) |
| `Jmax_dHd` | kJ mol⁻¹ | 200 | 100 | 500 |  | deactivation energy of Jmax |
| `Jmax_Topt` | K | 311.15 | 288 | 333 |  | temperature optimum of Jmax, in kelvin (°C + 273.15; a value below 200 is read as °C) |
| `TPU_dHa` | kJ mol⁻¹ | 53.1 | 1 | 200 |  | activation energy of TPU (peaked Arrhenius) |
| `TPU_dHd` | kJ mol⁻¹ | 201.8 | 100 | 500 |  | deactivation energy of TPU |
| `TPU_Topt` | K | 306.4 | 288 | 333 |  | temperature optimum of TPU, in kelvin (°C + 273.15; a value below 200 is read as °C) |
| `Rd_dHa` | kJ mol⁻¹ | 46.39 | 1 | 200 |  | activation energy of Rd (Arrhenius) |
| `GammaS25` | µmol mol⁻¹ | 42.75 | 10 | 120 | yes | CO2 compensation point (no Rd) at 25C |
| `GammaS_dHa` | kJ mol⁻¹ | 37.83 | 1 | 200 | yes | activation energy of Γ* (Arrhenius) |
| `Kc25` | µmol mol⁻¹ | 404.9 | 100 | 1000 | yes | Michaelis constant of Rubisco for CO2 at 25C |
| `Kc_dHa` | kJ mol⁻¹ | 79.43 | 1 | 200 | yes | activation energy of Kc (Arrhenius) |
| `Ko25` | mmol mol⁻¹ | 278.4 | 100 | 600 | yes | Michaelis constant of Rubisco for O2 at 25C |
| `Ko_dHa` | kJ mol⁻¹ | 36.38 | 1 | 200 | yes | activation energy of Ko (Arrhenius) |
| `O` | mmol mol⁻¹ | 213.5 | 100 | 300 | yes | chloroplastic O2 (mmol/mol) |

**Soft constraints** (penalties added to the fit; turn one off with `--no-constraint <key>`, all with `--constraints none`, turn on one that is off by default with `--constraint <key>`, reweight with `--weight <key>=<w>`):

| Key | Constraint | On by default | Default weight | Per curve | What it does |
|---|---|---|---|---|---|
| `ap_nonneg` | Ap ≥ 0 | yes | 1 |  | TPU-limited rate non-negative at every point |
| `aj_le_ac_mid` | Aj ≤ Ac (mid Ci) | yes | 0.3 | yes | Light-limited ≤ Rubisco-limited at Ci 500–700 |
| `rubisco_region` | Rubisco region exists | yes | 1 | yes | A low-Ci Rubisco-limited stretch must be present |
| `ap_above_cross` | Ap above Ac/Aj crossing | yes | 1 | yes | TPU not limiting at the Ac–Aj transition |
| `ap_limits_high` | Ap limits at high Ci | no | 1 | yes | TPU is the limiter at the highest-Ci point |
| `vcmax_window` | Vcmax25 ∈ [20, 130] | no | 0.05 |  | Soft prior keeping Vcmax25 in a typical range (PhoTorch) |
| `alphag_zero` | αG → 0 | yes | 2 |  | Pull the glycolate-return fraction toward 0 (PhoTorch) |
| `vj_ratio` | Jmax:Vcmax ≈ 1.6–2.0 | yes | 0.1 |  | Soft prior on the Jmax25:Vcmax25 ratio (single-curve analog of PhoTorch's correlation term) |

## fvcbC4

**von Caemmerer (C4)** — `pk fit photosynthesis <file> --model fvcbC4`

\[
A = \min\{A_c,\ A_j\},\quad V_p = \min\!\left(\tfrac{C_m V_{pmax}}{C_m+K_p},\ V_{pr}\right),\quad C_m = C_i - A/g_m
\]

**Inputs:** `Ci` (intercellular CO2 (read as µbar), µmol mol⁻¹), `Q` (absorbed light (PPFD), µmol m⁻² s⁻¹), `T` (leaf temperature in kelvin; pk fit converts a °C column, the array functions do not, K)

**Predicts:** `A` (net CO2 assimilation, µmol m⁻² s⁻¹)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `Vpmax25` | µmol m⁻² s⁻¹ | 200 | 1 | 600 |  | max PEP carboxylation at 25C |
| `Vcmax25` | µmol m⁻² s⁻¹ | 40 | 1 | 200 |  | max Rubisco carboxylation at 25C |
| `Jmax25` | µmol m⁻² s⁻¹ | 247.69 | 5 | 700 |  | max linear electron transport at 25C |
| `Rd25` | µmol m⁻² s⁻¹ | 1 | 0 | 15 |  | day respiration at 25C |
| `Vpmax_dHa` | kJ mol⁻¹ | 50.1 | 1 | 200 |  | activation energy of Vpmax (Arrhenius) |
| `Vcmax_dHa` | kJ mol⁻¹ | 78 | 1 | 200 |  | activation energy of Vcmax (Arrhenius) |
| `Jmax_dHa` | kJ mol⁻¹ | 77.9 | 1 | 200 |  | activation energy of Jmax (peaked Arrhenius) |
| `Jmax_dHd` | kJ mol⁻¹ | 260 | 100 | 500 |  | deactivation energy of Jmax |
| `Jmax_Topt` | K | 316.15 | 288 | 333 |  | temperature optimum of Jmax, in kelvin (°C + 273.15; a value below 200 is read as °C); above 1000 K, Jmax follows plain Arrhenius |
| `Rd_dHa` | kJ mol⁻¹ | 66.4 | 1 | 200 |  | activation energy of Rd (Arrhenius) |
| `gm` | mol m⁻² s⁻¹ bar⁻¹ | 1 | 0.05 | 5 | yes | mesophyll conductance (Cm = Ci − A/gm) |
| `gbs` | mol m⁻² s⁻¹ bar⁻¹ | 0.003 | 0.0001 | 0.05 | yes | bundle-sheath conductance |
| `Vpr` | µmol m⁻² s⁻¹ | 80 | 20 | 200 | yes | PEP regeneration cap |
| `alpha_psII` | dimensionless | 0 | 0 | 1 | yes | fraction of PSII activity in bundle sheath |
| `x_etr` | dimensionless | 0.4 | 0.05 | 0.95 | yes | fraction of linear ETR to mesophyll |
| `fcyc` | dimensionless | 0.45 | 0 | 0.85 | yes | cyclic electron flow fraction |
| `theta_etr` | dimensionless | 0.7 | 0.1 | 0.99 | yes | curvature of J(Q) |
| `Kc25` | µbar | 1210 | 500 | 3000 | yes | Rubisco Kc at 25C (μbar) |
| `Kc_dHa` | kJ mol⁻¹ | 64.2 | 1 | 200 | yes | activation energy of Kc (Arrhenius) |
| `Ko25` | µbar | 292000 | 100000 | 500000 | yes | Rubisco Ko at 25C (μbar) |
| `Ko_dHa` | kJ mol⁻¹ | 10.5 | 1 | 200 | yes | activation energy of Ko (Arrhenius) |
| `Kp25` | µbar | 82 | 20 | 300 | yes | PEPC Kp at 25C (μbar) |
| `Kp_dHa` | kJ mol⁻¹ | 38.3 | 1 | 200 | yes | activation energy of Kp (Arrhenius) |
| `GammaStar25` | dimensionless | 0.000381679 | 0.0001 | 0.001 | yes | half Rubisco O2/CO2 specificity (dimensionless) |
| `GammaStar_dHa` | kJ mol⁻¹ | 31.1 | 1 | 200 | yes | activation energy of γ* (Arrhenius) |
| `Om25` | µbar | 210000 | 100000 | 300000 | yes | bundle-sheath O2 partial pressure (μbar) |

**Soft constraints** (penalties added to the fit; turn one off with `--no-constraint <key>`, all with `--constraints none`, turn on one that is off by default with `--constraint <key>`, reweight with `--weight <key>=<w>`):

| Key | Constraint | On by default | Default weight | Per curve | What it does |
|---|---|---|---|---|---|
| `vpmax_gt_vpr` | Vpmax > Vpr | yes | 1 |  | PEP kinetic curve must reach its regeneration cap |
| `vp_breakpoint` | Vpr breakpoint in Ci range | yes | 0.02 | yes | Keep the Vpmax→Vpr transition inside the data (Vpmax/Vpr identifiability) |
| `acaj_regime` | Ac/Aj regime at high Ci | yes | 0.1 | yes | High light → Ac binds; low light → Aj binds, at saturating Ci |
| `rd_tether` | Rd ≈ 0.02·Vcmax | yes | 0.01 |  | Weak prior pulling day respiration toward 0.02·Vcmax where the data leave Rd undetermined |
| `rate_windows` | Rate windows | yes | 0.1 |  | Soft priors: Vpmax∈[20,400], Vcmax∈[5,200], Jmax∈[50,600] |

## vulnerability

**Sigmoidal vulnerability curve** — `pk fit hydraulics <file> --model vulnerability`

\[
K = \dfrac{K_{max}}{1+\exp\!\left[-\tfrac{S}{25}\,(\Psi - P_{50})\right]}
\]

**Inputs:** `P` (xylem water potential (negative), MPa)

**Predicts:** `K` (hydraulic conductance, units of the K column (dimensionless when fitted from PLC))

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `Kmax` | units of K (dimensionless when fitted from PLC) | 1 | 1e-09 | — |  | maximum hydraulic conductance at full hydration |
| `P50` | MPa | -2 | -15 | -0.05 |  | water potential at 50% loss of conductivity (MPa) |
| `S` | % MPa⁻¹ | 60 | 1 | 400 |  | slope at P50 (% MPa⁻¹) |

## pvcurve

**Pressure–volume curve** — `pk fit hydraulics <file> --model pvcurve`

\[
\Psi = \pi_o\left[\max\!\left(0,\tfrac{w-w_{tlp}}{1-w_{tlp}}\right)\right]^{\varepsilon} - \dfrac{\pi_o}{w}
\]

**Inputs:** `w` (relative water content, 0–1, dimensionless)

**Predicts:** `psi` (leaf water potential, MPa)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `πo` | MPa | 2 | 0.1 | 5 |  | osmotic pressure at full turgor (MPa) |
| `wtlp` | dimensionless | 0.85 | 0.5 | 0.99 |  | relative water content at turgor loss point |
| `ε` | dimensionless | 1 | 1 | 3 |  | wall-elasticity exponent on turgor |

## vanGenuchten

**van Genuchten** — `pk fit soil <file> --model vanGenuchten`

\[
\theta = \theta_r + (\theta_s-\theta_r)\left[1+(\alpha|\psi|)^{n}\right]^{-m},\; m=1-\tfrac{1}{n}
\]

**Inputs:** `psi` (matric potential (negative); pk fit soil converts other units to MPa, MPa)

**Predicts:** `theta` (volumetric water content, m³ m⁻³)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `θr` | m³ m⁻³ | 0.05 | 0 | 0.5 |  | residual volumetric water content |
| `θs` | m³ m⁻³ | 0.45 | 0.1 | 1 |  | saturated volumetric water content |
| `α` | MPa⁻¹ | 1 | 0.0001 | 100000 |  | inverse air-entry pressure (ψ⁻¹) |
| `n` | dimensionless | 1.5 | 1.01 | 12 |  | pore-size distribution parameter |

## modifiedVanGenuchten

**modified van Genuchten** — `pk fit soil <file> --model modifiedVanGenuchten`

\[
\theta = \theta_r + (\theta_s-\theta_r)\left[1+(\alpha_1|\psi|)^{n_1}\right]^{-(1+1/n_1)}
\]

**Inputs:** `psi` (matric potential (negative); pk fit soil converts other units to MPa, MPa)

**Predicts:** `theta` (volumetric water content, m³ m⁻³)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `θr` | m³ m⁻³ | 0.05 | 0 | 0.5 |  | residual volumetric water content |
| `θs` | m³ m⁻³ | 0.45 | 0.1 | 1 |  | saturated volumetric water content |
| `α1` | MPa⁻¹ | 1 | 0.0001 | 100000 |  | MVG retention scale parameter (ψ⁻¹) |
| `n1` | dimensionless | 1.5 | 0.2 | 12 |  | MVG retention shape parameter |
| `α2` | MPa⁻¹ | 1 | 0.0001 | 100000 | yes | MVG-RHC conductivity scale parameter (ψ⁻¹) |
| `n2` | dimensionless | 1.5 | 0.2 | 12 | yes | MVG-RHC conductivity shape parameter |

## raats

**Raats superclass** — `pk fit soil <file> --model raats`

\[
|\psi| = b^{*}\left[\dfrac{1-S^{\alpha}}{S^{\beta}}\right]^{\gamma},\quad S=\dfrac{\theta-\theta_r}{\theta_s-\theta_r}
\]

**Inputs:** `psi` (matric potential (negative); pk fit soil converts other units to MPa, MPa)

**Predicts:** `theta` (volumetric water content, m³ m⁻³)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `θr` | m³ m⁻³ | 0.05 | 0 | 0.5 |  | residual volumetric water content |
| `θs` | m³ m⁻³ | 0.45 | 0.1 | 1 |  | saturated volumetric water content |
| `b*` | MPa | 0.05 | 1e-05 | 100000 |  | characteristic pressure head (ψ units) |
| `α` | dimensionless | 5 | 0.01 | 10 |  | Raats shape parameter α |
| `β` | dimensionless | 5 | 0.1 | 40 |  | Raats shape parameter β (dry end) |
| `γ` | dimensionless | 0.8 | 0.01 | 0.999 |  | Raats shape parameter γ (wet end, 0–1) |
| `λ` | dimensionless | 0.5 | -5 | 5 | yes | Mualem tortuosity exponent (conductivity) |

## brooksCorey

**Brooks–Corey** — `pk fit soil <file> --model brooksCorey`

\[
S_e = \left(\dfrac{\psi_b}{|\psi|}\right)^{\lambda}\ (|\psi|>\psi_b),\quad \theta=\theta_r+(\theta_s-\theta_r)S_e
\]

**Inputs:** `psi` (matric potential (negative); pk fit soil converts other units to MPa, MPa)

**Predicts:** `theta` (volumetric water content, m³ m⁻³)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `θr` | m³ m⁻³ | 0.05 | 0 | 0.5 |  | residual volumetric water content |
| `θs` | m³ m⁻³ | 0.45 | 0.1 | 1 |  | saturated volumetric water content |
| `ψb` | MPa | 0.03 | 1e-05 | 10000 |  | air-entry (bubbling) potential (ψ units) |
| `λ` | dimensionless | 0.5 | 0.05 | 10 |  | pore-size distribution index |
| `L` | dimensionless | 0.5 | -5 | 5 | yes | Mualem tortuosity exponent (conductivity) |

## fractalSWRC

**fractal (Li et al. 2025)** — `pk fit soil <file> --model fractalSWRC`

\[
s = \dfrac{4\lambda}{\pi}\dfrac{2-D}{1-D}\dfrac{1-\delta}{r_{max}}\dfrac{1-\left(1-\tfrac{1-S_e}{(1-\delta)^2}\right)^{\frac{1-D}{2-D}}}{S_e}
\]

**Inputs:** `psi` (matric potential (negative); pk fit soil converts other units to MPa, MPa)

**Predicts:** `theta` (volumetric water content, m³ m⁻³)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `θr` | m³ m⁻³ | 0.05 | 0 | 0.5 |  | residual volumetric water content |
| `θs` | m³ m⁻³ | 0.45 | 0.1 | 1 |  | saturated volumetric water content |
| `D` | dimensionless | 1.5 | 1.001 | 1.999 |  | fractal dimension of the pore-size distribution |
| `r_max` | µm | 5 | 0.001 | 10000 |  | maximum equivalent pore radius (µm) |
| `δ` | dimensionless | 0.02 | 0 | 0.3 |  | relative adsorptive water-film thickness |
| `L` | dimensionless | 0.5 | -5 | 5 | yes | Mualem tortuosity exponent (conductivity) |

## bta

**Buckley–Turnbull–Adams 2012** — `pk fit stomatal <file> --model bta`

\[
g_s = \frac{E_m\,(Q + i_0)}{k + b\,Q + (Q + i_0)\,D_s}
\]

**Inputs:** `Q` (light (PPFD); pk fit stomatal prefers incident Qin over Qabs, µmol m⁻² s⁻¹), `Ds` (leaf-to-air vapour pressure deficit as a mole fraction, mmol mol⁻¹)

**Predicts:** `gs` (stomatal conductance to water vapour, mol m⁻² s⁻¹)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `Em` | mmol m⁻² s⁻¹ | 0.2 | 0.001 | 20 |  | maximum transpiration rate (gs in mol m⁻² s⁻¹ times Ds in mmol mol⁻¹) |
| `i0` | µmol m⁻² s⁻¹ | 20 | 0 | 500 |  | light compensation offset (µmol m⁻² s⁻¹) |
| `k` | µmol m⁻² s⁻¹ mmol mol⁻¹ | 100000 | 1 | 1e+07 |  | denominator constant |
| `b` | mmol mol⁻¹ | 100 | 0 | 20000 |  | VPD/light sensitivity |

## medlyn

**Medlyn et al. 2011 (optimality)** — `pk fit stomatal <file> --model medlyn`

\[
g_s = g_{s,0} + 1.6\left(1 + \frac{g_1}{\sqrt{D}}\right)\frac{A}{C_s}
\]

**Inputs:** `A` (net CO2 assimilation, µmol m⁻² s⁻¹), `Cs` (CO2 at the leaf surface, µmol mol⁻¹), `D` (leaf-to-air vapour pressure deficit, kPa)

**Predicts:** `gs` (stomatal conductance to water vapour, mol m⁻² s⁻¹)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `gs0` | mol m⁻² s⁻¹ | 0.01 | 0 | 1 |  | residual conductance (mol m⁻² s⁻¹) |
| `g1` | kPa^0.5 | 3 | 0.01 | 30 |  | slope, sensitivity to VPD (√kPa) |

## lightResponse

Available from Python (`fit_arrays`, `fit_many`, `predict`); no `pk fit` tool uses it.

\[
\frac{ Amax\cdot Q}{\left( Q+ Kq\right)}- Rd
\]

**Inputs:** `Q` (absorbed light (PPFD), µmol m⁻² s⁻¹)

**Predicts:** `A` (net CO2 assimilation, µmol m⁻² s⁻¹)

| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|---|
| `Amax` | µmol m⁻² s⁻¹ | 25 | 1 | 200 |  | light-saturated gross assimilation |
| `Kq` | µmol m⁻² s⁻¹ | 300 | 5 | 2000 |  | absorbed light at half saturation |
| `Rd` | µmol m⁻² s⁻¹ | 1.5 | 0 | 10 |  | day respiration |
