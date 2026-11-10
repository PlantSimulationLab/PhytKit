//! `pk fit --ci-based`: FvCB with no mesophyll resistance (gm infinite, Cc = Ci) and
//! alphaG = 0, the formulation Helios's photosynthesis plugin uses by default.

use phytkit_cli::{run, Output};
use serde_json::Value;

fn examples(name: &str) -> String {
    format!("{}/../../examples/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn pk(line: &str) -> Result<serde_json::Map<String, Value>, phytkit_cli::Error> {
    let argv: Vec<String> = line
        .split_whitespace()
        .map(|a| match a.strip_prefix("examples/") {
            Some(f) => examples(f),
            None => a.to_string(),
        })
        .collect();
    match run(&argv)? {
        Output::Json(o) => Ok(o),
        Output::Raw(s) => panic!("expected JSON, got {s}"),
    }
}

#[test]
fn matches_a_very_large_fixed_gm() {
    let ci = pk("fit photosynthesis examples/aci.csv --ci-based").unwrap();
    let big = pk("fit photosynthesis examples/aci.csv --set gm=1000000 --fix gm,alphaG").unwrap();
    assert_eq!(ci["ciBased"], Value::Bool(true));
    assert!(ci["params"]["gm"].is_null(), "infinite gm is written as null");
    assert_eq!(ci["params"]["alphaG"], 0.0);
    let fitted: Vec<&str> = ci["fitted"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert!(!fitted.contains(&"gm") && !fitted.contains(&"alphaG"));
    for k in ["Vcmax25", "Jmax25", "TPU25", "Rd25", "alpha", "theta"] {
        let (a, b) = (ci["params"][k].as_f64().unwrap(), big["params"][k].as_f64().unwrap());
        assert!((a - b).abs() <= 1e-4 * b.abs(), "{k}: {a} vs {b}");
    }
    // Without the flag the key is absent.
    assert!(!big.contains_key("ciBased"));
}

#[test]
fn only_fvcb_and_no_conflicting_gm_or_alphag_options() {
    let e = pk("fit photosynthesis examples/aci.csv --model fvcbC4 --ci-based").unwrap_err();
    assert_eq!(e.code, "not_applicable");
    for extra in ["--fix gm", "--set alphaG=0.1 --fix alphaG", "--fit Vcmax25,gm", "--set gm=5"] {
        let e = pk(&format!("fit photosynthesis examples/aci.csv --ci-based {extra}")).unwrap_err();
        assert_eq!(e.code, "invalid_combination", "{extra}");
    }
}

#[test]
fn li600_estimate_keeps_a_ci_based_fit_free_of_gm() {
    let fit = pk("fit photosynthesis examples/aci.csv --ci-based").unwrap();
    let dir = std::env::temp_dir().join(format!("phytkit-ci-based-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fit.json");
    std::fs::write(&path, Value::Object(fit).to_string()).unwrap();
    let est = pk(&format!("li600 estimate examples/li600.csv --params {}", path.display())).unwrap();
    std::fs::remove_dir_all(&dir).ok();
    assert!(est["settings"]["gm"].is_null());
    assert_eq!(est["settings"]["series"], Value::Bool(false));
}
