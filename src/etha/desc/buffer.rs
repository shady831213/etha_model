use std::convert::From;
const FRAME_DESC_SIZE: usize = 16;
use crate::desc::*;
use etha_model_generator::*;

mod bitfields {
    use super::*;
    use bitfield::bitfield;
    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct FrameDesc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub addr_lo, set_addr_lo: 31, 0;
        pub addr_hi, set_addr_hi: 63, 32;
        pub total_size, set_total_size: 87, 64;
        pub n_blocks, set_n_blocks: 95, 88;
        pub size, set_size: 119, 96;
        pub start, set_start: 120, 120;
        pub end, set_end: 121, 121;
    }
}
pub type FrameDesc = bitfields::FrameDesc<[DescEntryT; FRAME_DESC_SIZE / DESC_ENTRY_SIZE]>;

impl FrameDesc {
    pub fn full_addr(&self) -> u64 {
        self.addr_lo() as u64 | ((self.addr_hi() as u64) << 32)
    }
}

impl From<MemBlock> for FrameDesc {
    fn from(b: MemBlock) -> Self {
        let mut d = Self::default();
        d.set_addr_lo(b.addr as DescEntryT);
        d.set_addr_hi((b.addr >> 32) as DescEntryT);
        d.set_size(b.size as DescEntryT);
        d
    }
}
