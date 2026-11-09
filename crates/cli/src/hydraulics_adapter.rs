//! Bridges the plant-hydraulics parsers to `FitData`.

use crate::tools::{FitData, Parser};
use phytkit_core::hydraulics::{parse_medlyn, parse_pv_curve, parse_soil_retention, parse_stomatal, parse_vulnerability, ParsedFitData};

pub fn parse(parser: Parser, text: &str) -> Result<FitData, String> {
    let d: ParsedFitData = match parser {
        Parser::Stomatal => parse_stomatal(text),
        Parser::Medlyn => parse_medlyn(text),
        Parser::Vulnerability => parse_vulnerability(text),
        Parser::PvCurve => parse_pv_curve(text),
        Parser::Soil => parse_soil_retention(text),
        Parser::Photosynthesis => unreachable!("photosynthesis uses parse_csv"),
    }?;
    Ok(FitData { x: d.x, y: d.y, mapping: d.mapping.into_iter().collect(), warnings: d.warnings, curves: None, columns: None, kmax: d.kmax })
}
