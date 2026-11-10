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

**Inputs:** `Ci` (intercellular CO₂, µmol mol⁻¹), `Q` (absorbed light (PPFD), µmol m⁻² s⁻¹), `T` (leaf temperature, K)

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `Amax25` | 25 | 1 | 200 |  | max assimilation at 25C |
| `Rd25` | 2.5 | 0 | 10 |  | dark respiration at 25C |
| `GammaS25` | 42.75 | 10 | 120 |  | CO2 compensation point at 25C |
| `Kco25` | 700 | 50 | 3000 |  | CO2 half-saturation at 25C |
| `Kq25` | 50 | 5 | 2000 |  | light half-saturation at 25C |
| `Amax_dHa` | 50 | 1 | 200 |  | Amax activation energy |
| `Amax_dHd` | 200 | 100 | 500 |  | Amax deactivation energy |
| `Amax_Topt` | 308.12 | 288 | 330 |  | Amax optimum temp (K) |
| `Rd_dHa` | 46.39 | 1 | 200 |  | Rd activation energy |
| `GammaS_dHa` | 37.83 | 1 | 200 |  |  |
| `Kq_dHa` | 70 | 1 | 200 |  |  |
| `Kco_dHa` | 79.43 | 1 | 200 |  |  |
| `gm` | 0.25 | 0.01 | 99 |  | mesophyll conductance |

## fvcb

**Farquhar–von Caemmerer–Berry (C3)** — `pk fit photosynthesis <file> --model fvcb`

\[
A = \left(1-\tfrac{\Gamma^*}{C_c}\right)\min\{\,w_c,\ w_j,\ w_p\,\} - R_d,\quad C_c = C_i - A/g_m
\]

**Inputs:** `Ci` (intercellular CO₂, µmol mol⁻¹), `Q` (absorbed light (PPFD), µmol m⁻² s⁻¹), `T` (leaf temperature, K)

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `Vcmax25` | 100 | 5 | 400 |  | max Rubisco carboxylation at 25C |
| `Jmax25` | 200 | 5 | 700 |  | max electron transport at 25C |
| `TPU25` | 25 | 1 | 100 |  | triose-phosphate utilization at 25C |
| `Rd25` | 1.5 | 0 | 15 |  | day respiration at 25C |
| `gm` | 0.25 | 0.01 | 5 |  | mesophyll conductance (Cc = Ci − A/gm) |
| `alpha` | 0.5 | 0.1 | 0.9 |  | quantum yield of electron transport |
| `theta` | 0.7 | 0.1 | 0.99 |  | curvature of J(Q) |
| `alphaG` | 0 | 0 | 1 |  | glycolate carbon returned (TPU) |
| `Vcmax_dHa` | 65.33 | 1 | 200 |  |  |
| `Vcmax_dHd` | 200 | 100 | 500 |  |  |
| `Vcmax_Topt` | 311.15 | 288 | 333 |  |  |
| `Jmax_dHa` | 43.9 | 1 | 200 |  |  |
| `Jmax_dHd` | 200 | 100 | 500 |  |  |
| `Jmax_Topt` | 311.15 | 288 | 333 |  |  |
| `TPU_dHa` | 53.1 | 1 | 200 |  |  |
| `TPU_dHd` | 201.8 | 100 | 500 |  |  |
| `TPU_Topt` | 306.4 | 288 | 333 |  |  |
| `Rd_dHa` | 46.39 | 1 | 200 |  |  |
| `GammaS25` | 42.75 | 10 | 120 | yes | CO2 compensation point (no Rd) at 25C |
| `GammaS_dHa` | 37.83 | 1 | 200 | yes |  |
| `Kc25` | 404.9 | 100 | 1000 | yes |  |
| `Kc_dHa` | 79.43 | 1 | 200 | yes |  |
| `Ko25` | 278.4 | 100 | 600 | yes |  |
| `Ko_dHa` | 36.38 | 1 | 200 | yes |  |
| `O` | 213.5 | 100 | 300 | yes | chloroplastic O2 (mmol/mol) |

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

**Inputs:** `Ci` (intercellular CO₂, µmol mol⁻¹), `Q` (absorbed light (PPFD), µmol m⁻² s⁻¹), `T` (leaf temperature, K)

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `Vpmax25` | 200 | 1 | 600 |  | max PEP carboxylation at 25C |
| `Vcmax25` | 40 | 1 | 200 |  | max Rubisco carboxylation at 25C |
| `Jmax25` | 247.69 | 5 | 700 |  | max linear electron transport at 25C |
| `Rd25` | 1 | 0 | 15 |  | day respiration at 25C |
| `Vpmax_dHa` | 50.1 | 1 | 200 |  |  |
| `Vcmax_dHa` | 78 | 1 | 200 |  |  |
| `Jmax_dHa` | 77.9 | 1 | 200 |  |  |
| `Jmax_dHd` | 260 | 100 | 500 |  |  |
| `Jmax_Topt` | 316.15 | 288 | 333 |  |  |
| `Rd_dHa` | 66.4 | 1 | 200 |  |  |
| `gm` | 1 | 0.05 | 5 | yes | mesophyll conductance (Cm = Ci − A/gm) |
| `gbs` | 0.003 | 0.0001 | 0.05 | yes | bundle-sheath conductance |
| `Vpr` | 80 | 20 | 200 | yes | PEP regeneration cap |
| `alpha_psII` | 0 | 0 | 1 | yes | fraction of PSII activity in bundle sheath |
| `x_etr` | 0.4 | 0.05 | 0.95 | yes | fraction of linear ETR to mesophyll |
| `fcyc` | 0.45 | 0 | 0.85 | yes | cyclic electron flow fraction |
| `theta_etr` | 0.7 | 0.1 | 0.99 | yes | curvature of J(Q) |
| `Kc25` | 1210 | 500 | 3000 | yes | Rubisco Kc at 25C (μbar) |
| `Kc_dHa` | 64.2 | 1 | 200 | yes |  |
| `Ko25` | 292000 | 100000 | 500000 | yes | Rubisco Ko at 25C (μbar) |
| `Ko_dHa` | 10.5 | 1 | 200 | yes |  |
| `Kp25` | 82 | 20 | 300 | yes | PEPC Kp at 25C (μbar) |
| `Kp_dHa` | 38.3 | 1 | 200 | yes |  |
| `GammaStar25` | 0.000381679 | 0.0001 | 0.001 | yes | half Rubisco O2/CO2 specificity (dimensionless) |
| `GammaStar_dHa` | 31.1 | 1 | 200 | yes |  |
| `Om25` | 210000 | 100000 | 300000 | yes | bundle-sheath O2 partial pressure (μbar) |

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

**Inputs:** `P` (water potential, MPa (negative))

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `Kmax` | 1 | 1e-09 | — |  | maximum hydraulic conductance at full hydration |
| `P50` | -2 | -15 | -0.05 |  | water potential at 50% loss of conductivity (MPa) |
| `S` | 60 | 1 | 400 |  | slope at P50 (% MPa⁻¹) |

## pvcurve

**Pressure–volume curve** — `pk fit hydraulics <file> --model pvcurve`

\[
\Psi = \pi_o\left[\max\!\left(0,\tfrac{w-w_{tlp}}{1-w_{tlp}}\right)\right]^{\varepsilon} - \dfrac{\pi_o}{w}
\]

**Inputs:** `w` (relative water content, 0–1)

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `πo` | 2 | 0.1 | 5 |  | osmotic pressure at full turgor (MPa) |
| `wtlp` | 0.85 | 0.5 | 0.99 |  | relative water content at turgor loss point |
| `ε` | 1 | 1 | 3 |  | wall-elasticity exponent on turgor |

## vanGenuchten

**van Genuchten** — `pk fit soil <file> --model vanGenuchten`

\[
\theta = \theta_r + (\theta_s-\theta_r)\left[1+(\alpha|\psi|)^{n}\right]^{-m},\; m=1-\tfrac{1}{n}
\]

**Inputs:** `psi` (matric potential, MPa (negative))

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `θr` | 0.05 | 0 | 0.5 |  | residual volumetric water content |
| `θs` | 0.45 | 0.1 | 1 |  | saturated volumetric water content |
| `α` | 1 | 0.0001 | 100000 |  | inverse air-entry pressure (ψ⁻¹) |
| `n` | 1.5 | 1.01 | 12 |  | pore-size distribution parameter |

## modifiedVanGenuchten

**modified van Genuchten** — `pk fit soil <file> --model modifiedVanGenuchten`

\[
\theta = \theta_r + (\theta_s-\theta_r)\left[1+(\alpha_1|\psi|)^{n_1}\right]^{-(1+1/n_1)}
\]

**Inputs:** `psi` (matric potential, MPa (negative))

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `θr` | 0.05 | 0 | 0.5 |  | residual volumetric water content |
| `θs` | 0.45 | 0.1 | 1 |  | saturated volumetric water content |
| `α1` | 1 | 0.0001 | 100000 |  | MVG retention scale parameter (ψ⁻¹) |
| `n1` | 1.5 | 0.2 | 12 |  | MVG retention shape parameter |
| `α2` | 1 | 0.0001 | 100000 | yes | MVG-RHC conductivity scale parameter (ψ⁻¹) |
| `n2` | 1.5 | 0.2 | 12 | yes | MVG-RHC conductivity shape parameter |

## raats

**Raats superclass** — `pk fit soil <file> --model raats`

\[
|\psi| = b^{*}\left[\dfrac{1-S^{\alpha}}{S^{\beta}}\right]^{\gamma},\quad S=\dfrac{\theta-\theta_r}{\theta_s-\theta_r}
\]

**Inputs:** `psi` (matric potential, MPa (negative))

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `θr` | 0.05 | 0 | 0.5 |  | residual volumetric water content |
| `θs` | 0.45 | 0.1 | 1 |  | saturated volumetric water content |
| `b*` | 0.05 | 1e-05 | 100000 |  | characteristic pressure head (ψ units) |
| `α` | 5 | 0.01 | 10 |  | Raats shape parameter α |
| `β` | 5 | 0.1 | 40 |  | Raats shape parameter β (dry end) |
| `γ` | 0.8 | 0.01 | 0.999 |  | Raats shape parameter γ (wet end, 0–1) |
| `λ` | 0.5 | -5 | 5 | yes | Mualem tortuosity exponent (conductivity) |

## brooksCorey

**Brooks–Corey** — `pk fit soil <file> --model brooksCorey`

\[
S_e = \left(\dfrac{\psi_b}{|\psi|}\right)^{\lambda}\ (|\psi|>\psi_b),\quad \theta=\theta_r+(\theta_s-\theta_r)S_e
\]

**Inputs:** `psi` (matric potential, MPa (negative))

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `θr` | 0.05 | 0 | 0.5 |  | residual volumetric water content |
| `θs` | 0.45 | 0.1 | 1 |  | saturated volumetric water content |
| `ψb` | 0.03 | 1e-05 | 10000 |  | air-entry (bubbling) potential (ψ units) |
| `λ` | 0.5 | 0.05 | 10 |  | pore-size distribution index |
| `L` | 0.5 | -5 | 5 | yes | Mualem tortuosity exponent (conductivity) |

## fractalSWRC

**fractal (Li et al. 2025)** — `pk fit soil <file> --model fractalSWRC`

\[
s = \dfrac{4\lambda}{\pi}\dfrac{2-D}{1-D}\dfrac{1-\delta}{r_{max}}\dfrac{1-\left(1-\tfrac{1-S_e}{(1-\delta)^2}\right)^{\frac{1-D}{2-D}}}{S_e}
\]

**Inputs:** `psi` (matric potential, MPa (negative))

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `θr` | 0.05 | 0 | 0.5 |  | residual volumetric water content |
| `θs` | 0.45 | 0.1 | 1 |  | saturated volumetric water content |
| `D` | 1.5 | 1.001 | 1.999 |  | fractal dimension of the pore-size distribution |
| `r_max` | 5 | 0.001 | 10000 |  | maximum equivalent pore radius (µm) |
| `δ` | 0.02 | 0 | 0.3 |  | relative adsorptive water-film thickness |
| `L` | 0.5 | -5 | 5 | yes | Mualem tortuosity exponent (conductivity) |

## bta

**Buckley–Turnbull–Adams 2012** — `pk fit stomatal <file> --model bta`

\[
g_s = \frac{E_m\,(Q + i_0)}{k + b\,Q + (Q + i_0)\,D_s}
\]

**Inputs:** `Q` (light (PPFD), µmol m⁻² s⁻¹; `pk fit stomatal` prefers `Qin` (incident) over `Qabs`, see Input files), `Ds` (leaf-to-air vapour pressure deficit as a mole fraction, mmol mol⁻¹)

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `Em` | 0.2 | 0.001 | 20 |  | conductance scale (mol m⁻² s⁻¹) |
| `i0` | 20 | 0 | 500 |  | light compensation offset (µmol m⁻² s⁻¹) |
| `k` | 100000 | 1 | 1e+07 |  | denominator constant |
| `b` | 100 | 0 | 20000 |  | VPD/light sensitivity |

## medlyn

**Medlyn et al. 2011 (optimality)** — `pk fit stomatal <file> --model medlyn`

\[
g_s = g_{s,0} + 1.6\left(1 + \frac{g_1}{\sqrt{D}}\right)\frac{A}{C_s}
\]

**Inputs:** `A` (net CO₂ assimilation, µmol m⁻² s⁻¹), `Cs` (CO₂ at the leaf surface, µmol mol⁻¹), `D` (leaf-to-air vapour pressure deficit, kPa)

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `gs0` | 0.01 | 0 | 1 |  | residual conductance (mol m⁻² s⁻¹) |
| `g1` | 3 | 0.01 | 30 |  | slope, sensitivity to VPD (√kPa) |

## lightResponse

Available from Python (`fit_arrays`, `fit_many`, `predict`); no `pk fit` tool uses it.

\[
\frac{ Amax\cdot Q}{\left( Q+ Kq\right)}- Rd
\]

**Inputs:** `Q` (absorbed light (PPFD), µmol m⁻² s⁻¹)

| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |
|---|---|---|---|---|---|
| `Amax` | 25 | 1 | 200 |  |  |
| `Kq` | 300 | 5 | 2000 |  |  |
| `Rd` | 1.5 | 0 | 10 |  |  |
