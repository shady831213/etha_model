use super::etha;
use super::etha_ipsec;
use super::logger;
use std::sync::Mutex;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{filter, util::SubscriberInitExt};
static LOGGER: Mutex<Option<Option<tracing_chrome::FlushGuard>>> = Mutex::new(None);

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_logger_en(statics_lvl: u32) {
    let mut logger_guard = LOGGER.lock().unwrap();
    if logger_guard.is_some() {
        return;
    }
    let logger = tracing_subscriber::registry().with(logger::default());
    if statics_lvl & 0x3 != 0 {
        let filter = filter::filter_fn(move |metadata| {
            (metadata.level() == &logger::STATICS_LEVEL && (statics_lvl & 0x2 != 0)
                || metadata.level() == &logger::STATICS_REG_LEVEL && (statics_lvl & 0x1 != 0))
                && (metadata.target() == etha::STATICS_TAR
                    || metadata.target() == etha_ipsec::STATICS_TAR)
        });
        let (statics, guard) = logger::statics("model");
        logger.with(statics.with_filter(filter)).init();
        *logger_guard = Some(Some(guard));
    } else {
        logger.init();
        *logger_guard = Some(None)
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn etha_logger_dis() {
    LOGGER.lock().unwrap().take();
}
