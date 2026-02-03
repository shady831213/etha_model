use super::buffer::FrameDesc;
use crate::desc::*;
use etha_model_generator::*;
pub const RX_DESC_ENTRY_SIZE: usize = 128;

const L2_DESC_SIZE: usize = 24;
const L3_DESC_SIZE: usize = 40;
const L4_DESC_SIZE: usize = 12;
pub const RX_STATUS_ENTRY_SIZE: usize = 4;
mod bitfields {
    use super::*;
    use bitfield::bitfield;

    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct RxResultL2Desc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub l2_src_lo, set_l2_src_lo: 31, 0;
        pub l2_src_hi, set_l2_src_hi: 47, 32;
        pub l2_vlan_flags, set_l2_vlan_flags: 51, 48;
        pub l2_vlan_vid, set_l2_vlan_vid: 63, 52;
        pub l2_dst_lo, set_l2_dst_lo: 95, 64;
        pub l2_dst_hi, set_l2_dst_hi: 111, 96;
        pub l2_etype, set_l2_etype: 127, 112;
        pub l2_header_len, set_l2_header_len: 135, 128;
        pub l2_is_vlan, set_l2_is_vlan: 136, 136;
        pub l2_payload_len, set_l2_payload_len: 183, 160;
    }

    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct RxResultL3Desc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub l3_src, set_l3_src: 31, 0;
        pub l3_src1, set_l3_src1: 63, 32;
        pub l3_src2, set_l3_src2: 95, 64;
        pub l3_src3, set_l3_src3: 127, 96;
        pub l3_dst, set_l3_dst: 159, 128;
        pub l3_dst1, set_l3_dst1: 191, 160;
        pub l3_dst2, set_l3_dst2: 223, 192;
        pub l3_dst3, set_l3_dst3: 255, 224;
        pub l3_protocol, set_l3_protocol: 263, 256;
        pub l3_version, set_l3_version: 271, 264;
        pub l3_header_len, set_l3_header_len: 287, 272;
        pub l3_payload_len, set_l3_payload_len: 311, 288;
    }

    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct RxResultL4Desc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub l4_src_port, set_l4_src_port: 15, 0;
        pub l4_dst_port, set_l4_dst_port: 31, 16;
        pub l4_header_len, set_l4_header_len: 47, 32;
        pub l4_payload_len, set_l4_payload_len: 87, 64;
    }

    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct RxStatusDesc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub too_large, set_too_large: 0, 0;
    }
}

pub type RxResultL2Desc = bitfields::RxResultL2Desc<[DescEntryT; L2_DESC_SIZE / DESC_ENTRY_SIZE]>;
pub type RxResultL3Desc = bitfields::RxResultL3Desc<[DescEntryT; L3_DESC_SIZE / DESC_ENTRY_SIZE]>;
pub type RxResultL4Desc = bitfields::RxResultL4Desc<[DescEntryT; L4_DESC_SIZE / DESC_ENTRY_SIZE]>;
pub type RxStatusDesc =
    bitfields::RxStatusDesc<[DescEntryT; RX_STATUS_ENTRY_SIZE / DESC_ENTRY_SIZE]>;

#[desc_gen(padding_to = RX_DESC_ENTRY_SIZE)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RxResultDesc {
    pub frame: FrameDesc,
    pub l2: RxResultL2Desc,
    pub l3: RxResultL3Desc,
    pub l4: RxResultL4Desc,
    pub status: RxStatusDesc,
}
