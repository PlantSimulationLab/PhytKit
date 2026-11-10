---
title: PhytKit
hide:
  - navigation
  - toc
---

<div class="pk-hero" markdown>

![PhytKit](assets/phytkit_wordmark.png){ .pk-wordmark }

# Plant physiology model fitting for you and your agent { .pk-tagline }

<p class="pk-lead">
Fit photosynthesis, stomatal conductance, hydraulic vulnerability,
<span class="pk-nowrap">pressure–volume</span> and soil water retention models to your
measurements, and correct <span class="pk-nowrap">LI-600</span> porometer data.
</p>

<div class="pk-actions" markdown>
[Get started](guide/install.md){ .md-button .md-button--primary }
[Browse workflows](workflows/index.md){ .md-button }
</div>

<div class="pk-snippet" markdown>
```bash
pip install phytkit
```
</div>

</div>

## What it does

<div class="grid cards" markdown>

- :material-leaf: **Fit photosynthesis curves**

    Fit A–Ci, light and temperature responses with the Farquhar–von
    Caemmerer–Berry C3 model, the von Caemmerer C4 model, or an empirical
    model. Read LI-6800 logs and `.xlsx` exports directly, with one curve per
    leaf or many pooled under a CurveID.

- :material-water-percent: **Correct LI-600 conductance**

    Apply the Rizzo & Bailey (2025) energy-balance correction to LI-600
    porometer readings. The porometer ignores evaporative cooling in its
    chamber, so it overestimates conductance, most at high gsw.

- :material-sprout: **Estimate A from porometry**

    Solve net assimilation, Ci and Cc for every porometer reading from its
    conductance, light and leaf temperature and a photosynthesis parameter set.

- :material-weather-windy: **Fit stomatal models**

    Fit the Buckley–Turnbull–Adams (2012) and Medlyn et al. (2011) stomatal
    conductance models to gas exchange or porometry data.

- :material-pipe: **Fit water relations**

    Fit xylem vulnerability curves and pressure–volume curves, reporting P50,
    Kmax and the turgor loss point, plus five soil water retention models.

- :material-layers-triple: **Fit many curves at once**

    Fit a folder of files, or split one file by a column such as Leaf, in a
    single command. From Python, fit thousands of datasets in parallel across
    your CPU cores.

- :material-console: **Built for scripts and agents**

    Every command prints one JSON object by default (`pk concat` prints the
    merged CSV unless asked for JSON), errors are structured, and
    `pk describe` lists every model, parameter, bound and constraint, so
    scripts and coding agents can discover what to pass.

- :material-language-python: **Use it from Python**

    `import phytkit` runs the same code as the command line and returns the
    same results as Python dictionaries, plus functions for data already in
    memory.

</div>

## Get started

New to PhytKit? Start with **[Install PhytKit](guide/install.md)**, then
**[Your first fit](guide/first-fit.md)**.

Looking for something specific?

- **[Workflows](workflows/index.md)** walk through common tasks step by step.
- **[Concepts](concepts/index.md)** explain the models and how fitting works.
- **[Reference](reference/index.md)** lists every command, option, model and
  parameter.

## Credits

The original version of PhytKit was developed primarily by Kyle Rizzo
([@ktrizzo](https://github.com/ktrizzo)). It is maintained by the
[PlantSimulationLab](https://github.com/PlantSimulationLab).
