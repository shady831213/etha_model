use super::desc::req::*;
use super::desc::resp::*;
use super::STATICS_TAR;
use crate::logger;

use super::rohc_wrapper::*;
pub struct RohcEngine {
    comp_v1: EthaRohcComp,
    comp_v2: EthaRohcComp,
    decomp_v1: EthaRohcDeComp,
    decomp_v2: EthaRohcDeComp,
}

impl RohcEngine {
    pub fn new() -> RohcEngine {
        RohcEngine {
            comp_v1: EthaRohcComp::new(false).unwrap(),
            comp_v2: EthaRohcComp::new(true).unwrap(),
            decomp_v1: EthaRohcDeComp::new(false).unwrap(),
            decomp_v2: EthaRohcDeComp::new(true).unwrap(),
        }
    }
    pub fn process(&self, mut req: RohcReqDesc) -> RohcStatusDesc {
        let mut status = RohcStatusDesc::default();
        let mut src = vec![0u8; req.src.total_size() as usize];
        if req
            .src
            .read_with(
                &mut src,
                |b| {
                    tracing::event!(
                        target: STATICS_TAR,
                        logger::STATICS_LEVEL,
                        name = "src read data",
                        addr = b.addr,
                        size = b.size
                    );
                },
                |addr, size| {
                    if size > 1 {
                        tracing::event!(
                            target: STATICS_TAR,
                            logger::STATICS_LEVEL,
                            name = "src read sc-list",
                            addr = addr,
                            size = size * std::mem::size_of::<crate::desc::SCBufferEntry>(),
                        );
                    }
                },
            )
            .is_err()
        {
            status.set_src_err(1);
            return status;
        }
        tracing::debug!(target : "rohc-engine", "load src!");
        let mut dst = vec![0u8; req.dst.total_size() as usize];
        let decomp = req.cfg.decomp() != 0;
        let v2 = req.cfg.v2() != 0;
        let r = if decomp {
            if v2 { &self.decomp_v2 } else { &self.decomp_v1 }.decompress(&src, &mut dst)
        } else {
            if v2 { &self.comp_v2 } else { &self.comp_v1 }.compress(&src, &mut dst)
        };
        match r {
            Ok(len) => status.set_len(len as crate::desc::DescEntryT),
            Err(rohc_status_t::ROHC_STATUS_OUTPUT_TOO_SMALL) => status.set_too_small(1),
            Err(rohc_status_t::ROHC_STATUS_BAD_CRC) => status.set_bad_crc(1),
            Err(rohc_status_t::ROHC_STATUS_NO_CONTEXT) => status.set_no_ctx(1),
            Err(rohc_status_t::ROHC_STATUS_MALFORMED) => status.set_bad_fmt(1),
            _ => panic!("[rohc engine]unexpect error {:?}", r),
        }
        if !status.is_err() {
            if req
                .dst
                .write_with(
                    &dst,
                    |b| {
                        tracing::event!(
                            target: STATICS_TAR,
                            logger::STATICS_LEVEL,
                            name = "dst write data",
                            addr = b.addr,
                            size = b.size
                        );
                    },
                    |addr, size| {
                        if size > 1 {
                            tracing::event!(
                                target: STATICS_TAR,
                                logger::STATICS_LEVEL,
                                name = "dst read sc-list",
                                addr = addr,
                                size = size * std::mem::size_of::<crate::desc::SCBufferEntry>(),
                            );
                        }
                    },
                )
                .is_err()
            {
                status.set_dst_err(1);
            }
        }
        status
    }
}
