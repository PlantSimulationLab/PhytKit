//! Units and meanings of model inputs and responses, as `pk describe` reports them.
//!
//! Parameter units live on each `ParamSpec`. Inputs are shared between models (`Ci`, `Q`,
//! `T`, ...), so their units are kept here, with per-model entries where a meaning differs.

use crate::model::InputSpec;

const fn spec(name: &'static str, units: &'static str, description: &'static str) -> InputSpec {
    InputSpec {
        name,
        units,
        description,
    }
}

/// Units and meaning of input `name` of model `model`. An input not listed here has empty
/// units and description; a unit test checks every built-in model's inputs are listed.
pub fn input(model: &str, name: &'static str) -> InputSpec {
    match (model, name) {
        ("bta", "Q") => spec(
            "Q",
            "µmol m⁻² s⁻¹",
            "light (PPFD); pk fit stomatal prefers incident Qin over Qabs",
        ),
        (_, "Q") => spec("Q", "µmol m⁻² s⁻¹", "absorbed light (PPFD)"),
        ("fvcbC4", "Ci") => spec("Ci", "µmol mol⁻¹", "intercellular CO2 (read as µbar)"),
        (_, "Ci") => spec("Ci", "µmol mol⁻¹", "intercellular CO2"),
        (_, "T") => spec(
            "T",
            "K",
            "leaf temperature in kelvin; pk fit converts a °C column, the array functions do not",
        ),
        (_, "Ds") => spec(
            "Ds",
            "mmol mol⁻¹",
            "leaf-to-air vapour pressure deficit as a mole fraction",
        ),
        (_, "A") => spec("A", "µmol m⁻² s⁻¹", "net CO2 assimilation"),
        (_, "Cs") => spec("Cs", "µmol mol⁻¹", "CO2 at the leaf surface"),
        (_, "D") => spec("D", "kPa", "leaf-to-air vapour pressure deficit"),
        (_, "P") => spec("P", "MPa", "xylem water potential (negative)"),
        (_, "w") => spec("w", "dimensionless", "relative water content, 0–1"),
        (_, "psi") => spec(
            "psi",
            "MPa",
            "matric potential (negative); pk fit soil converts other units to MPa",
        ),
        _ => spec(name, "", ""),
    }
}

/// The quantity model `model` predicts.
pub fn response(model: &str) -> InputSpec {
    match model {
        "fvcb" | "fvcbC4" | "empiricalA" | "lightResponse" => {
            spec("A", "µmol m⁻² s⁻¹", "net CO2 assimilation")
        }
        "bta" | "medlyn" => spec("gs", "mol m⁻² s⁻¹", "stomatal conductance to water vapour"),
        "vulnerability" => spec(
            "K",
            "units of the K column (dimensionless when fitted from PLC)",
            "hydraulic conductance",
        ),
        "pvcurve" => spec("psi", "MPa", "leaf water potential"),
        "vanGenuchten" | "modifiedVanGenuchten" | "raats" | "brooksCorey" | "fractalSWRC" => {
            spec("theta", "m³ m⁻³", "volumetric water content")
        }
        _ => spec("y", "", ""),
    }
}

#[cfg(test)]
mod tests {
    use crate::models::builtin_models;

    #[test]
    fn every_parameter_input_and_response_has_units_and_a_description() {
        for m in builtin_models() {
            for p in m.params() {
                assert!(!p.units.is_empty(), "{}.{} has no units", m.name(), p.name);
                assert!(
                    p.description.is_some_and(|d| !d.trim().is_empty()),
                    "{}.{} has no description",
                    m.name(),
                    p.name
                );
            }
            for i in m.input_specs() {
                assert!(
                    !i.units.is_empty() && !i.description.is_empty(),
                    "{} input {} undocumented",
                    m.name(),
                    i.name
                );
            }
            let r = m.response();
            assert!(
                !r.units.is_empty() && !r.description.is_empty(),
                "{} response undocumented",
                m.name()
            );
        }
    }

    #[test]
    fn temperature_optima_are_kelvin() {
        for m in builtin_models() {
            for p in m.params() {
                if p.name.ends_with("_Topt") {
                    assert_eq!(p.units, "K", "{}.{}", m.name(), p.name);
                    assert!(
                        p.description.unwrap().contains("kelvin"),
                        "{}.{}",
                        m.name(),
                        p.name
                    );
                }
            }
        }
    }
}
