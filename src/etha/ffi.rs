use super::STATICS_TAR;
use super::{CHS, Etha, RX_ET_FILTERS, RX_TP5_FILTERS, reg_if::TopRegs};
use crate::aborter::*;
use crate::arbiter::*;
use crate::irq::*;
use crate::logger;
use crate::mac::{PcapMac, pcap_cmp};
use crate::reg_if::RegBus;
use smoltcp::phy::*;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use std::thread;

struct CHandle {
    abort: Arc<Aborter>,
    regs: Arc<TopRegs<CHS, RX_ET_FILTERS, RX_TP5_FILTERS>>,
    irqs: Arc<Mutex<IrqVec>>,
    model_thread: thread::JoinHandle<()>,
}
impl CHandle {
    fn new<M: for<'a> Device<'a> + Send + 'static>(etha: Etha<MyArbiter, M>, core_id: i32) -> Self {
        let core_id = if core_id < 0 {
            None
        } else {
            let r = core_affinity::get_core_ids()
                .map(|core_set| {
                    core_set
                        .into_iter()
                        .filter(|id| core_id as usize == id.id)
                        .next()
                })
                .flatten();
            if r.is_none() {
                tracing::warn!("Etha affinity to core {} fail!", core_id);
            }
            r
        };
        CHandle {
            abort: etha.abort(),
            regs: etha.regs(),
            irqs: etha.irqs(),
            model_thread: etha.spawn(core_id),
        }
    }
}

static C_HANDLE: RwLock<Option<CHandle>> = RwLock::new(None);

type MyArbiter = RRArbiter<CHS>;

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_simulate_pcap(
    rx_file: *const std::os::raw::c_char,
    tx_file: *const std::os::raw::c_char,
    core_id: i32,
) {
    unsafe {
        let rx_file = std::ffi::CStr::from_ptr(rx_file).to_str().unwrap();
        let tx_file = std::ffi::CStr::from_ptr(tx_file).to_str().unwrap();
        let etha = Etha::new(MyArbiter::new(), PcapMac::new(rx_file, tx_file));
        *C_HANDLE.write().unwrap() = Some(CHandle::new(etha, core_id));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_simulate_loopback(core_id: i32) {
    let etha = Etha::new(MyArbiter::new(), Loopback::new(Medium::Ethernet));
    *C_HANDLE.write().unwrap() = Some(CHandle::new(etha, core_id));
}

struct DeviceWrapper<D: for<'a> Device<'a>>(D);

impl<'a, D: for<'b> Device<'b>> Device<'a> for DeviceWrapper<D> {
    type RxToken = <D as Device<'a>>::RxToken;
    type TxToken = <D as Device<'a>>::TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        self.0.capabilities()
    }

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        self.0.receive()
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        self.0.transmit()
    }
}

unsafe impl<D: for<'a> Device<'a>> Send for DeviceWrapper<D> {}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_simulate_raw_socket(
    socket_file: *const std::os::raw::c_char,
    core_id: i32,
) {
    let socket_file = unsafe { std::ffi::CStr::from_ptr(socket_file).to_str().unwrap() };
    let etha = Etha::new(
        MyArbiter::new(),
        DeviceWrapper(
            RawSocket::new(socket_file, Medium::Ethernet)
                .expect(&format!("socket file {} open failed!", socket_file)),
        ),
    );
    *C_HANDLE.write().unwrap() = Some(CHandle::new(etha, core_id));
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_simulate_tap(tap_file: *const std::os::raw::c_char, core_id: i32) {
    let tap_file = unsafe { std::ffi::CStr::from_ptr(tap_file).to_str().unwrap() };
    let etha = Etha::new(
        MyArbiter::new(),
        DeviceWrapper(
            TunTapInterface::new(tap_file, Medium::Ethernet)
                .expect(&format!("tap file {} open failed!", tap_file)),
        ),
    );
    *C_HANDLE.write().unwrap() = Some(CHandle::new(etha, core_id));
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_abort() {
    let h = C_HANDLE
        .write()
        .unwrap()
        .take()
        .expect("etha model does not exist!");
    h.abort.abort();
    h.model_thread.join().expect("etha model exit with err!");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_register_irq_handler(id: u32, f: extern "C" fn(u32)) {
    let h = C_HANDLE.read().unwrap();
    h.as_ref()
        .expect("etha model does not exist!")
        .irqs
        .lock()
        .unwrap()
        .bind(id as usize, move |id| f(id as u32))
        .expect(&format!(
            "etha_register_irq_handler: invalid irq id {}!",
            id
        ));
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_reg_write(addr: u32, value: u32) {
    let h = C_HANDLE.read().unwrap();
    h.as_ref()
        .expect("etha model does not exist!")
        .regs
        .write(addr as u64, value as u64)
        .expect(format!("etha_reg_write @{:#x} error!", addr).as_str());
    tracing::event!(
        target: STATICS_TAR,
        logger::STATICS_REG_LEVEL,
        name = "reg write",
        addr = addr,
        data = value
    );
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_reg_read(addr: u32) -> u32 {
    let h = C_HANDLE.read().unwrap();
    let data = h
        .as_ref()
        .expect("etha model does not exist!")
        .regs
        .read(addr as u64)
        .expect(format!("etha_reg_read @{:#x} error!", addr).as_str()) as u32;
    tracing::event!(
        target: STATICS_TAR,
        logger::STATICS_REG_LEVEL,
        name = "reg read",
        addr = addr,
        data = data
    );
    data
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_pcap_cmp(
    lhs: *const std::os::raw::c_char,
    rhs: *const std::os::raw::c_char,
    verbose: bool,
) -> bool {
    unsafe {
        let lhs = std::ffi::CStr::from_ptr(lhs).to_str().unwrap();
        let rhs = std::ffi::CStr::from_ptr(rhs).to_str().unwrap();
        pcap_cmp(lhs, rhs, verbose).is_err()
    }
}
