# Rescale curves to survey points

Response curves are often measured on a few leaves, while a larger survey
measures A on many leaves under one fixed condition. **Rescale mode** puts the
curves on the survey's level of photosynthesis. It keeps each curve's shape
and scales its amplitude, then refits.

## How it works

1. Fit the curves as usual.
2. Take the survey's mean light and leaf temperature as the reference
   condition. The reference Ci is the highest Ci in the curve data, or the
   value you set with `--ref-ci`.
3. Predict A at that condition from the fit.
4. Compute the factor: the survey A statistic divided by the predicted A, using
   `--stat mean` (default), `median` or `max`.
5. Multiply every measured A by the factor and fit again.

## Steps

`examples/survey.csv` holds eight survey points with columns A, Ci, Qabs
and Tleaf:

```bash
pk fit photosynthesis examples/aci.csv --model fvcb --set gm=1000000 --fix gm,alphaG --survey examples/survey.csv
```

The result has a `rescale` section:

```json
"rescale": {
  "factor": 1.147867,
  "refCi": 1602.84,
  "refQ": 1500,
  "refT": 301.15,
  "stat": "mean",
  "surveyPoints": 8,
  "firstPass": { "r2": ..., "rmse": ..., "params": { ... } }
}
```

`factor` is the multiplier applied to A. `firstPass` holds the fit before
rescaling. `params`, `r2` and `rmse` describe the rescaled fit.

Give several survey files by repeating `--survey`. Their points are pooled.
With several curve files, each curve is rescaled by its own factor.
