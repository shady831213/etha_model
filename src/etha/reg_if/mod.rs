pub mod global;
pub mod rx;
use crate::reg_if::{RegBus, ring::*};
use etha_model_generator::*;
use std::sync::Arc;

pub const RX_REGS_RANGE: std::ops::Range<usize> = 0..1024;
pub const TX_REGS_RANGE: std::ops::Range<usize> = RX_REGS_RANGE.end..RX_REGS_RANGE.end + 1024;
pub const QUEUE_REGS_RANGE: std::ops::Range<usize> = TX_REGS_RANGE.end..TX_REGS_RANGE.end + 2048;
pub const GLOBAL_REGS_RANGE: std::ops::Range<usize> =
    QUEUE_REGS_RANGE.end..QUEUE_REGS_RANGE.end + 1024;
pub const QUEUE_REG_SIZE: usize = RING_REGS_SIZE * 2;
pub const fn queue_base(i: usize) -> usize {
    QUEUE_REGS_RANGE.start + i * QUEUE_REG_SIZE
}
pub const fn rx_ring_base(i: usize) -> usize {
    queue_base(i)
}
pub const fn tx_ring_base(i: usize) -> usize {
    queue_base(i) + RING_REGS_SIZE
}

pub struct ChRegs {
    pub rx: Arc<LockedRingRegs>,
    pub tx: Arc<LockedRingRegs>,
}

impl ChRegs {
    pub fn new() -> Self {
        ChRegs {
            rx: Arc::new(LockedRingRegs::new(32)),
            tx: Arc::new(LockedRingRegs::new(32)),
        }
    }
}

impl RegBus for ChRegs {
    fn write(&self, addr: u64, data: u64) -> Option<()> {
        let offset = addr % (RING_REGS_SIZE as u64);
        match (addr as usize) / RING_REGS_SIZE {
            0 => {
                let r = self.rx.write(offset, data);
                self.rx.r_update_status();
                r
            }
            1 => {
                let r = self.tx.write(offset, data);
                self.tx.r_update_status();
                r
            }
            _ => None,
        }
    }

    fn read(&self, addr: u64) -> Option<u64> {
        let offset = addr % (RING_REGS_SIZE as u64);
        match (addr as usize) / RING_REGS_SIZE {
            0 => {
                self.rx.r_update_status();
                self.rx.read(offset)
            }
            1 => {
                self.tx.r_update_status();
                self.tx.read(offset)
            }
            _ => None,
        }
    }
}

impl GenHeader for ChRegs {
    fn render_name() -> &'static str {
        "ChRegs"
    }
    fn gen_c_header<W: std::io::Write>(header: &mut W) -> std::io::Result<()> {
        writeln!(header, "#define QUEUE_REGS_SIZE {:#x}", QUEUE_REG_SIZE)?;

        writeln!(header, "#define RX_RINGS_OFFSET {:#x}", 0)?;

        writeln!(
            header,
            "#define TX_RINGS_OFFSET (RX_RINGS_OFFSET + RING_REGS_SIZE)",
        )?;

        writeln!(
            header,
            "#define RX_RING_OFFSET(base, name, i) ((base) + (QUEUE_REGS_SIZE * i) + RX_RINGS_OFFSET + RING_REGS_##name##_OFFSET)",
        )?;
        writeln!(
            header,
            "#define TX_RING_OFFSET(base, name, i) ((base) + (QUEUE_REGS_SIZE * i) + TX_RINGS_OFFSET + RING_REGS_##name##_OFFSET)",
        )?;
        Ok(())
    }
}

pub struct TopRegs<const CHS: usize, const ET_FILTER_N: usize, const TP5_FILTER_N: usize> {
    pub rx: rx::RxRegs<ET_FILTER_N, TP5_FILTER_N>,
    pub chs: [ChRegs; CHS],
    pub global: Arc<global::LockedEthaGlobalRegs>,
}

impl<const CHS: usize, const ET_FILTER_N: usize, const TP5_FILTER_N: usize>
    TopRegs<CHS, ET_FILTER_N, TP5_FILTER_N>
{
    pub fn new() -> Self {
        TopRegs {
            rx: rx::RxRegs::new(),
            chs: array_init::array_init(|_| ChRegs::new()),
            global: Arc::new(global::LockedEthaGlobalRegs::new(32)),
        }
    }
}

impl<const CHS: usize, const ET_FILTER_N: usize, const TP5_FILTER_N: usize> RegBus
    for TopRegs<CHS, ET_FILTER_N, TP5_FILTER_N>
{
    fn write(&self, addr: u64, data: u64) -> Option<()> {
        let offset = addr as usize;
        if RX_REGS_RANGE.contains(&offset) {
            self.rx.write(addr, data)
        } else if QUEUE_REGS_RANGE.contains(&offset) {
            let offset = offset - QUEUE_REGS_RANGE.start;
            let idx = offset / QUEUE_REG_SIZE;
            if idx < self.chs.len() {
                self.chs[idx].write((offset % QUEUE_REG_SIZE) as u64, data)
            } else {
                None
            }
        } else if GLOBAL_REGS_RANGE.contains(&offset) {
            let offset = offset - GLOBAL_REGS_RANGE.start;
            self.global.write(offset as u64, data)
        } else {
            None
        }
    }

    fn read(&self, addr: u64) -> Option<u64> {
        let offset = addr as usize;
        if RX_REGS_RANGE.contains(&offset) {
            self.rx.read(addr)
        } else if QUEUE_REGS_RANGE.contains(&offset) {
            let offset = offset - QUEUE_REGS_RANGE.start;
            let idx = offset / QUEUE_REG_SIZE;
            if idx < self.chs.len() {
                self.chs[idx].read((offset % QUEUE_REG_SIZE) as u64)
            } else {
                None
            }
        } else if GLOBAL_REGS_RANGE.contains(&offset) {
            let offset = offset - GLOBAL_REGS_RANGE.start;
            self.global.read(offset as u64)
        } else {
            None
        }
    }
}

impl<const CHS: usize, const ET_FILTER_N: usize, const TP5_FILTER_N: usize> GenHeader
    for TopRegs<CHS, ET_FILTER_N, TP5_FILTER_N>
{
    fn render_name() -> &'static str {
        "TopRegs"
    }
    fn gen_c_header<W: std::io::Write>(header: &mut W) -> std::io::Result<()> {
        writeln!(header, "#define RX_REGS_OFFSET {:#x}", RX_REGS_RANGE.start)?;
        writeln!(header, "#define TX_REGS_OFFSET {:#x}", TX_REGS_RANGE.start)?;
        writeln!(header, "#define QUEUE_NUM {}", CHS)?;
        writeln!(
            header,
            "#define QUEUE_REGS_OFFSET {:#x}",
            QUEUE_REGS_RANGE.start
        )?;
        writeln!(
            header,
            "#define GLOBAL_REGS_OFFSET {:#x}",
            GLOBAL_REGS_RANGE.start
        )?;
        global::EthaGlobalRegs::gen_c_header(header)?;
        global::EthaEn::gen_c_header(header)?;

        ChRegs::gen_c_header(header)?;

        rx::RxRegs::<ET_FILTER_N, TP5_FILTER_N>::gen_c_header(header)?;

        writeln!(
            header,
            "#define TP5_FILTER(name, i) TP5_FILTER_OFFSET(RX_REGS_OFFSET, name, i)",
        )?;
        writeln!(
            header,
            "#define ET_FILTER(i) ET_FILTER_OFFSET(RX_REGS_OFFSET, i)",
        )?;
        writeln!(header, "#define DEFAULT_Q DEFAULT_Q_OFFSET(RX_REGS_OFFSET)",)?;
        writeln!(
            header,
            "#define RX_RING(name, i) RX_RING_OFFSET(QUEUE_REGS_OFFSET, name, i)",
        )?;
        writeln!(
            header,
            "#define TX_RING(name, i) TX_RING_OFFSET(QUEUE_REGS_OFFSET, name, i)",
        )?;
        writeln!(
            header,
            "#define RX_EN (GLOBAL_REGS_OFFSET + ETHA_GLOBAL_REGS_RX_EN_OFFSET)",
        )?;
        writeln!(
            header,
            "#define TX_EN (GLOBAL_REGS_OFFSET + ETHA_GLOBAL_REGS_TX_EN_OFFSET)",
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use etha_model_generator::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use terminus_spaceport::memory::prelude::*;
    use terminus_spaceport::memory::region::Region;
    use terminus_spaceport::space::Space;
    csr_map! {
        Regs(0x0, 0x2) {
            ring_base(RW): RingBase, 0x1;
            ring_base1(RO): RingBase, 0x2;
        }
    }
    define_csr! {
    RingBase {
        fields {
            queue_id(RW): 4, 0;
            congestion_drop(RW):8, 5;
            en(RW): 31, 31;
        }
    }
    }

    #[derive_io(U32)]
    struct RegMod(Arc<LockedRegs>);
    impl U32Access for RegMod {
        fn write(&self, addr: &u64, data: u32) {
            self.0.write((*addr) >> 2, data as u64).unwrap()
        }

        fn read(&self, addr: &u64) -> u32 {
            self.0.read((*addr) >> 2).unwrap() as u32
        }
    }

    struct TestModule {
        bus: Mutex<Space>,
        reg: Arc<LockedRegs>,
    }
    unsafe impl Send for TestModule {}
    unsafe impl Sync for TestModule {}
    impl TestModule {
        fn new() -> Self {
            let bus = Mutex::new(Space::new());
            let reg = Arc::new(LockedRegs::new(32));
            bus.lock()
                .unwrap()
                .add_region(
                    "test",
                    &Region::remap(
                        0x1000,
                        &Region::io(0, 0x1000, Box::new(RegMod(reg.clone()))),
                    ),
                )
                .unwrap();
            TestModule { bus, reg }
        }
        fn write(&self, addr: u64, data: u32) {
            self.bus.lock().unwrap().write_u32(&addr, data).unwrap()
        }
        fn read(&self, addr: u64) -> u32 {
            self.bus.lock().unwrap().read_u32(&addr).unwrap()
        }
    }

    #[test]
    fn reg_access_test() {
        let m = Arc::new(TestModule::new());
        let m1 = m.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_micros(3));
                let data = m1.read(0x1004);
                m1.reg.ring_base1_mut().set(m1.reg.ring_base().get());
                println!("fd data = {:#x}", data);
                if data == 0x10 {
                    break;
                }
            }
        });
        for i in 1..0x11 {
            thread::sleep(Duration::from_millis(2));
            m.write(0x1004, i);
            loop {
                m.write(0x1008, 0);
                let data = m.read(0x1008);
                println!("bd data = {:#x}", data);
                if data == i {
                    break;
                }
            }
        }
    }
}
