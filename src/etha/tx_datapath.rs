use super::reg_if::TopRegs;
use super::tx_sequencer::*;
use super::*;
use crate::arbiter::*;
use crate::irq::*;
use std::sync::{Arc, Mutex};
pub struct EthaTxDataPath<A: Arbiter> {
    seqr: EthaTxSequencer<A>,
}

impl<A: Arbiter> EthaTxDataPath<A> {
    pub fn new(
        arbiter: A,
        regs: &TopRegs<CHS, RX_ET_FILTERS, RX_TP5_FILTERS>,
        irqs: &Arc<Mutex<IrqVec>>,
    ) -> Self {
        EthaTxDataPath {
            seqr: EthaTxSequencer::new(arbiter, regs, irqs),
        }
    }
    pub fn pipeline<'a>(&'a mut self) -> impl Pipeline<Input = (), Output = TxLoadInfo> + 'a {
        self.seqr.pipeline()
    }
}
