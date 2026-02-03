use super::desc::req::*;
use super::desc::resp::*;
use crate::irq::*;
use crate::reg_if::ring::*;
use std::ops::Deref;
use std::sync::Arc;
pub struct EthaIpsecCh {
    id: usize,
    irq_num: usize,
    ring: Arc<LockedRingRegs>,
}

impl EthaIpsecCh {
    pub fn new(id: usize, ring: &Arc<LockedRingRegs>, irqs: &mut IrqVec) -> Self {
        EthaIpsecCh {
            id,
            irq_num: irqs.alloc(&format!("EthaIpsecChIrq{}", id)),
            ring: ring.clone(),
        }
    }
    pub fn req(&self) -> Option<IpsecReqDesc> {
        if self.r_c_valids() > 0 {
            let req = self.r_get_req();
            Some(req)
        } else {
            None
        }
    }
    pub fn resp(&self, resp: &IpsecResultDesc) {
        if self.r_set_resp(resp).is_none() {
            println!("Ipsec Ch[{}]: Warning! resp has been ignored!", self.id);
        }
        self.r_advance_c();
    }
}

impl WithIrq for EthaIpsecCh {
    fn poll_irq(&self) -> Option<usize> {
        if self.r_irq_pendings() == 0 {
            None
        } else {
            Some(self.irq_num)
        }
    }
}

impl HwRing for EthaIpsecCh {
    type REQ = IpsecReqDesc;
    type RESP = IpsecResultDesc;
    type R = LockedRingRegs;
    const REQ_SIZE: usize = IPSEC_REQ_ENTRY_SIZE;
    const RESP_SIZE: usize = IPSEC_RESULT_ENTRY_SIZE;

    fn get_ring(&self) -> &Self::R {
        &self.ring
    }
}

impl Deref for EthaIpsecCh {
    type Target = LockedRingRegs;
    fn deref(&self) -> &Self::Target {
        self.get_ring()
    }
}
