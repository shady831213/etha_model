mod ch;
pub mod desc;
mod engine;
mod etha_rohc;
mod etha_rohc_core;
mod pipeline;
pub mod reg_if;
mod rohc_wrapper;
use pipeline::{Error as PipeError, EthaIrqs, Pipeline, Result as PipeResult};
pub const ROHC_CH_NUM: usize = 1;
pub const STATICS_TAR: &str = "etha_rohc";
pub use etha_rohc::*;
#[cfg(not(test))]
mod ffi;
