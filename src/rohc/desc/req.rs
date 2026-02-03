use crate::desc::*;
use etha_model_generator::*;
pub const ROHC_CFG_DESC_ENTRY_SIZE: usize = 4;
mod bitfields {
    use super::*;
    use bitfield::bitfield;
    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct RohcCfgDesc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub v2, set_v2: 0, 0;
        pub decomp, set_decomp: 1, 1;
        pub resp_en, set_resp_en: 31, 31;
    }
}
pub type RohcCfgDesc =
    bitfields::RohcCfgDesc<[DescEntryT; ROHC_CFG_DESC_ENTRY_SIZE / DESC_ENTRY_SIZE]>;

pub const ROHC_REQ_ENTRY_SIZE: usize = 32;

#[desc_gen(padding_to = ROHC_REQ_ENTRY_SIZE)]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RohcReqDesc {
    pub src: SCFrameDesc,
    pub dst: SCFrameDesc,
    pub cfg: RohcCfgDesc,
}
