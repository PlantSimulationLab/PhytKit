//! The built-in models.
//!
//! `builtin_models()` lists them in this order, which `pk describe` reports:
//! empiricalA, fvcb, fvcbC4, vulnerability, pvcurve, vanGenuchten, modifiedVanGenuchten,
//! raats, brooksCorey, fractalSWRC, bta, medlyn, lightResponse.

pub mod brooks_corey;
pub mod bta;
pub mod empirical_a;
pub mod example_light_response;
pub mod fractal_swrc;
pub mod fvcb;
pub mod fvcb_c4;
pub mod medlyn;
pub mod modified_van_genuchten;
pub mod pvcurve;
pub mod raats;
pub mod smooth_min;
pub mod van_genuchten;
pub mod vulnerability;

pub use empirical_a::empirical_a_jacobian;
pub use fractal_swrc::{fractal_saturation, fractal_suction};
pub use raats::{raats_saturation, reg_inc_beta};

use crate::model::Model;
use std::sync::OnceLock;

pub fn builtin_models() -> &'static [Box<dyn Model>] {
    static MODELS: OnceLock<Vec<Box<dyn Model>>> = OnceLock::new();
    MODELS.get_or_init(|| {
        vec![
            Box::new(empirical_a::EmpiricalA::new()),
            Box::new(fvcb::Fvcb::new()),
            Box::new(fvcb_c4::FvcbC4::new()),
            Box::new(vulnerability::Vulnerability::new()),
            Box::new(pvcurve::Pvcurve::new()),
            Box::new(van_genuchten::VanGenuchten::new()),
            Box::new(modified_van_genuchten::ModifiedVanGenuchten::new()),
            Box::new(raats::Raats::new()),
            Box::new(brooks_corey::BrooksCorey::new()),
            Box::new(fractal_swrc::FractalSwrc::new()),
            Box::new(bta::Bta::new()),
            Box::new(medlyn::Medlyn::new()),
            Box::new(example_light_response::LightResponse::new()),
        ]
    })
}

pub fn model_by_name(name: &str) -> Option<&'static dyn Model> {
    builtin_models()
        .iter()
        .find(|m| m.name() == name)
        .map(|b| b.as_ref())
}
