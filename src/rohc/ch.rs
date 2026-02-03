use super::desc::req::*;
use super::desc::resp::*;
use crate::irq::*;
use crate::reg_if::ring::*;
use std::ops::Deref;
use std::sync::Arc;
pub struct EthaRohcCh {
    id: usize,
    irq_num: usize,
    ring: Arc<LockedRingRegs>,
}

impl EthaRohcCh {
    pub fn new(id: usize, ring: &Arc<LockedRingRegs>, irqs: &mut IrqVec) -> Self {
        EthaRohcCh {
            id,
            irq_num: irqs.alloc(&format!("EthaRohcChIrq{}", id)),
            ring: ring.clone(),
        }
    }
    pub fn req(&self) -> Option<RohcReqDesc> {
        if self.r_c_valids() > 0 {
            let req = self.r_get_req();
            Some(req)
        } else {
            None
        }
    }
    pub fn resp(&self, resp: &RohcResultDesc) {
        if self.r_set_resp(resp).is_none() {
            println!("Rohc Ch[{}]: Warning! resp has been ignored!", self.id);
        }
        self.r_advance_c();
    }
}

impl WithIrq for EthaRohcCh {
    fn poll_irq(&self) -> Option<usize> {
        if self.r_irq_pendings() == 0 {
            None
        } else {
            Some(self.irq_num)
        }
    }
}

impl HwRing for EthaRohcCh {
    type REQ = RohcReqDesc;
    type RESP = RohcResultDesc;
    type R = LockedRingRegs;
    const REQ_SIZE: usize = ROHC_REQ_ENTRY_SIZE;
    const RESP_SIZE: usize = ROHC_RESULT_ENTRY_SIZE;

    fn get_ring(&self) -> &Self::R {
        &self.ring
    }
}

impl Deref for EthaRohcCh {
    type Target = LockedRingRegs;
    fn deref(&self) -> &Self::Target {
        self.get_ring()
    }
}
