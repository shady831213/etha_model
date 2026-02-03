use crate::reg_if::{ring::*, RegBus};
use etha_model_generator::*;
use std::sync::Arc;
pub const QUEUE_REGS_RANGE: std::ops::Range<usize> = 0..1024;

pub const fn queue_base(i: usize) -> usize {
    QUEUE_REGS_RANGE.start + i * RING_REGS_SIZE
}
pub struct TopRegs<const CHS: usize> {
    pub chs: [Arc<LockedRingRegs>; CHS],
}
impl<const CHS: usize> TopRegs<CHS> {
    pub fn new() -> Self {
        TopRegs {
            chs: array_init::array_init(|_| Arc::new(LockedRingRegs::new(32))),
        }
    }
}

impl<const CHS: usize> RegBus for TopRegs<CHS> {
    fn write(&self, addr: u64, data: u64) -> Option<()> {
        let offset = addr as usize;
        if QUEUE_REGS_RANGE.contains(&offset) {
            let offset = offset - QUEUE_REGS_RANGE.start;
            let idx = offset / RING_REGS_SIZE;
            if idx < self.chs.len() {
                let r = self.chs[idx].write((offset % RING_REGS_SIZE) as u64, data);
                self.chs[idx].r_update_status();
                r
            } else {
                None
            }
        } else {
            None
        }
    }

    fn read(&self, addr: u64) -> Option<u64> {
        let offset = addr as usize;
        if QUEUE_REGS_RANGE.contains(&offset) {
            let offset = offset - QUEUE_REGS_RANGE.start;
            let idx = offset / RING_REGS_SIZE;
            if idx < self.chs.len() {
                self.chs[idx].r_update_status();
                self.chs[idx].read((offset % RING_REGS_SIZE) as u64)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<const CHS: usize> GenHeader for TopRegs<CHS> {
    fn render_name() -> &'static str {
        "TopRegs"
    }
    fn gen_c_header<W: std::io::Write>(header: &mut W) -> std::io::Result<()> {
        writeln!(header, "#define QUEUE_NUM {}", CHS)?;
        writeln!(
            header,
            "#define QUEUE_REGS_OFFSET {:#x}",
            QUEUE_REGS_RANGE.start
        )?;

        writeln!(
            header,
            "#define RING(name, i) (QUEUE_REGS_OFFSET + (RING_REGS_SIZE * i) + RING_REGS_##name##_OFFSET)",
        )?;

        Ok(())
    }
}
