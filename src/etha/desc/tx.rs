use super::buffer::FrameDesc;
use crate::desc::*;
use etha_model_generator::*;

pub const TX_REQ_ENTRY_SIZE: usize = 32;
pub const TX_STATUS_ENTRY_SIZE: usize = 8;
pub const TX_CTRL_ENTRY_SIZE: usize = 4;
mod bitfields {
    use super::*;
    use bitfield::bitfield;

    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct TxCtrlDesc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub resp_en, set_resp_en: 0, 0;
    }

    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct TxStatusDesc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub too_large, set_too_large: 0, 0;
        pub too_small, set_too_small: 1, 1;
    }
}
pub const TX_RESULT_ENTRY_SIZE: usize = TX_STATUS_ENTRY_SIZE;
pub type TxStatusDesc =
    bitfields::TxStatusDesc<[DescEntryT; TX_STATUS_ENTRY_SIZE / DESC_ENTRY_SIZE]>;
pub type TxCtrlDesc = bitfields::TxCtrlDesc<[DescEntryT; TX_CTRL_ENTRY_SIZE / DESC_ENTRY_SIZE]>;

#[desc_gen(padding_to = TX_RESULT_ENTRY_SIZE)]
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct TxResultDesc {
    status: TxStatusDesc,
}
impl std::ops::Deref for TxResultDesc {
    type Target = TxStatusDesc;
    fn deref(&self) -> &Self::Target {
        &self.status
    }
}

impl std::ops::DerefMut for TxResultDesc {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.status
    }
}

#[desc_gen(padding_to = TX_REQ_ENTRY_SIZE)]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TxReqDesc {
    pub frame: FrameDesc,
    pub ctrl: TxCtrlDesc,
}
