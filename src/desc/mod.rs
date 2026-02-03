mod buffer;
pub type DescEntryT = u32;

pub const DESC_ENTRY_SIZE: usize = std::mem::size_of::<DescEntryT>();
pub use buffer::*;
