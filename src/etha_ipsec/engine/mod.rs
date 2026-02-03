use super::desc::req::*;
use super::desc::resp::*;
use super::reg_if::sessions::*;
use super::session_cache::*;
use super::STATICS_TAR;
use super::*;
use crate::logger;
use std::io::Read;
use std::sync::Arc;
mod cbc;
mod ccm;
mod gcm;
mod hmac;
mod null;
struct IpsecEngineOpts<'a> {
    context: &'a IpsecContext,
    cfg: &'a IpsecCfgDesc,
    src: &'a [u8],
    dst: &'a mut [u8],
    status: &'a mut IpsecStatusDesc,
}
impl<'a> IpsecEngineOpts<'a> {
    fn src_aad(&'a self) -> &'a [u8] {
        &self.src[self.cfg.src.aad_offset() as usize
            ..self.cfg.src.aad_offset() as usize + self.cfg.cfg.aad_len() as usize]
    }
    fn src_text(&'a self) -> &'a [u8] {
        &self.src[self.cfg.src.text_offset() as usize
            ..self.cfg.src.text_offset() as usize + self.cfg.cfg.text_len() as usize]
    }
    fn src_icv(&'a self) -> &'a [u8] {
        &self.src[self.cfg.src.icv_offset() as usize
            ..self.cfg.src.icv_offset() as usize + self.context.icv_len()]
    }
    fn iv(&'a self) -> Vec<u8> {
        [
            &self.context.salt[..],
            &self.src[self.cfg.src.iv_offset() as usize
                ..self.cfg.src.iv_offset() as usize + self.context.iv_len()],
        ]
        .concat()
    }
    fn check_src(&self) -> bool {
        if self.cfg.src.aad_offset() as usize + self.cfg.cfg.aad_len() as usize > self.src.len()
            && self.cfg.cfg.aad_len() > 0
        {
            println!("Ipsec engine: Warning! aad size is bigger than src buffer size!");
            false
        } else if self.cfg.src.text_offset() as usize + self.cfg.cfg.text_len() as usize
            > self.src.len()
            && self.cfg.cfg.text_len() > 0
        {
            println!("Ipsec engine: Warning! text size is bigger than src buffer size!");
            false
        } else if self.cfg.src.iv_offset() as usize + self.context.iv_len() > self.src.len()
            && self.context.iv_len() > 0
        {
            println!("Ipsec engine: Warning! iv offset + len is bigger than src buffer size!");
            false
        } else if self.cfg.src.icv_offset() as usize + self.context.icv_len() > self.src.len()
            && self.context.icv_len() > 0
            && self.cfg.cfg.encrypt() == 0
        {
            println!("Ipsec engine: Warning! icv offset + len is bigger than src buffer size!");
            false
        } else {
            true
        }
    }
    fn dst_aad(&'a self) -> &'a [u8] {
        &self.dst[self.cfg.dst.aad_offset() as usize
            ..self.cfg.dst.aad_offset() as usize + self.cfg.cfg.aad_len() as usize]
    }
    fn dst_text(&'a self) -> &'a [u8] {
        &self.dst[self.cfg.dst.text_offset() as usize
            ..self.cfg.dst.text_offset() as usize + self.cfg.cfg.text_len() as usize]
    }
    fn set_dst_text(&mut self, text: &[u8]) -> &mut Self {
        self.dst[self.cfg.dst.text_offset() as usize
            ..self.cfg.dst.text_offset() as usize + self.cfg.cfg.text_len() as usize]
            .copy_from_slice(text);
        self
    }
    fn set_dst_icv(&mut self, icv: &[u8]) -> &mut Self {
        self.dst[self.cfg.dst.icv_offset() as usize
            ..self.cfg.dst.icv_offset() as usize + self.context.icv_len()]
            .copy_from_slice(icv);
        self
    }
    fn check_dst(&self) -> bool {
        if self.cfg.dst.aad_offset() as usize + self.cfg.cfg.aad_len() as usize > self.dst.len()
            && self.cfg.cfg.aad_len() > 0
            && self.cfg.cfg.aad_copy() == 1
        {
            println!("Ipsec engine: Warning! aad size is bigger than dst buffer size!");
            false
        } else if self.cfg.dst.text_offset() as usize + self.cfg.cfg.text_len() as usize
            > self.dst.len()
            && self.cfg.cfg.text_len() > 0
        {
            println!("Ipsec engine: Warning! text size is bigger than dst buffer size!");
            false
        } else if self.cfg.dst.iv_offset() as usize + self.context.iv_len() > self.dst.len()
            && self.context.iv_len() > 0
            && self.cfg.cfg.iv_copy() == 1
        {
            println!("Ipsec engine: Warning! iv offset + len is bigger than dst buffer size!");
            false
        } else if self.cfg.dst.icv_offset() as usize + self.context.icv_len() > self.dst.len()
            && self.context.icv_len() > 0
            && self.cfg.cfg.encrypt() == 1
        {
            println!("Ipsec engine: Warning! icv offset + len is bigger than dst buffer size!");
            false
        } else {
            true
        }
    }

    fn copy_aad(&mut self) -> &mut Self {
        if self.cfg.cfg.aad_copy() == 1 {
            self.dst[self.cfg.dst.aad_offset() as usize
                ..self.cfg.dst.aad_offset() as usize + self.cfg.cfg.aad_len() as usize]
                .copy_from_slice(
                    &self.src[self.cfg.src.aad_offset() as usize
                        ..self.cfg.src.aad_offset() as usize + self.cfg.cfg.aad_len() as usize],
                );
        }
        self
    }
    fn copy_iv(&mut self) -> &mut Self {
        if self.cfg.cfg.iv_copy() == 1 {
            self.dst[self.cfg.dst.iv_offset() as usize
                ..self.cfg.dst.iv_offset() as usize + self.context.iv_len()]
                .copy_from_slice(
                    &self.src[self.cfg.src.iv_offset() as usize
                        ..self.cfg.src.iv_offset() as usize + self.context.iv_len()],
                );
        }
        self
    }
    fn copy_text(&mut self) -> &mut Self {
        self.dst[self.cfg.dst.text_offset() as usize
            ..self.cfg.dst.text_offset() as usize + self.cfg.cfg.text_len() as usize]
            .copy_from_slice(
                &self.src[self.cfg.src.text_offset() as usize
                    ..self.cfg.src.text_offset() as usize + self.cfg.cfg.text_len() as usize],
            );
        self
    }
}
impl<'a> IpsecEngineOpts<'a> {
    fn xform(&mut self) {
        if !self.check_src() {
            self.status.set_src_err(1);
            return;
        }
        if !self.check_dst() {
            self.status.set_dst_err(1);
            return;
        }
        self.copy_aad();
        self.copy_iv();
        match self.context.cipher_alg {
            CipherAlg::AES128 | CipherAlg::AES256 => match self.context.cipher_mode {
                CipherMode::GCM => self.aes_gcm(),
                CipherMode::CCM => self.aes_ccm(),
                CipherMode::CBC => self.aes_cbc(),
                _ => unreachable!(),
            },
            CipherAlg::Null => self.null(),
            _ => unreachable!(),
        }
    }
}

pub struct IpsecEngine {
    cache: IpsecSessionCache,
}

impl IpsecEngine {
    pub fn new(regs: &Arc<SecSessions<IPSEC_SESSION_NUM>>) -> IpsecEngine {
        IpsecEngine {
            cache: IpsecSessionCache::new(regs),
        }
    }
    pub fn process(&self, mut req: IpsecReqDesc) -> IpsecStatusDesc {
        let mut status = IpsecStatusDesc::default();
        let mut src = vec![0u8; crate::mac::MAC_MAX_LEN];
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
        src.truncate(req.src.total_size() as usize);
        tracing::debug!(target : "ipsec-engine", "load src!");
        let mut dst = vec![0u8; crate::mac::MAC_MAX_LEN];
        req.dst
            .read(&mut dst)
            .expect("Ipsec engine: dst sync error!");
        dst.truncate(req.dst.total_size() as usize);
        tracing::debug!(target : "ipsec-engine", "load dst!");
        if let Some(context) = self.cache.get_context(req.cfg.cfg.session_id() as usize) {
            let mut opts = IpsecEngineOpts {
                context: &context,
                cfg: &req.cfg,
                src: &src,
                dst: &mut dst,
                status: &mut status,
            };
            tracing::debug!(target : "ipsec-engine", "begin xform!");
            opts.xform();
        } else {
            status.set_invalid_session(1);
        }
        if !status.is_err() {
            req.dst
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
                .unwrap();
        }
        status
    }
}
