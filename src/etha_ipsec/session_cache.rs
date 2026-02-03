use super::reg_if::sessions::*;
use super::STATICS_TAR;
use super::*;
use crate::logger;
use std::sync::{Arc, Mutex};
//clock handle cache: https://www.cs.swarthmore.edu/~margarel/Papers/CS25.pdf
pub struct IpsecCacheEntry<const N: usize> {
    data: [u8; N],
    id: u8,
    valid: bool,
    visited: bool,
}

impl<const N: usize> IpsecCacheEntry<N> {
    fn new() -> Self {
        IpsecCacheEntry {
            data: [0; N],
            id: 0,
            valid: false,
            visited: false,
        }
    }
}

trait Cache<const M: usize> {
    fn get_cached(&mut self, id: u8, data: &[u8]) -> [u8; M];
}

pub struct IpsecCacheT<const N: usize, const M: usize> {
    entries: [IpsecCacheEntry<M>; N],
    clock: usize,
}

impl<const N: usize, const M: usize> Cache<M> for IpsecCacheT<N, M> {
    fn get_cached(&mut self, id: u8, data: &[u8]) -> [u8; M] {
        self.get(id).unwrap_or_else(|| {
            tracing::event!(
                target: STATICS_TAR,
                logger::STATICS_LEVEL,
                name = "cache refill read data",
                addr = data.as_ptr() as u64,
                size = data.len()
            );
            let id = self.refill(id, data);
            self.entries[id].visited = true;
            self.entries[id].data
        })
    }
}

impl<const N: usize, const M: usize> IpsecCacheT<N, M> {
    fn new() -> Self {
        IpsecCacheT {
            entries: array_init::array_init(|_| IpsecCacheEntry::new()),
            clock: 0,
        }
    }
    fn get(&mut self, id: u8) -> Option<[u8; M]> {
        for e in self.entries.iter_mut() {
            if e.valid && e.id == id {
                e.visited = true;
                return Some(e.data);
            }
        }
        None
    }
    fn refill(&mut self, id: u8, data: &[u8]) -> usize {
        for (i, e) in self.entries.iter_mut().enumerate() {
            if !e.valid {
                e.id = id;
                e.data[..data.len()].copy_from_slice(data);
                e.valid = true;
                return i;
            }
        }
        self.lru(id, data)
    }

    fn advance_clock(&mut self) -> usize {
        let cur = self.clock;
        if cur == N - 1 {
            self.clock = 0;
        } else {
            self.clock += 1;
        }
        cur
    }
    fn lru(&mut self, id: u8, data: &[u8]) -> usize {
        loop {
            let i = self.advance_clock();
            let e = &mut self.entries[i];
            if !e.visited {
                e.id = id;
                e.data[..data.len()].copy_from_slice(data);
                return i;
            } else {
                e.visited = false;
            }
        }
    }

    fn invalid(&mut self, id: u8) {
        for e in self.entries.iter_mut() {
            if e.id == id {
                e.valid = false;
                e.visited = false;
                break;
            }
        }
    }
}

type IpsecAesKeyCache = IpsecCacheT<IPSEC_CACHE_NUM, { 256 / 8 }>;
type IpsecHashKeyCache = IpsecCacheT<IPSEC_CACHE_NUM, { 1024 / 8 }>;

#[derive(Debug)]
pub struct IpsecContextCfg {
    pub cipher_alg: CipherAlg,
    pub cipher_mode: CipherMode,
    pub hmac_alg: HmacAlg,
    pub salt: Vec<u8>,
    icv_len: usize,
    iv_len: usize,
}

impl IpsecContextCfg {
    pub fn cipher_key_len(&self) -> usize {
        (match self.cipher_alg {
            CipherAlg::AES128 => 128,
            CipherAlg::AES256 => 256,
            _ => 0,
        }) / 8
    }
    pub fn iv_len(&self) -> usize {
        if self.iv_len != 0 {
            self.iv_len
        } else {
            match self.cipher_mode {
                CipherMode::CCM | CipherMode::GCM => 8,
                CipherMode::CBC => 16,
                _ => 0,
            }
        }
    }
    pub fn hash_key_len(&self) -> usize {
        (match self.hmac_alg {
            HmacAlg::SHA1 => 128,
            HmacAlg::SHA256 => 512,
            HmacAlg::SHA512 => 1024,
            _ => 0,
        }) / 8
    }
    pub fn icv_len(&self) -> usize {
        if self.icv_len != 0 {
            self.icv_len
        } else {
            match self.cipher_mode {
                CipherMode::CCM => 8,
                CipherMode::GCM => 16,
                _ => match self.hmac_alg {
                    HmacAlg::SHA1 => 20,
                    HmacAlg::SHA256 | HmacAlg::SHA512 => 32,
                    _ => 0,
                },
            }
        }
    }
}

pub struct IpsecContext {
    cfg: IpsecContextCfg,
    pub aes_key: Option<Vec<u8>>,
    pub hash_key: Option<Vec<u8>>,
}

impl std::ops::Deref for IpsecContext {
    type Target = IpsecContextCfg;
    fn deref(&self) -> &Self::Target {
        &self.cfg
    }
}

pub struct IpsecSessionCache {
    regs: Arc<SecSessions<IPSEC_SESSION_NUM>>,
    aes_key_cache: Arc<Mutex<IpsecAesKeyCache>>,
    hash_key_cache: Arc<Mutex<IpsecHashKeyCache>>,
}

impl IpsecSessionCache {
    pub fn new(regs: &Arc<SecSessions<IPSEC_SESSION_NUM>>) -> Self {
        let cache = IpsecSessionCache {
            regs: regs.clone(),
            aes_key_cache: Arc::new(Mutex::new(IpsecAesKeyCache::new())),
            hash_key_cache: Arc::new(Mutex::new(IpsecHashKeyCache::new())),
        };
        for (id, s) in cache.regs.sessions.iter().enumerate() {
            let aes_key_cache = cache.aes_key_cache.clone();
            let hash_key_cache = cache.hash_key_cache.clone();
            s.ctx_mut().set_valid_transform(move |v| {
                aes_key_cache.lock().unwrap().invalid(id as u8);
                hash_key_cache.lock().unwrap().invalid(id as u8);
                v
            })
        }
        cache
    }

    pub fn get_context(&self, session: usize) -> Option<IpsecContext> {
        self.get_session(session).map(|s| {
            let cipher_key = s.cipher_key_addr();
            let hash_key = s.hash_key_addr();
            let salt = s.get_salt();
            let ctx = s.ctx();
            let cfg = IpsecContextCfg {
                cipher_alg: ctx.get_cipher_alg(),
                cipher_mode: ctx.get_cipher_mode(),
                hmac_alg: ctx.get_hmac_alg(),
                icv_len: ctx.icv_len() as usize,
                iv_len: ctx.iv_len() as usize,
                salt,
            };
            tracing::debug!(target : "ipsec-engine-cache", "get ctx cfg {:?}!", cfg);
            let aes_key = self.aes_key(s.id as u8, cipher_key, cfg.cipher_key_len());
            tracing::debug!(target : "ipsec-engine-cache", "get cipher key!");
            let hash_key = self.hash_key(s.id as u8, hash_key, cfg.hash_key_len());
            tracing::debug!(target : "ipsec-engine-cache", "get hash_key!");
            IpsecContext {
                cfg,
                aes_key,
                hash_key,
            }
        })
    }

    fn get_session<'a>(&'a self, session: usize) -> Option<&'a SecSessionRegs> {
        if session < self.regs.sessions.len() {
            match self.regs.sessions[session].ctx().check_session() {
                Ok(_) => Some(&self.regs.sessions[session]),
                Err(msg) => {
                    println!("Ipsec Session[{}]: Error!{}", session, msg);
                    None
                }
            }
        } else {
            None
        }
    }
    fn cached_key<const N: usize, C: Cache<N>>(
        &self,
        cache: &Arc<Mutex<C>>,
        id: u8,
        addr: Option<*mut u8>,
        len: usize,
    ) -> Option<Vec<u8>> {
        addr.map(|addr| {
            let key_in_mem = unsafe { std::slice::from_raw_parts(addr, len) };
            Some(cache.lock().unwrap().get_cached(id, &key_in_mem)[..len].to_vec())
        })
        .flatten()
    }

    fn aes_key(&self, id: u8, addr: Option<*mut u8>, len: usize) -> Option<Vec<u8>> {
        tracing::debug!(target : "ipsec-engine-cache", "aes_key!");
        self.cached_key(&self.aes_key_cache, id, addr, len)
    }
    fn hash_key(&self, id: u8, addr: Option<*mut u8>, len: usize) -> Option<Vec<u8>> {
        self.cached_key(&self.hash_key_cache, id, addr, len)
    }
}
