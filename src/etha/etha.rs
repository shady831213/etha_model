use super::etha_core::EthaCore;
use super::reg_if::TopRegs;
use super::*;
use crate::aborter::*;
use crate::arbiter::*;
use crate::irq::*;
use core_affinity::{set_for_current, CoreId};
use smoltcp::phy::Device;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Etha<A: Arbiter, M: for<'a> Device<'a>> {
    core: EthaCore<A, M>,
    regs: Arc<TopRegs<CHS, RX_ET_FILTERS, RX_TP5_FILTERS>>,
}

impl<A: Arbiter, M: for<'a> Device<'a>> Etha<A, M> {
    pub fn new(arbiter: A, mac: M) -> Self {
        let regs = Arc::new(TopRegs::new());
        Etha {
            core: EthaCore::new(arbiter, mac, &regs),
            regs,
        }
    }

    pub fn abort(&self) -> Arc<Aborter> {
        self.core.abort()
    }
    pub fn regs(&self) -> Arc<TopRegs<CHS, RX_ET_FILTERS, RX_TP5_FILTERS>> {
        self.regs.clone()
    }
    pub fn irqs(&self) -> Arc<Mutex<IrqVec>> {
        self.core.irqs()
    }
}

impl<A: Arbiter + Send + 'static, M: for<'a> Device<'a> + Send + 'static> Etha<A, M> {
    pub fn spawn(mut self, core_id: Option<CoreId>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            if let Some(id) = core_id {
                set_for_current(id);
            }
            self.core.run();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::tests_driver_helper::*;
    use super::*;
    use crate::mac::pcap_cmp;
    use std::time::Duration;

    extern "C" fn ch_handler(id: usize) {
        println!("get irq[{}]!", id);
    }
    #[test]
    fn abort_test() {
        let etha = Etha::new(
            RRArbiter::<CHS>::new(),
            crate::mac::PcapMac::new("pcaps/20_ecpri_pkts.pcap", "pcaps/tmp/abort_test.pcap"),
        );
        let abort = etha.abort();
        let t = etha.spawn(Some(CoreId { id: 0 }));
        thread::sleep(Duration::from_millis(20));
        abort.abort();
        t.join().unwrap();
    }
    #[test_log::test]
    fn pcap_simple_test() {
        let etha = Etha::new(
            RRArbiter::<CHS>::new(),
            crate::mac::PcapMac::new(
                "pcaps/20_ecpri_pkts.pcap",
                "pcaps/tmp/pcap_simple_test.pcap",
            ),
        );
        let abort = etha.abort();
        let reg = etha.regs();
        let irqs = etha.irqs();
        let t = etha.spawn(Some(CoreId { id: 0 }));

        let driver = SwEtha::new(&reg);
        let mut default_ch = driver.alloc_default_ch(2, 1060, 2, CongestionAction::Blocking);
        //enable full irq
        default_ch
            .tx
            .regs
            .write(
                addr(default_ch.tx.base + RING_INTM_OFFSET),
                RING_FULL_FLAG as u64,
            )
            .unwrap();
        irqs.lock()
            .unwrap()
            .bind(CHS + 0, |id| ch_handler(id))
            .unwrap();
        let mut ecpri_ch = driver.alloc_et_ch(2, 1060, 2, CongestionAction::Blocking, 0xaefe);
        driver.rx_en();
        driver.tx_en();
        let mut cnt = 0;
        loop {
            if let Some(r) = ecpri_ch.rx.receive().take() {
                let n_blocks = r.len();
                let data = r.concat();
                let frame = smoltcp::wire::EthernetFrame::new_checked(&data).unwrap();
                println!(
                    "{}, length = {}, n_blocks = {}",
                    smoltcp::wire::PrettyPrinter::<smoltcp::wire::EthernetFrame<&[u8]>>::new(
                        "", &frame,
                    ),
                    data.len(),
                    n_blocks,
                );
                cnt += 1;
                if cnt < 20 {
                    default_ch.tx.send(&r, false);
                } else {
                    ecpri_ch.tx.send(&r, true).unwrap();
                }
                ecpri_ch.rx.release(n_blocks);
            }
            if cnt == 20 {
                break;
            }
        }

        abort.abort();
        t.join().unwrap();
        check_sample_file(
            "pcaps/20_ecpri_pkts.pcap",
            "pcaps/tmp/pcap_simple_test.pcap",
        );
    }

    #[test_log::test]
    fn pcap_default_queue_test() {
        let etha = Etha::new(
            RRArbiter::<CHS>::new(),
            crate::mac::PcapMac::new(
                "pcaps/icmp_12_pkts.pcap",
                "pcaps/tmp/pcap_default_queue_test.pcap",
            ),
        );
        let abort = etha.abort();
        let reg = etha.regs();
        let t = etha.spawn(Some(CoreId { id: 0 }));

        let driver = SwEtha::new(&reg);
        let mut default_ch = driver.alloc_default_ch(12, 1060, 2, CongestionAction::Blocking);
        let mut ecpri_ch = driver.alloc_et_ch(12, 1060, 2, CongestionAction::Blocking, 0xaefe);
        driver.rx_en();
        driver.tx_en();
        let mut cnt = 0;
        loop {
            if let Some(r) = default_ch.rx.receive().take() {
                let n_blocks = r.len();
                let data = r.concat();
                let frame = smoltcp::wire::EthernetFrame::new_checked(&data).unwrap();
                println!(
                    "{}, length = {}, n_blocks = {}",
                    smoltcp::wire::PrettyPrinter::<smoltcp::wire::EthernetFrame<&[u8]>>::new(
                        "", &frame,
                    ),
                    data.len(),
                    n_blocks,
                );
                cnt += 1;
                if cnt < 12 {
                    default_ch.tx.send(&r, false);
                } else {
                    ecpri_ch.tx.send(&r, true).unwrap();
                }
                default_ch.rx.release(n_blocks);
            }
            if cnt == 12 {
                break;
            }
        }

        abort.abort();
        t.join().unwrap();
        check_sample_file(
            "pcaps/icmp_12_pkts.pcap",
            "pcaps/tmp/pcap_default_queue_test.pcap",
        );
    }

    #[test_log::test]
    fn loopback_mac_test() {
        let frame_send: [u8; 1057] = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x11, 0x22, 0x33, 0x44, 0x66, 0xae, 0xfe, 0x10, 0x0,
            0x4, 0xf, 0x0, 0x1, 0x13, 0x80, 0x90, 0x0, 0x0, 0x3, 0x0, 0x30, 0xe8, 0x29, 0x81, 0x0,
            0x8, 0xf1, 0xb6, 0xca, 0xde, 0x19, 0x5, 0xca, 0x19, 0xc0, 0xca, 0x22, 0xca, 0x36, 0x19,
            0xca, 0xe7, 0xc0, 0x19, 0x4a, 0xca, 0xf, 0xe7, 0xe7, 0xfb, 0x8, 0xe7, 0xf, 0xd4, 0x36,
            0xf1, 0x4a, 0x36, 0x22, 0xe7, 0x19, 0x19, 0xca, 0xf, 0xb6, 0x2c, 0xc0, 0xf1, 0xb6,
            0x22, 0xe7, 0x5, 0x40, 0x4a, 0xfb, 0x8, 0xfb, 0xc0, 0xfb, 0xde, 0x36, 0xca, 0x36, 0xde,
            0x19, 0xde, 0x2c, 0xf1, 0xe7, 0x36, 0xb6, 0xe7, 0x4a, 0x19, 0xd4, 0xe7, 0xe7, 0xfb,
            0xd4, 0x2c, 0x8, 0xf1, 0x36, 0x5, 0xca, 0xb6, 0xe7, 0x36, 0x22, 0xc0, 0xe7, 0x4a, 0xfb,
            0xb6, 0xf1, 0x5, 0xe7, 0xca, 0xd4, 0xc0, 0x19, 0x19, 0x36, 0x5, 0x36, 0x8, 0x22, 0xfb,
            0xb6, 0x40, 0xc0, 0x4a, 0xd4, 0x4a, 0xf1, 0x22, 0xc0, 0x22, 0xd4, 0xfb, 0xde, 0x5,
            0xfb, 0xe7, 0x40, 0xfb, 0xe7, 0xe7, 0x36, 0x40, 0x8, 0xe7, 0xca, 0x5, 0x5, 0xde, 0xc0,
            0x36, 0xb6, 0xf, 0xfb, 0x2c, 0xca, 0xde, 0x36, 0x4a, 0x36, 0x4a, 0xf1, 0x5, 0x5, 0x5,
            0x40, 0x4a, 0x2c, 0x8, 0xe7, 0xd4, 0x2c, 0x2c, 0xca, 0xc0, 0xf1, 0xfb, 0x40, 0x5, 0x5,
            0xe7, 0x4a, 0xca, 0xd4, 0xe7, 0xb6, 0xc0, 0xca, 0x19, 0xe7, 0x19, 0xe7, 0x5, 0x8, 0x2c,
            0xca, 0xde, 0x2c, 0x4a, 0x4a, 0xe7, 0xfb, 0xd4, 0x36, 0xfb, 0xde, 0x36, 0x5, 0xc0,
            0x4a, 0x40, 0xb6, 0xf1, 0xde, 0xc0, 0x2c, 0xde, 0xe7, 0x8, 0xb6, 0x22, 0xd4, 0xfb, 0xf,
            0xe7, 0x4a, 0x5, 0x36, 0xca, 0x5, 0xd4, 0xfb, 0x22, 0xc0, 0xde, 0x2c, 0xf, 0xf, 0xca,
            0x19, 0xfb, 0xf, 0x22, 0x8, 0x2c, 0x4a, 0x22, 0x22, 0xca, 0xb6, 0x4a, 0x36, 0xf, 0x22,
            0x40, 0xf, 0x2c, 0x19, 0xca, 0x4a, 0x4a, 0xc0, 0xd4, 0x36, 0xf, 0x2c, 0xb6, 0x36, 0x8,
            0x4a, 0x40, 0x5, 0xde, 0x36, 0xca, 0x19, 0x19, 0x36, 0xc0, 0xf, 0xca, 0xca, 0xca, 0x19,
            0xb6, 0x22, 0x36, 0xe7, 0x2c, 0x22, 0xf, 0xfb, 0x36, 0x8, 0xe7, 0xde, 0xe7, 0xc0, 0x22,
            0x2c, 0xe7, 0xe7, 0x2c, 0xb6, 0x19, 0xf, 0xf1, 0xb6, 0xe7, 0xe7, 0xc0, 0xc0, 0xde, 0x5,
            0x36, 0xfb, 0xca, 0xfb, 0x8, 0xf1, 0x5, 0x2c, 0x2c, 0x2c, 0x22, 0xfb, 0x2c, 0x4a, 0xfb,
            0x36, 0x36, 0x36, 0x5, 0xe7, 0x2c, 0x19, 0xf, 0xde, 0xca, 0x19, 0x5, 0xc0, 0x36, 0x8,
            0x4a, 0x22, 0xf1, 0x5, 0x19, 0x5, 0xf, 0xf, 0xf1, 0xfb, 0xfb, 0xde, 0xf1, 0x5, 0xb6,
            0xe7, 0xfb, 0xf1, 0x19, 0xb6, 0xfb, 0xde, 0x2c, 0x19, 0x8, 0xca, 0x19, 0x22, 0xb6,
            0xc0, 0xb6, 0xd4, 0x4a, 0xc0, 0xf, 0x36, 0xfb, 0x36, 0xf1, 0xfb, 0xca, 0xc0, 0xf1,
            0x4a, 0x19, 0xf1, 0xf1, 0x5, 0x40, 0x8, 0x4a, 0x5, 0xf, 0x4a, 0x36, 0x5, 0x4a, 0x36,
            0x40, 0xf1, 0x2c, 0x19, 0xc0, 0x40, 0x19, 0x2c, 0xd4, 0x19, 0xe7, 0xca, 0xc0, 0x5, 0x5,
            0x5, 0x8, 0xd4, 0xc0, 0x36, 0x2c, 0x19, 0xca, 0xe7, 0x22, 0xc0, 0x5, 0xca, 0xe7, 0x4a,
            0xb6, 0xfb, 0x5, 0xd4, 0xc0, 0xb6, 0xde, 0xf, 0xe7, 0xe7, 0xb6, 0x8, 0x36, 0x40, 0xc0,
            0x2c, 0x5, 0xe7, 0xfb, 0x19, 0xfb, 0xb6, 0xca, 0xd4, 0xd4, 0xde, 0xd4, 0x36, 0xf, 0x19,
            0x5, 0x40, 0x22, 0xc0, 0x4a, 0xc0, 0x8, 0xb6, 0xf, 0xf1, 0xc0, 0x36, 0xca, 0x19, 0xf,
            0xfb, 0xb6, 0xc0, 0xe7, 0x19, 0x5, 0x40, 0xde, 0x40, 0x22, 0xca, 0x2c, 0x36, 0xb6, 0x5,
            0x40, 0x8, 0x22, 0x40, 0xc0, 0xca, 0xd4, 0x36, 0x5, 0xb6, 0x4a, 0xde, 0xca, 0x2c, 0x5,
            0x36, 0xfb, 0x19, 0x5, 0xf, 0x5, 0x40, 0x40, 0x22, 0xb6, 0x36, 0x8, 0xe7, 0x5, 0xfb,
            0xd4, 0xf1, 0xca, 0xd4, 0xf, 0x19, 0xb6, 0xfb, 0x4a, 0xf, 0x5, 0x4a, 0x19, 0xf, 0xc0,
            0x36, 0xd4, 0x22, 0x4a, 0x4a, 0x5, 0x8, 0x22, 0xc0, 0x2c, 0xfb, 0x5, 0x40, 0x2c, 0x40,
            0x40, 0xe7, 0x19, 0x4a, 0xb6, 0x22, 0x19, 0xf1, 0x40, 0xde, 0xb6, 0x4a, 0x5, 0xd4,
            0xc0, 0xf1, 0x8, 0x5, 0x4a, 0x22, 0xde, 0x5, 0xe7, 0xde, 0xf, 0xe7, 0x5, 0x4a, 0x5,
            0xe7, 0xde, 0xb6, 0xfb, 0xc0, 0x2c, 0x2c, 0xd4, 0xca, 0x19, 0x22, 0xf, 0x8, 0x4a, 0xca,
            0x22, 0x2c, 0x22, 0xf, 0x19, 0xca, 0xf1, 0xc0, 0xc0, 0xd4, 0xc0, 0xb6, 0x36, 0x4a,
            0x22, 0x2c, 0xf, 0xfb, 0xde, 0x5, 0x40, 0xd4, 0x8, 0x5, 0xf, 0x19, 0xe7, 0x22, 0xde,
            0xfb, 0xde, 0xd4, 0xf, 0xca, 0xc0, 0xc0, 0x40, 0xca, 0xb6, 0x19, 0xfb, 0x4a, 0xb6,
            0x22, 0x4a, 0xfb, 0x4a, 0x8, 0xf, 0x22, 0xe7, 0xb6, 0xe7, 0x2c, 0xf1, 0xf, 0xb6, 0x40,
            0x5, 0x40, 0xfb, 0xf, 0xca, 0xf, 0x22, 0xd4, 0xde, 0xf, 0x36, 0x19, 0xf, 0xd4, 0x8,
            0xd4, 0x4a, 0x4a, 0x19, 0xc0, 0x40, 0x4a, 0x4a, 0x4a, 0x19, 0xd4, 0xfb, 0xf, 0x40,
            0xca, 0x22, 0xc0, 0xfb, 0x22, 0xc0, 0xf1, 0x2c, 0xe7, 0x5, 0x8, 0xca, 0x22, 0x2c, 0xf,
            0x5, 0xc0, 0xd4, 0xf, 0xfb, 0xf, 0xf, 0xb6, 0x5, 0xb6, 0x19, 0x22, 0x22, 0xf1, 0xca,
            0x2c, 0x4a, 0xf, 0xfb, 0x19, 0x8, 0x4a, 0xf, 0xb6, 0xc0, 0x40, 0x19, 0xf1, 0xca, 0xb6,
            0xe7, 0xd4, 0xd4, 0x5, 0x40, 0xde, 0x40, 0xca, 0xe7, 0xf, 0x36, 0xfb, 0x19, 0xf, 0xe7,
            0x8, 0x36, 0x22, 0x2c, 0xd4, 0xf1, 0xe7, 0xca, 0xd4, 0xc0, 0xb6, 0xb6, 0x22, 0xde, 0x5,
            0xb6, 0xde, 0x40, 0xb6, 0xc0, 0x40, 0xca, 0xde, 0x36, 0xc0, 0x8, 0x4a, 0x4a, 0xd4,
            0xca, 0xfb, 0x4a, 0xb6, 0xde, 0xca, 0xf1, 0x40, 0x19, 0xf, 0x36, 0xde, 0xf1, 0xde,
            0x40, 0x19, 0x40, 0x19, 0xf, 0x40, 0xe7, 0x8, 0xd4, 0x4a, 0xf, 0xe7, 0x19, 0x2c, 0xca,
            0x22, 0xb6, 0x2c, 0xf, 0xd4, 0x40, 0x5, 0xca, 0x5, 0xde, 0x22, 0xe7, 0x36, 0x4a, 0x5,
            0xc0, 0x5, 0x7, 0x94, 0x58, 0xa8, 0xe3, 0x1d, 0xf6, 0xf6, 0x58, 0x94, 0xcf, 0xf6, 0x6c,
            0xcf, 0xe3, 0x58, 0xbb, 0x7f, 0xbb, 0x94, 0xf6, 0x6c, 0x7f, 0x80, 0xbb, 0x8, 0xf1,
            0xca, 0xe7, 0x4a, 0xd4, 0xb6, 0xca, 0xd4, 0xf1, 0x36, 0xb6, 0x22, 0xe7, 0x36, 0x36,
            0xe7, 0x40, 0xd4, 0x2c, 0x5, 0x19, 0x2c, 0xca, 0x36, 0x8, 0xd4, 0xc0, 0xf, 0xca, 0x5,
            0xde, 0x4a, 0xc0, 0xde, 0x2c, 0x22, 0x2c, 0xb6, 0xb6, 0x19, 0x2c, 0xb6, 0xb6, 0x36,
            0x36, 0xe7, 0x36, 0xca, 0x4a, 0x8, 0x19, 0xe7, 0xfb, 0x22, 0x19, 0x36, 0xb6, 0x4a,
            0xfb, 0x5, 0xf1, 0xb6, 0xf1, 0xb6, 0x4a, 0xde, 0x19, 0xca, 0xc0, 0xe7, 0xe7, 0xf, 0xb6,
            0xd4, 0x8, 0xb6, 0xd4, 0xe7, 0xf1, 0x4a, 0x22, 0xde, 0xf1, 0xca, 0xfb, 0xca, 0x40, 0xf,
            0xca, 0xfb, 0x19, 0xf, 0xd4, 0xc0, 0xd4, 0x40, 0x5, 0xde, 0xde, 0x8, 0x36, 0xe7, 0xc0,
            0xde, 0xd4, 0xf1, 0xd4, 0x5, 0xf, 0xf, 0xca, 0x36, 0xf, 0x2c, 0x5, 0x4a, 0xb6, 0x2c,
            0xb6, 0x2c, 0xf1, 0x36, 0xc0, 0x5, 0x8, 0xb6, 0xd4, 0x19, 0xfb, 0xd4, 0xc0, 0x2c, 0xb6,
            0x40, 0x36, 0xca, 0xfb, 0xf1, 0xf1, 0x40, 0x36, 0xca, 0xd4, 0xfb, 0xd4, 0xd4, 0xca,
            0xf, 0x4a, 0x8, 0x2c, 0xd4, 0x4a, 0xd4, 0x4a, 0xca, 0xfb, 0x40, 0x22, 0xd4, 0xf, 0x19,
            0x36, 0x2c, 0x36, 0x40, 0x36, 0xf, 0xe7, 0xca, 0xc0, 0xe7, 0x5, 0x19, 0x8, 0xf1, 0x19,
            0xd4, 0xf1, 0x5, 0xf, 0x2c, 0xf1, 0xb6, 0xe7, 0xc0, 0x22, 0xf1, 0xf, 0xe7, 0x40, 0x40,
            0xfb, 0x22, 0xb6, 0xca, 0x19, 0x22, 0x2c,
        ];
        let etha = Etha::new(
            RRArbiter::<CHS>::new(),
            smoltcp::phy::Loopback::new(smoltcp::phy::Medium::Ethernet),
        );
        let abort = etha.abort();
        let reg = etha.regs();
        let t = etha.spawn(Some(CoreId { id: 0 }));

        let driver = SwEtha::new(&reg);
        let _default_ch = driver.alloc_default_ch(1, 1024, 1, CongestionAction::Blocking);
        let mut ecpri_ch = driver.alloc_et_ch(1, 2048, 1, CongestionAction::Blocking, 0xaefe);
        driver.rx_en();
        driver.tx_en();

        ecpri_ch.tx.send(&[&frame_send], false);
        loop {
            if let Some(r) = ecpri_ch.rx.receive().take() {
                let data = r.concat();
                let frame = smoltcp::wire::EthernetFrame::new_checked(&data).unwrap();
                println!(
                    "{}",
                    smoltcp::wire::PrettyPrinter::<smoltcp::wire::EthernetFrame<&[u8]>>::new(
                        "", &frame
                    )
                );
                assert_eq!(data.as_slice(), frame_send);
                break;
            }
        }
        abort.abort();
        t.join().unwrap();
    }

    fn check_sample_file(expect_file: &str, output_file: &str) {
        pcap_cmp(expect_file, output_file, true).unwrap()
    }
}

#[cfg(test)]
mod tests_driver_helper {
    use super::*;
    pub(super) use crate::desc::*;
    pub(super) use crate::etha::desc::buffer::*;
    pub(super) use crate::etha::desc::rx::*;
    pub(super) use crate::etha::desc::tx::*;
    pub(super) use crate::etha::reg_if::global::*;
    pub(super) use crate::etha::reg_if::rx::*;
    pub(super) use crate::etha::reg_if::*;
    pub(super) use crate::reg_if::{
        ring::{sw_ring::*, *},
        RegBus,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub(super) const fn addr(offset: usize) -> u64 {
        offset as u64
    }
    type RegT = TopRegs<CHS, RX_ET_FILTERS, RX_TP5_FILTERS>;

    pub(super) struct SwRxQueue {
        id: usize,
        mem_size: usize,
        buffers: Vec<Vec<u8>>,
        resp_ptr: usize,
        ring: SwQueue<RegT, u64, [DescEntryT; RX_DESC_ENTRY_SIZE / DESC_ENTRY_SIZE]>,
    }

    impl SwRxQueue {
        pub(super) fn new(regs: &Arc<RegT>, id: usize, ring_size: usize, mem_size: usize) -> Self {
            SwRxQueue {
                id,
                mem_size,
                buffers: vec![vec![0; mem_size]; ring_size],
                resp_ptr: 0,
                ring: SwQueue::new(
                    regs,
                    rx_ring_base(id),
                    ring_size,
                    vec![0; ring_size],
                    vec![[0; RX_DESC_ENTRY_SIZE / DESC_ENTRY_SIZE]; ring_size],
                ),
            }
        }
        pub(super) fn init(&self) {
            self.regs
                .write(
                    addr(rx_ring_base(self.id) + RING_MEM_SIZE_OFFSET),
                    self.mem_size as u64,
                )
                .unwrap();
            self.ring.init();
            let buffer_addrs = self
                .buffers
                .iter()
                .map(|b| b.as_ptr() as u64)
                .collect::<Vec<_>>();
            self.r_push_req(&buffer_addrs).unwrap();
        }
        pub(super) fn init_check(&self) {
            println!(
                "SwRxQueue[{}]: consumer_valids: {}",
                self.id,
                self.r_c_valids()
            );
            assert_eq!(self.r_c_valids(), self.ring_size);
            println!("SwRxQueue[{}]: full: {}", self.id, self.r_full());
            assert_eq!(self.r_full(), true);
        }
        pub(super) fn receive<'a>(&'a mut self) -> Option<Vec<&'a [u8]>> {
            if self.r_p_valids() > 0 {
                let head = unsafe { *self.r_get_resp_at(self.resp_ptr).unwrap() };
                assert!(
                    head.frame.start() == 1,
                    "rx[{}] expect start frame @ ptr {:#x}",
                    self.id,
                    self.resp_ptr
                );
                let mut buffer = vec![];
                for i in 0..=head.frame.n_blocks() as usize {
                    let ptr = self.r_incr_ptr(self.resp_ptr, i);
                    let idx = <SwQueue<
                        RegT,
                        u64,
                        [DescEntryT; RX_DESC_ENTRY_SIZE / DESC_ENTRY_SIZE],
                    > as Ring>::r_ptr_l(ptr);
                    let resp = unsafe { *self.r_get_resp_at(ptr).unwrap() };
                    assert_eq!(
                        resp.frame.full_addr(),
                        self.buffers[idx].as_ptr() as u64,
                        "rx[{}] addr error @ptr {:#x} expect = {:#x} @buffers[{}], actual = {:#x}",
                        self.id,
                        ptr,
                        idx,
                        self.buffers[idx].as_ptr() as u64,
                        resp.frame.full_addr()
                    );
                    buffer.push(&self.buffers[idx][..resp.frame.size() as usize])
                }
                self.resp_ptr = self.r_incr_ptr(self.resp_ptr, head.frame.n_blocks() as usize + 1);
                Some(buffer)
            } else {
                None
            }
        }

        pub(super) fn release(&self, n: usize) {
            self.r_advance_p_n(n);
        }
    }

    impl SwRing for SwRxQueue {
        type REQ = u64;
        type RESP = RxResultDesc;
        type R = SwQueue<RegT, u64, [DescEntryT; RX_DESC_ENTRY_SIZE / DESC_ENTRY_SIZE]>;
        const RESP_SIZE: usize = RX_DESC_ENTRY_SIZE;
        fn get_ring(&self) -> &Self::R {
            &self.ring
        }
    }

    impl std::ops::Deref for SwRxQueue {
        type Target = SwQueue<RegT, u64, [DescEntryT; RX_DESC_ENTRY_SIZE / DESC_ENTRY_SIZE]>;
        fn deref(&self) -> &Self::Target {
            self.get_ring()
        }
    }

    pub(super) struct SwTxQueue {
        id: usize,
        resp_ptr: usize,
        ring: SwQueue<
            RegT,
            [DescEntryT; TX_REQ_ENTRY_SIZE / DESC_ENTRY_SIZE],
            [DescEntryT; TX_RESULT_ENTRY_SIZE / DESC_ENTRY_SIZE],
        >,
    }

    impl SwTxQueue {
        pub(super) fn new(regs: &Arc<RegT>, id: usize, ring_size: usize) -> Self {
            SwTxQueue {
                id,
                resp_ptr: 0,
                ring: SwQueue::new(
                    regs,
                    tx_ring_base(id),
                    ring_size,
                    vec![[0; TX_REQ_ENTRY_SIZE / DESC_ENTRY_SIZE]; ring_size],
                    vec![[0; TX_RESULT_ENTRY_SIZE / DESC_ENTRY_SIZE]; ring_size],
                ),
            }
        }
        pub(super) fn init(&self) {
            self.ring.init()
        }
        pub(super) fn init_check(&self) {
            println!(
                "SwTxQueue[{}]: producer_valids: {}",
                self.id,
                self.r_p_valids()
            );
            assert_eq!(self.r_p_valids(), self.ring_size);
            println!("SwTxQueue[{}]: empty: {}", self.id, self.r_empty());
            assert_eq!(self.r_empty(), true);
        }
        pub(super) fn send(&mut self, data: &[&[u8]], blocking: bool) -> Option<TxResultDesc> {
            loop {
                if self.r_p_valids() >= data.len() {
                    break;
                }
            }
            let mut reqs = data
                .iter()
                .map(|b| TxReqDesc {
                    frame: FrameDesc::from(MemBlock {
                        addr: b.as_ptr() as u64,
                        size: b.len(),
                    }),
                    ctrl: TxCtrlDesc::default(),
                })
                .collect::<Vec<_>>();
            let n_reqs = reqs.len();
            reqs[0].frame.set_n_blocks(n_reqs as u32 - 1);
            reqs[0]
                .frame
                .set_total_size(data.iter().map(|b| b.len()).reduce(|a, b| a + b).unwrap() as u32);
            reqs[0].frame.set_start(1);
            reqs[0].ctrl.set_resp_en(blocking as u32);
            reqs[n_reqs - 1].frame.set_end(1);
            self.r_push_req(&reqs).unwrap();
            let resp_ptr = self.resp_ptr;
            self.resp_ptr = self.r_p_ptr();
            if blocking {
                loop {
                    if self.r_empty() {
                        break;
                    }
                }
                Some(unsafe { *self.r_get_resp_at(resp_ptr).unwrap() })
            } else {
                None
            }
        }
    }

    impl SwRing for SwTxQueue {
        type REQ = TxReqDesc;
        type RESP = TxResultDesc;
        type R = SwQueue<
            RegT,
            [DescEntryT; TX_REQ_ENTRY_SIZE / DESC_ENTRY_SIZE],
            [DescEntryT; TX_RESULT_ENTRY_SIZE / DESC_ENTRY_SIZE],
        >;
        const REQ_SIZE: usize = TX_REQ_ENTRY_SIZE;
        const RESP_SIZE: usize = TX_RESULT_ENTRY_SIZE;

        fn get_ring(&self) -> &Self::R {
            &self.ring
        }
    }

    impl std::ops::Deref for SwTxQueue {
        type Target = SwQueue<
            RegT,
            [DescEntryT; TX_REQ_ENTRY_SIZE / DESC_ENTRY_SIZE],
            [DescEntryT; TX_RESULT_ENTRY_SIZE / DESC_ENTRY_SIZE],
        >;
        fn deref(&self) -> &Self::Target {
            self.get_ring()
        }
    }

    pub(super) struct SwCh {
        pub rx: SwRxQueue,
        pub tx: SwTxQueue,
    }
    impl SwCh {
        pub(super) fn new(rx: SwRxQueue, tx: SwTxQueue) -> Self {
            SwCh { rx, tx }
        }
        pub(super) fn init(&self) {
            self.rx.init();
            self.tx.init();
        }
        pub(super) fn init_check(&self) {
            self.rx.init_check();
            self.tx.init_check();
        }
    }

    pub(super) struct SwEtha {
        regs: Arc<RegT>,
        ch_id: AtomicUsize,
        et_filter_id: AtomicUsize,
        default_ch: AtomicUsize,
    }
    impl SwEtha {
        pub(super) fn new(regs: &Arc<RegT>) -> Self {
            SwEtha {
                regs: regs.clone(),
                ch_id: AtomicUsize::new(0),
                et_filter_id: AtomicUsize::new(0),
                default_ch: AtomicUsize::new(0),
            }
        }
        fn alloc_ch(&self) -> usize {
            let id = self.ch_id.fetch_add(1, Ordering::SeqCst);
            assert!(id < CHS);
            id
        }
        fn alloc_et_filter(&self) -> usize {
            let id = self.et_filter_id.fetch_add(1, Ordering::SeqCst);
            assert!(id < RX_ET_FILTERS);
            id
        }
        pub(super) fn alloc_default_ch(
            &self,
            rx_size: usize,
            rx_mem_size: usize,
            tx_size: usize,
            congest: CongestionAction,
        ) -> SwCh {
            assert!(self.default_ch.load(Ordering::SeqCst) == 0);
            let id = self.alloc_ch();
            let ch = SwCh::new(
                SwRxQueue::new(&self.regs, id, rx_size, rx_mem_size),
                SwTxQueue::new(&self.regs, id, tx_size),
            );
            self.regs
                .write(
                    addr(RX_REGS_RANGE.start + self.regs.rx.default_filter_offset()),
                    (set_filter_queue_id(id)
                        | set_filter_congestion_action(congest)
                        | set_filter_en(true)) as u64,
                )
                .unwrap();
            ch.init();
            ch.init_check();
            ch
        }
        pub(super) fn alloc_et_ch(
            &self,
            rx_size: usize,
            rx_mem_size: usize,
            tx_size: usize,
            congest: CongestionAction,
            et: u16,
        ) -> SwCh {
            let id = self.alloc_ch();
            let filter_id = self.alloc_et_filter();
            let ch = SwCh::new(
                SwRxQueue::new(&self.regs, id, rx_size, rx_mem_size),
                SwTxQueue::new(&self.regs, id, tx_size),
            );
            self.regs
                .write(
                    addr(RX_REGS_RANGE.start + self.regs.rx.et_filter_offset(filter_id)),
                    (set_filter_etype(et)
                        | set_filter_queue_id(id)
                        | set_filter_congestion_action(congest)
                        | set_filter_en(true)) as u64,
                )
                .unwrap();
            ch.init();
            ch.init_check();
            ch
        }
        pub(super) fn rx_en(&self) {
            self.regs
                .write(addr(GLOBAL_REGS_RANGE.start + GLOBAL_RX_EN_OFFSET), 0x1)
                .unwrap();
        }
        pub(super) fn tx_en(&self) {
            self.regs
                .write(addr(GLOBAL_REGS_RANGE.start + GLOBAL_TX_EN_OFFSET), 0x1)
                .unwrap();
        }
    }
}
