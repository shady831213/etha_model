use super::parser::*;
use super::reg_if::TopRegs;
use super::rx_dispatcher::*;
use super::rx_filter::*;
use super::*;
use crate::irq::*;
use std::sync::{Arc, Mutex};
pub struct EthaRxDataPath {
    pub dispather: EthaRxDispatcher,
    pub filter: EthaRxFilter,
}
impl EthaRxDataPath {
    pub fn new(
        regs: &TopRegs<CHS, RX_ET_FILTERS, RX_TP5_FILTERS>,
        irqs: &Arc<Mutex<IrqVec>>,
    ) -> Self {
        EthaRxDataPath {
            dispather: EthaRxDispatcher::new(regs, irqs),
            filter: EthaRxFilter::new(&regs.rx.filters),
        }
    }
    pub fn pipeline<'a>(&'a self) -> impl Pipeline<Input = (), Output = ()> + 'a {
        EthaRxParser
            .pipeline()
            .comb(self.filter.pipeline())
            .comb(self.dispather.pipeline())
    }
}
