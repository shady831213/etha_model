use crate::desc::*;
use etha_model_generator::*;
pub const FRAME_CFG_DESC_ENTRY_SIZE: usize = 8;
pub const FRAME_FMT_DESC_ENTRY_SIZE: usize = 16;
mod bitfields {
    use super::*;
    use bitfield::bitfield;
    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct IpsecFrameFmtDesc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub aad_offset, set_aad_offset: 31, 0;
        pub text_offset, set_text_offset: 63, 32;
        pub iv_offset, set_iv_offset: 95, 64;
        pub icv_offset, set_icv_offset: 127, 96;
    }
    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct IpsecFrameCfgDesc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub aad_len, set_aad_len: 23, 0;
        pub session_id, set_session_id: 31, 24;
        pub text_len, set_text_len: 55, 32;
        pub encrypt, set_encrypt: 56, 56;
        pub resp_en, set_resp_en: 57, 57;
        pub aad_copy, set_aad_copy: 58, 58;
        pub iv_copy, set_iv_copy: 59, 59;
    }
}
pub type IpsecFrameCfgDesc =
    bitfields::IpsecFrameCfgDesc<[DescEntryT; FRAME_CFG_DESC_ENTRY_SIZE / DESC_ENTRY_SIZE]>;

pub type IpsecFrameFmtDesc =
    bitfields::IpsecFrameFmtDesc<[DescEntryT; FRAME_FMT_DESC_ENTRY_SIZE / DESC_ENTRY_SIZE]>;

pub const IPSEC_REQ_ENTRY_SIZE: usize = 64;

#[desc_gen]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IpsecCfgDesc {
    pub src: IpsecFrameFmtDesc,
    pub dst: IpsecFrameFmtDesc,
    pub cfg: IpsecFrameCfgDesc,
}

#[desc_gen(padding_to = IPSEC_REQ_ENTRY_SIZE)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IpsecReqDesc {
    pub src: SCFrameDesc,
    pub dst: SCFrameDesc,
    pub cfg: IpsecCfgDesc,
}
