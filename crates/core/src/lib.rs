//! PhytKit core: plant physiology models, the fitter, file parsers and LI-600 tools.

// Numeric code here steps through several parallel arrays by index, which reads more
// clearly than zipped iterators.
#![allow(clippy::needless_range_loop)]
// `!(x >= y)` is used deliberately where NaN must count as failing the comparison.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

pub mod concat;
pub mod dual;
pub mod fit;
pub mod group_csv;
pub mod hydraulics;
pub mod li600;
pub mod licor;
pub mod linalg;
pub mod model;
pub mod models;
pub mod numbers;
pub mod parse;
pub mod poro;
pub mod rescale;
pub mod table;
pub mod tresp;
pub mod units;
pub mod warn;
pub mod xlsx;

pub use fit::{fit, FitOptions, FitResult};
pub use model::{Columns, Model, ParamSpec, Params};
