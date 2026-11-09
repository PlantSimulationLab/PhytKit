#!/usr/bin/env python3
"""Make the example data files in this folder. Seeded, so the files are reproducible.

These are synthetic: plausible shapes plus noise, not draws from the exact model equations,
so the fitters have real work to do.
"""
import math
import random
from pathlib import Path

OUT = Path(__file__).parent
rng = random.Random(20261109)


def write(name, header, rows):
    (OUT / name).write_text(",".join(header) + "\n" + "\n".join(",".join(f"{v:.6g}" if isinstance(v, float) else str(v) for v in r) for r in rows) + "\n")


# Vulnerability: PLC rising sigmoidally with tension, three stems.
rows = []
for stem, p50 in (("s1", -2.1), ("s2", -2.6), ("s3", -1.8)):
    for i in range(14):
        p = -0.2 - 0.35 * i
        plc = 100 / (1 + math.exp(1.8 * (p - p50))) + rng.gauss(0, 3)
        rows.append((stem, p, max(0.0, min(100.0, plc))))
write("vulnerability.csv", ["stem", "P", "PLC"], rows)
# Vulnerability from conductance (K column, so the parser derives PLC and Kmax).
write("vulnerability_k.csv", ["Psi (MPa)", "Kleaf"], [(-0.1 - 0.3 * i, 12 / (1 + math.exp(-1.5 * (-0.1 - 0.3 * i) - 3.2)) + rng.gauss(0, 0.3)) for i in range(16)])

# Soil retention: van Genuchten-like curve in kPa, two samples.
rows = []
for sample, (ts, tr, a, n) in (("loam", (0.43, 0.05, 0.036, 1.56)), ("sand", (0.38, 0.03, 0.145, 2.68))):
    for h in (0.5, 1, 2, 5, 10, 20, 33, 50, 100, 300, 600, 1000, 1500):
        theta = tr + (ts - tr) / (1 + (a * h) ** n) ** (1 - 1 / n) + rng.gauss(0, 0.004)
        rows.append((sample, -h, theta))
write("soil.csv", ["sample", "psi", "theta"], rows)

# BTA stomatal conductance: gs falling with VPD and saturating with light.
rows = []
for _ in range(60):
    q = rng.uniform(50, 2000)
    d = rng.uniform(0.6, 4.0)
    gs = 0.45 * q / (q + 250) / (1 + 0.45 * d) + rng.gauss(0, 0.012)
    rows.append((max(gs, 0.005), q, d))
write("bta.csv", ["gsw", "Qin", "VPDleaf"], rows)

# Medlyn: gs = 1.6 (1 + g1/sqrt(D)) A/Cs.
rows = []
for _ in range(60):
    a = rng.uniform(2, 28)
    d = rng.uniform(0.6, 3.5)
    cs = rng.uniform(380, 430)
    gs = 0.01 + 1.6 * (1 + 3.8 / math.sqrt(d)) * a / cs + rng.gauss(0, 0.01)
    rows.append((max(gs, 0.005), a, cs, d))
write("medlyn.csv", ["gsw", "A", "Cs", "VPDleaf"], rows)

# Edge cases for parsing: quoting, blank lines, CRLF, a BOM and junk values.
(OUT / "messy_aci.csv").write_bytes(
    "﻿A,Ci,\"Qabs\",Tleaf,note\r\n"
    "12.1,250,1500,25,\"plain\"\r\n"
    "\r\n"
    "15.3,400,1500,25.5,\"has, comma\"\r\n"
    "18.0,600,1500,26,\"quote \"\"inside\"\"\"\r\n"
    "  19.4 ,800,1500,26,\r\n"
    "bad,900,1500,26,dropped\r\n"
    "20.1,1000,1500,1e1,celsius as 1e1\r\n"
    "21.0,1200,1500,25,\r\n"
    "   \r\n".encode()
)
# A-Ci curves at two light levels and three temperatures, from a simple FvCB (Ci in place of Cc).
def fvcb_a(ci, q, t_c, vcmax25=95.0, jmax25=180.0, rd25=1.4):
    tk = t_c + 273.15
    arr = lambda k25, ha: k25 * math.exp(ha / 0.008314 * (1 / 298.15 - 1 / tk))
    vc, jm, rd = arr(vcmax25, 65.3), arr(jmax25, 43.9), arr(rd25, 46.4)
    gamma, kc, ko = arr(42.75, 37.83), arr(404.9, 79.43), arr(278.4, 36.38)
    aq = 0.3 * q
    j = (aq + jm - math.sqrt((aq + jm) ** 2 - 4 * 0.7 * aq * jm)) / (2 * 0.7)
    wc = vc * ci / (ci + kc * (1 + 210 / ko))
    wj = j * ci / (4 * ci + 8 * gamma)
    return (1 - gamma / ci) * min(wc, wj) - rd


rows = []
curve = 0
for t_c in (20.0, 28.0, 36.0):
    for q in (1500.0, 400.0):
        curve += 1
        for ci in (60, 100, 150, 200, 300, 400, 600, 800, 1000, 1300, 1600):
            ci_obs = ci + rng.gauss(0, 3)
            rows.append((round(fvcb_a(ci_obs, q, t_c) + rng.gauss(0, 0.4), 4), round(ci_obs, 2), q, t_c, curve))
write("aci.csv", ["A", "Ci", "Qabs", "Tleaf", "CurveID"], rows)
# Survey points for rescale mode: A at one light and temperature, about 15% above the curves.
write("survey.csv", ["A", "Ci", "Qabs", "Tleaf"], [(round(1.15 * fvcb_a(1500, 1500, 28) + rng.gauss(0, 0.5), 3), 1500.0, 1500.0, 28.0) for _ in range(8)])

# Pressure-volume curves for three leaves: RWC falling with drying, psi in MPa.
rows = []
for leaf, (pi_o, w_tlp, eps) in ((1, (1.6, 0.86, 1.4)), (2, (1.9, 0.83, 1.7)), (3, (1.4, 0.88, 1.2))):
    for i in range(14):
        w = 1.0 - 0.018 * i - 0.004 * i * i
        turgor = pi_o * max(0.0, (w - w_tlp) / (1 - w_tlp)) ** eps
        psi = turgor - pi_o / w + rng.gauss(0, 0.03)
        rows.append((leaf, round(w * 100, 3), round(psi, 4)))
write("pv.csv", ["Leaf", "RWC", "Psi (MPa)"], rows)

# An LI-600 porometer export: category row, column names, units row, then readings.
# Each reading starts from a stomatal conductance; outlet humidity and apparent
# transpiration then follow from the porometer's isothermal mass balance, as the
# instrument would report them.
def es(t):
    return 0.61365 * math.exp(17.502 * t / (t + 240.97))


AREA, GBW = 0.441786e-4, 2.921  # m2 aperture, mol m-2 s-1 boundary layer


def li600_reading(gsw, tleaf, tref, rh_r, flow, p):
    u = flow * 1e-6
    w_in, w_leaf = es(tref) * rh_r / 100 / p, es(tleaf) / p
    gtw = gsw * GBW / (gsw + GBW)
    lo, hi = w_in, w_leaf
    for _ in range(100):  # outlet mole fraction where the leaf flux equals the flow's uptake
        w_out = 0.5 * (lo + hi)
        e_flow = u * (w_out - w_in) / (AREA * (1 - w_out))
        e_leaf = gtw * (w_leaf - 0.5 * (w_in + w_out))
        lo, hi = (w_out, hi) if e_flow < e_leaf else (lo, w_out)
    return 100 * w_out * p / es(tref), 1000 * e_flow


lines = [
    "SYS,SYS,PORO,PORO,PORO,PORO,PORO,PORO,PORO,PORO,PORO,FLUORO",
    "Obs#,Time,gsw,E_apparent,Tref,Tleaf,rh_r,rh_s,flow,P_atm,Qamb,ETR",
    ",,mol+1m-2s-1,mmol+1m-2s-1,C,C,%,%,umol+1s-1,kPa,umol+1m-2s-1,umol+1m-2s-1",
]
for i in range(40):
    tleaf = 24 + 8 * (i / 39) + rng.gauss(0, 0.3)
    tref = tleaf - 0.4 + rng.gauss(0, 0.2)
    rh_r = 38 + rng.gauss(0, 2)
    gsw = max(0.02, 0.05 + 0.5 * math.sin(math.pi * i / 39) + rng.gauss(0, 0.03))
    flow, p = 150.0, 100.8
    rh_s, e_app = li600_reading(gsw, tleaf, tref, rh_r, flow, p)
    q = round(200 + 1600 * math.sin(math.pi * i / 39) + rng.gauss(0, 30), 1)
    lines.append(f"{i + 1},10:{i:02d}:00,{gsw:.4f},{e_app:.4f},{tref:.2f},{tleaf:.2f},{rh_r:.2f},{rh_s:.2f},{flow},{p},{q},{max(5.0, 0.3 * q):.1f}")
(OUT / "li600.csv").write_text("\n".join(lines) + "\n")

print("wrote", sorted(p.name for p in OUT.glob("*.csv")))
