use super::desc::buffer::*;
use super::desc::rx::*;
use super::parser::ParserInfo;
use super::STATICS_TAR;
use crate::desc::*;
use crate::irq::*;
use crate::logger;
use crate::reg_if::ring::*;
use std::io::Write;
use std::ops::Deref;
use std::sync::Arc;
pub struct EthaRxCh {
    id: usize,
    irq_num: usize,
    ring: Arc<LockedRingRegs>,
}

impl EthaRxCh {
    pub fn new(id: usize, ring: &Arc<LockedRingRegs>, irqs: &mut IrqVec) -> Self {
        EthaRxCh {
            id,
            irq_num: irqs.alloc(&format!("EthaRxChIrq{}", id)),
            ring: ring.clone(),
        }
    }
    fn mem_size_avail(&self) -> usize {
        self.mem_size().size() as usize * self.r_c_valids()
    }
    pub fn write(&self, info: ParserInfo, data: &[u8]) -> Option<()> {
        if self.mem_size_avail() >= data.len() {
            let span = tracing::span!(
                target: STATICS_TAR,
                logger::STATICS_LEVEL,
                "rx recive packet",
                id = self.id
            );
            let _enter = span.enter();
            let mut blocks = 0;
            for (i, (req, resp)) in self.entries().enumerate() {
                tracing::event!(
                    target: STATICS_TAR,
                    logger::STATICS_LEVEL,
                    name = "rx read desc req",
                    size = <Self as HwRing>::REQ_SIZE
                );
                let addr = unsafe { *req };
                let desc =
                    unsafe { &mut *resp.expect(&format!("rx[{}]: resp is disabled!", self.id)) };
                let pos = self.mem_size().size() as usize * i;
                let (len, end) = if data.len() - pos > self.mem_size().size() as usize {
                    (self.mem_size().size() as usize, false)
                } else {
                    (data.len() - pos, true)
                };
                let start = i == 0;
                let mut b = MemBlock { addr, size: len };
                tracing::event!(
                    target: STATICS_TAR,
                    logger::STATICS_LEVEL,
                    name = "rx write data",
                    addr = b.addr,
                    size = b.size
                );
                b.write(&data[pos..pos + len])
                    .expect(&format!("rx[{}]: write error!", self.id));
                desc.frame = FrameDesc::from(b);
                desc.frame.set_start(start as u32);
                desc.frame.set_end(end as u32);
                if end {
                    blocks = i;
                    break;
                }
            }
            let head = unsafe {
                &mut *self
                    .r_get_resp_at(self.r_c_ptr())
                    .expect(&format!("rx[{}]: resp is disabled!", self.id))
            };
            head.frame.set_n_blocks(blocks as u32);
            head.frame.set_total_size(data.len() as u32);
            self.set_info(head, info);
            tracing::event!(
                target: STATICS_TAR,
                logger::STATICS_LEVEL,
                name = "rx write desc resp",
                size = <Self as HwRing>::RESP_SIZE
            );
            self.r_advance_c_n(blocks + 1);
            tracing::event!(
                target: STATICS_TAR,
                logger::STATICS_LEVEL,
                name = "rx receive done",
                size = data.len()
            );
            Some(())
        } else {
            None
        }
    }
    fn set_info(&self, desc: &mut RxResultDesc, info: ParserInfo) {
        let l2_payload_len = desc.frame.total_size() as usize - info.l2.header_len;
        let l3_payload_len = if info.l3.header_len == 0 {
            0
        } else {
            l2_payload_len - info.l3.header_len
        };
        let l4_payload_len = if info.l4.header_len == 0 || l3_payload_len == 0 {
            0
        } else {
            l2_payload_len - info.l4.header_len
        };
        desc.l2 = info.l2.into();
        desc.l3 = info.l3.into();
        desc.l4 = info.l4.into();
        desc.l2.set_l2_payload_len(l2_payload_len as u32);
        desc.l3.set_l3_payload_len(l3_payload_len as u32);
        desc.l4.set_l4_payload_len(l4_payload_len as u32);
    }
}

impl WithIrq for EthaRxCh {
    fn poll_irq(&self) -> Option<usize> {
        if self.r_irq_pendings() == 0 {
            None
        } else {
            Some(self.irq_num)
        }
    }
}

impl HwRing for EthaRxCh {
    type REQ = u64;
    type RESP = RxResultDesc;
    type R = LockedRingRegs;
    const RESP_SIZE: usize = RX_DESC_ENTRY_SIZE;
    fn get_ring(&self) -> &Self::R {
        &self.ring
    }
}

impl Deref for EthaRxCh {
    type Target = LockedRingRegs;
    fn deref(&self) -> &Self::Target {
        &self.ring
    }
}
