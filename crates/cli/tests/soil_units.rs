//! Soil ψ columns named with their unit ("Psi (kPa)", "h_cm") are found, and read in that
//! unit, end to end through `pk fit soil`.

use phytkit_cli::{run, Output};
use serde_json::{Map, Value};

fn example() -> String {
    std::fs::read_to_string(format!(
        "{}/../../examples/soil.csv",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

/// `examples/soil.csv` with its `psi` header renamed, fitted with `pk fit soil` plus `extra`.
fn fit_with_header(header: &str, extra: &[&str]) -> Map<String, Value> {
    let text = example().replacen("sample,psi,theta", &format!("sample,{header},theta"), 1);
    let dir = std::env::temp_dir().join(format!("phytkit-soil-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let safe: String = header
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let path = dir.join(format!("soil_{safe}.csv"));
    std::fs::write(&path, text).unwrap();
    let mut argv = vec!["fit".to_string(), "soil".into(), path.display().to_string()];
    argv.extend(["--group-by", "sample"].iter().map(|s| s.to_string()));
    argv.extend(extra.iter().map(|s| s.to_string()));
    let out = run(&argv).unwrap_or_else(|e| panic!("{header}: {} ({:?})", e.message, e.hint));
    std::fs::remove_file(&path).ok();
    match out {
        Output::Json(o) => o,
        Output::Raw(s) => panic!("expected JSON, got {s}"),
    }
}

fn first_fit(o: &Map<String, Value>) -> &Value {
    &o["results"][0]
}

#[test]
fn psi_columns_named_with_a_unit_are_found_and_converted() {
    // The example's ψ values are in kPa; --units kPa on the plain name is the reference.
    let reference = fit_with_header("psi", &["--units", "kPa"]);
    for header in [
        "Psi (kPa)",
        "psi_kPa",
        "PSI kPa",
        "Ψ (kPa)",
        "matric potential (kPa)",
        "h (kPa)",
    ] {
        let o = fit_with_header(header, &[]);
        let fit = first_fit(&o);
        assert_eq!(
            fit["mapping"]["psi"],
            Value::String(header.into()),
            "{header}"
        );
        let converted = fit["warnings"].as_array().unwrap().iter().any(|w| {
            w["code"] == "unit_converted_by_name" && w["message"].as_str().unwrap().contains("kPa")
        });
        assert!(
            converted,
            "{header}: no kPa conversion in {}",
            fit["warnings"]
        );
        assert_eq!(fit["params"], first_fit(&reference)["params"], "{header}");
    }
}

#[test]
fn every_unit_suffix_is_read_in_its_unit() {
    for (header, unit) in [
        ("h_cm", "cm"),
        ("h (cm)", "cm"),
        ("head_m", "m"),
        ("Psi (hPa)", "hPa"),
        ("psi_bar", "bar"),
    ] {
        let o = fit_with_header(header, &[]);
        let fit = first_fit(&o);
        assert_eq!(
            fit["mapping"]["psi"],
            Value::String(header.into()),
            "{header}"
        );
        let msg = fit["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|w| w["code"] == "unit_converted_by_name")
            .unwrap_or_else(|| panic!("{header}: no conversion warning"))["message"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(
            msg.contains(&format!("is in {unit} by its name")),
            "{header}: {msg}"
        );
    }
}

#[test]
fn a_psi_column_in_mpa_is_not_converted() {
    let o = fit_with_header("Psi (MPa)", &[]);
    let fit = first_fit(&o);
    assert_eq!(fit["mapping"]["psi"], Value::String("Psi (MPa)".into()));
    assert!(fit["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|w| w["code"] != "unit_converted_by_name"));
}
