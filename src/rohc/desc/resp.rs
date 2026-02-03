use crate::desc::*;
use etha_model_generator::*;
pub const ROHC_STATUS_ENTRY_SIZE: usize = 4;
mod bitfields {
    use super::*;
    use bitfield::bitfield;
    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct RohcStatusDesc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub src_err, set_src_err: 0, 0;
        pub dst_err, set_dst_err: 1, 1;
        pub too_small, set_too_small: 2, 2;
        pub bad_crc, set_bad_crc: 3, 3;
        pub no_ctx, set_no_ctx: 4, 4;
        pub bad_fmt, set_bad_fmt: 5, 5;
        pub len, set_len: 31, 16;
    }
}
pub const ROHC_RESULT_ENTRY_SIZE: usize = ROHC_STATUS_ENTRY_SIZE;
pub type RohcStatusDesc =
    bitfields::RohcStatusDesc<[DescEntryT; ROHC_STATUS_ENTRY_SIZE / DESC_ENTRY_SIZE]>;

impl RohcStatusDesc {
    pub fn is_err(&self) -> bool {
        self.src_err() != 0
            || self.dst_err() != 0
            || self.too_small() != 0
            || self.bad_crc() != 0
            || self.no_ctx() != 0
            || self.bad_fmt() != 0
    }
}
#[desc_gen(padding_to = ROHC_RESULT_ENTRY_SIZE)]
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct RohcResultDesc {
    pub status: RohcStatusDesc,
}
impl std::ops::Deref for RohcResultDesc {
    type Target = RohcStatusDesc;
    fn deref(&self) -> &Self::Target {
        &self.status
    }
}

impl std::ops::DerefMut for RohcResultDesc {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.status
    }
}
