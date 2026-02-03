mod ch;
pub mod desc;
mod engine;
mod etha_ipsec;
mod etha_ipsec_core;
#[cfg(not(test))]
mod ffi;
mod pipeline;
pub mod reg_if;
mod session_cache;
use pipeline::{Error as PipeError, EthaIrqs, Pipeline, Result as PipeResult};
pub const IPSEC_CH_NUM: usize = 4;
pub const IPSEC_SESSION_NUM: usize = 64;
pub const IPSEC_CACHE_NUM: usize = 8;
pub use etha_ipsec::*;
pub const STATICS_TAR: &str = "etha_ipsec";
