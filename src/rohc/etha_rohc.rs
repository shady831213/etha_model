use super::etha_rohc_core::EthaRohcCore;
use super::reg_if::TopRegs;
use super::*;
use crate::aborter::*;
use crate::arbiter::*;
use crate::irq::*;
use core_affinity::{set_for_current, CoreId};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct EthaRohc<A: Arbiter> {
    core: EthaRohcCore<A>,
    regs: Arc<TopRegs<ROHC_CH_NUM>>,
}

impl<A: Arbiter> EthaRohc<A> {
    pub fn new(arbiter: A) -> Self {
        let regs = Arc::new(TopRegs::new());
        EthaRohc {
            core: EthaRohcCore::new(arbiter, &regs),
            regs,
        }
    }

    pub fn abort(&self) -> Arc<Aborter> {
        self.core.abort()
    }
    pub fn regs(&self) -> Arc<TopRegs<ROHC_CH_NUM>> {
        self.regs.clone()
    }
    pub fn irqs(&self) -> Arc<Mutex<IrqVec>> {
        self.core.irqs()
    }
}

impl<A: Arbiter + Send + 'static> EthaRohc<A> {
    pub fn spawn(mut self, core_id: Option<CoreId>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            if let Some(id) = core_id {
                set_for_current(id);
            }
            self.core.run();
        })
    }
}
