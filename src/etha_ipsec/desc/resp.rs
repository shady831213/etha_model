use crate::desc::*;
use etha_model_generator::*;
pub const IPSEC_STATUS_ENTRY_SIZE: usize = 8;
mod bitfields {
    use super::*;
    use bitfield::bitfield;
    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct IpsecStatusDesc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub src_err, set_src_err: 0, 0;
        pub dst_err, set_dst_err: 1, 1;
        pub invalid_session, set_invalid_session: 2, 2;
        pub ciper_err, set_ciper_err: 3, 3;
        pub auth_fail, set_auth_fail: 4, 4;
    }
}
pub const IPSEC_RESULT_ENTRY_SIZE: usize = IPSEC_STATUS_ENTRY_SIZE;
pub type IpsecStatusDesc =
    bitfields::IpsecStatusDesc<[DescEntryT; IPSEC_STATUS_ENTRY_SIZE / DESC_ENTRY_SIZE]>;

impl IpsecStatusDesc {
    pub fn is_err(&self) -> bool {
        self.src_err() != 0
            || self.dst_err() != 0
            || self.invalid_session() != 0
            || self.ciper_err() != 0
            || self.auth_fail() != 0
    }
}
#[desc_gen(padding_to = IPSEC_RESULT_ENTRY_SIZE)]
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct IpsecResultDesc {
    pub status: IpsecStatusDesc,
}
impl std::ops::Deref for IpsecResultDesc {
    type Target = IpsecStatusDesc;
    fn deref(&self) -> &Self::Target {
        &self.status
    }
}

impl std::ops::DerefMut for IpsecResultDesc {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.status
    }
}
