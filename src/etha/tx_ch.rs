use super::desc::tx::*;
use super::STATICS_TAR;
use crate::desc::*;
use crate::irq::*;
use crate::logger;
use crate::reg_if::ring::*;
use std::io::Read;
use std::ops::Deref;
use std::sync::Arc;
pub struct EthaTxCh {
    id: usize,
    irq_num: usize,
    ring: Arc<LockedRingRegs>,
}

impl EthaTxCh {
    pub fn new(id: usize, ring: &Arc<LockedRingRegs>, irqs: &mut IrqVec) -> Self {
        EthaTxCh {
            id,
            irq_num: irqs.alloc(&format!("EthaTxChIrq{}", id)),
            ring: ring.clone(),
        }
    }
    pub fn req(&self) -> Option<TxReqDesc> {
        if self.r_c_valids() > 0 {
            let req = self.r_get_req();
            assert!(
                req.frame.start() == 1,
                "tx[{}]: head is not start frame! {:#x?}",
                self.id,
                req
            );
            if self.r_c_valids() > req.frame.n_blocks() as usize {
                Some(req)
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn read(&self, data: &mut [u8]) {
        let head = self.r_get_req();
        let mut cnt = 0;
        for (i, (req, _)) in self.entries().enumerate() {
            let req = unsafe { *req };
            tracing::event!(
                target: STATICS_TAR,
                logger::STATICS_LEVEL,
                name = "tx read desc req",
                size = <Self as HwRing>::REQ_SIZE
            );
            let mut b = MemBlock {
                addr: req.frame.full_addr(),
                size: req.frame.size() as usize,
            };
            tracing::event!(
                target: STATICS_TAR,
                logger::STATICS_LEVEL,
                name = "tx read data",
                addr = b.addr,
                size = b.size
            );
            cnt += b
                .read(&mut data[cnt..])
                .expect(&format!("tx[{}]: read error!", self.id));
            if i == head.frame.n_blocks() as usize {
                assert!(
                    req.frame.end() == 1,
                    "tx[{}]: expect end frame flag! head : {:#x?}, end: {:#x?}",
                    self.id,
                    head,
                    req
                );
                assert_eq!(
                    cnt,
                    head.frame.total_size() as usize,
                    "tx[{}]: scatter-gather length error! expect: {}, actaul: {}",
                    self.id,
                    head.frame.total_size(),
                    cnt
                );
                return;
            }
        }
        unreachable!("tx[{}]: no end frame found! head: {:#x?}", self.id, head);
    }

    pub fn write_resp(&self, resp: &Option<TxResultDesc>) {
        let blocks = self.r_get_req().frame.n_blocks() as usize;
        if let Some(resp) = resp {
            tracing::event!(
                target: STATICS_TAR,
                logger::STATICS_LEVEL,
                name = "tx write desc resp",
                size = <Self as HwRing>::RESP_SIZE
            );
            if self.r_set_resp(resp).is_none() {
                println!("tx[{}]: Warning! resp has been ignored!", self.id);
            };
        }
        self.r_advance_c_n(blocks + 1);
    }
}

impl WithIrq for EthaTxCh {
    fn poll_irq(&self) -> Option<usize> {
        if self.r_irq_pendings() == 0 {
            None
        } else {
            Some(self.irq_num)
        }
    }
}

impl HwRing for EthaTxCh {
    type REQ = TxReqDesc;
    type RESP = TxResultDesc;
    type R = LockedRingRegs;
    const REQ_SIZE: usize = TX_REQ_ENTRY_SIZE;
    const RESP_SIZE: usize = TX_RESULT_ENTRY_SIZE;

    fn get_ring(&self) -> &Self::R {
        &self.ring
    }
}

impl Deref for EthaTxCh {
    type Target = LockedRingRegs;
    fn deref(&self) -> &Self::Target {
        self.get_ring()
    }
}
