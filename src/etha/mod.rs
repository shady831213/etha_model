pub mod desc;
mod etha;
mod etha_core;
#[cfg(not(test))]
mod ffi;
mod l2_parser;
mod l3_parser;
mod l4_parser;
mod parser;
mod pipeline;
pub mod reg_if;
mod rx_ch;
mod rx_datapath;
mod rx_dispatcher;
mod rx_filter;
mod tx_ch;
mod tx_datapath;
mod tx_sequencer;
pub use etha::*;
use pipeline::*;

pub const CHS: usize = 16;
pub const RX_TP5_FILTERS: usize = CHS;
pub const RX_ET_FILTERS: usize = 4;

const MIN_FRAME_LEN: usize = smoltcp::wire::ETHERNET_HEADER_LEN;
pub const STATICS_TAR: &str = "etha";
