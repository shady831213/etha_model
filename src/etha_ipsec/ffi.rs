use super::STATICS_TAR;
use super::{EthaIpsec, IPSEC_CH_NUM, IPSEC_SESSION_NUM, reg_if::TopRegs};
use crate::aborter::*;
use crate::arbiter::*;
use crate::irq::*;
use crate::logger;
use crate::reg_if::RegBus;
use std::sync::RwLock;

use std::sync::{Arc, Mutex};
use std::thread;

struct CHandle {
    abort: Arc<Aborter>,
    regs: Arc<TopRegs<IPSEC_CH_NUM, IPSEC_SESSION_NUM>>,
    irqs: Arc<Mutex<IrqVec>>,
    model_thread: thread::JoinHandle<()>,
}

impl CHandle {
    fn new(ipsec: EthaIpsec<MyArbiter>, core_id: i32) -> Self {
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
                tracing::warn!("Etha_ipsec affinity to core {} fail!", core_id);
            }
            r
        };
        CHandle {
            abort: ipsec.abort(),
            regs: ipsec.regs(),
            irqs: ipsec.irqs(),
            model_thread: ipsec.spawn(core_id),
        }
    }
}

static C_HANDLE: RwLock<Option<CHandle>> = RwLock::new(None);

type MyArbiter = RRArbiter<IPSEC_CH_NUM>;

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_ipsec_simulate(core_id: i32) {
    let etha_ipsec = EthaIpsec::new(MyArbiter::new());
    *C_HANDLE.write().unwrap() = Some(CHandle::new(etha_ipsec, core_id));
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_ipsec_abort() {
    let h = C_HANDLE
        .write()
        .unwrap()
        .take()
        .expect("etha_ipsec model does not exist!");
    h.abort.abort();
    h.model_thread
        .join()
        .expect("etha_ipsec model exit with err!");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_ipsec_register_irq_handler(id: u32, f: extern "C" fn(u32)) {
    let h = C_HANDLE.read().unwrap();
    h.as_ref()
        .expect("etha_ipsec model does not exist!")
        .irqs
        .lock()
        .unwrap()
        .bind(id as usize, move |id| f(id as u32))
        .expect(&format!(
            "etha_ipsec_register_irq_handler: invalid irq id {}!",
            id
        ));
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_ipsec_reg_write(addr: u32, value: u32) {
    let h = C_HANDLE.read().unwrap();
    h.as_ref()
        .expect("etha_ipsec model does not exist!")
        .regs
        .write(addr as u64, value as u64)
        .expect(format!("etha_ipsec_reg_write @{:#x} error!", addr).as_str());
    tracing::event!(
        target: STATICS_TAR,
        logger::STATICS_REG_LEVEL,
        name = "reg write",
        addr = addr,
        data = value
    );
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_ipsec_reg_read(addr: u32) -> u32 {
    let h = C_HANDLE.read().unwrap();
    let data = h
        .as_ref()
        .expect("etha_ipsec model does not exist!")
        .regs
        .read(addr as u64)
        .expect(format!("etha_ipsec_reg_read @{:#x} error!", addr).as_str()) as u32;
    tracing::event!(
        target: STATICS_TAR,
        logger::STATICS_REG_LEVEL,
        name = "reg read",
        addr = addr,
        data = data
    );
    data
}
