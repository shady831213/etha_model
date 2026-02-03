use etha_model_generator::*;
pub const GLOBAL_RX_EN_OFFSET: usize = 0x0;
pub const GLOBAL_TX_EN_OFFSET: usize = 0x1;
define_reg! {
    EthaEn {
        fields {
            en(RW): 0, 0;
        }
    }
}

reg_map! {
    pub EthaGlobalRegs(1024) {
        rx_en(RW): EthaEn, 0;
        tx_en(RW): EthaEn, 1;
    }
}
