pub mod sessions;
use crate::reg_if::{ring::*, RegBus};
use etha_model_generator::*;
use sessions::*;
use std::sync::Arc;
pub const SESSION_REGS_RANGE: std::ops::Range<usize> = 0..2048;
pub const QUEUE_REGS_RANGE: std::ops::Range<usize> =
    SESSION_REGS_RANGE.end..SESSION_REGS_RANGE.end + 1024;

pub const fn queue_base(i: usize) -> usize {
    QUEUE_REGS_RANGE.start + i * RING_REGS_SIZE
}
pub struct TopRegs<const CHS: usize, const SESSIONS: usize> {
    pub chs: [Arc<LockedRingRegs>; CHS],
    pub sessions: Arc<SecSessions<SESSIONS>>,
}
impl<const CHS: usize, const SESSIONS: usize> TopRegs<CHS, SESSIONS> {
    pub fn new() -> Self {
        TopRegs {
            chs: array_init::array_init(|_| Arc::new(LockedRingRegs::new(32))),
            sessions: Arc::new(SecSessions::new()),
        }
    }
}

impl<const CHS: usize, const SESSIONS: usize> RegBus for TopRegs<CHS, SESSIONS> {
    fn write(&self, addr: u64, data: u64) -> Option<()> {
        let offset = addr as usize;
        if SESSION_REGS_RANGE.contains(&offset) {
            self.sessions.write(addr, data)
        } else if QUEUE_REGS_RANGE.contains(&offset) {
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
        if SESSION_REGS_RANGE.contains(&offset) {
            self.sessions.read(addr)
        } else if QUEUE_REGS_RANGE.contains(&offset) {
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

impl<const CHS: usize, const SESSIONS: usize> GenHeader for TopRegs<CHS, SESSIONS> {
    fn render_name() -> &'static str {
        "TopRegs"
    }
    fn gen_c_header<W: std::io::Write>(header: &mut W) -> std::io::Result<()> {
        SecSessions::<SESSIONS>::gen_c_header(header)?;
        writeln!(
            header,
            "#define SEC_SESSIONS_OFFSET {:#x}",
            SESSION_REGS_RANGE.start
        )?;

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

        writeln!(
            header,
            "#define SEC_SESSION(name, i) SEC_SESSION_OFFSET(SEC_SESSIONS_OFFSET, name, i)",
        )?;
        Ok(())
    }
}
