use crate::reg_if::RegBus;
use etha_model_generator::*;
define_reg! {
    KeyAddress {
        fields {
            addr(RW): 31, 0;
        }
    }
}

define_reg! {
    KeyValue {
        fields {
            value(RW): 31, 0;
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CipherAlg {
    Null = 0,
    AES128 = 1,
    AES256 = 2,
    Unknown = 3,
}

impl std::convert::From<u8> for CipherAlg {
    fn from(value: u8) -> Self {
        match value {
            0 => CipherAlg::Null,
            1 => CipherAlg::AES128,
            2 => CipherAlg::AES256,
            _ => CipherAlg::Unknown,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CipherMode {
    CBC = 0,
    CCM = 1,
    GCM = 2,
    Unknown = 3,
}

impl std::convert::From<u8> for CipherMode {
    fn from(value: u8) -> Self {
        match value {
            0 => CipherMode::CBC,
            1 => CipherMode::CCM,
            2 => CipherMode::GCM,
            _ => CipherMode::Unknown,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum HmacAlg {
    Null = 0,
    SHA1 = 1,
    SHA256 = 2,
    SHA512 = 3,
}

impl std::convert::From<u8> for HmacAlg {
    fn from(value: u8) -> Self {
        match value {
            0 => HmacAlg::Null,
            1 => HmacAlg::SHA1,
            2 => HmacAlg::SHA256,
            3 => HmacAlg::SHA512,
            _ => HmacAlg::Null,
        }
    }
}

define_reg! {
    XformCtx {
        fields {
            valid(RW, volatile){invalid: 0, valid: 1}: 0, 0;
            cipher_alg(RW){null: 0, aes_128: 1, aes_256: 2}: 2, 1;
            cipher_mode(RW){cbc: 0, ccm: 1, gcm: 2}: 4, 3;
            hmac_alg(RW){null: 0, sha1: 1, sha256: 2, sha512: 3}: 6, 5;
            salt_len(RW): 9, 7;
            iv_len(RW): 15, 10;
            icv_len(RW): 27, 16;
        }
    }
}

impl LockedXformCtx {
    pub fn get_cipher_alg(&self) -> CipherAlg {
        CipherAlg::from(self.cipher_alg() as u8)
    }
    pub fn get_cipher_mode(&self) -> CipherMode {
        CipherMode::from(self.cipher_mode() as u8)
    }
    pub fn get_hmac_alg(&self) -> HmacAlg {
        HmacAlg::from(self.hmac_alg() as u8)
    }
    pub fn check_session(&self) -> std::result::Result<(), String> {
        if self.valid() == 0 {
            return Err("session valid bit is not set!".to_string());
        }
        if let CipherAlg::Unknown = self.get_cipher_alg() {
            return Err(format!("Unkonwn cipher algorithm: {:x}", self.cipher_alg()));
        }
        if let CipherMode::Unknown = self.get_cipher_mode() {
            return Err(format!("Unkonwn cipher mode: {:x}", self.cipher_mode()));
        }
        Ok(())
    }
}

pub const SEC_SESSION_REGS_SIZE: usize = 8;

reg_map! {
    pub SecSession(8) {
        ctx(RW): XformCtx, 0;
        salt(WO): KeyValue, 1;
        cipher_key_lo(WO): KeyAddress, 2;
        cipher_key_hi(WO): KeyAddress, 3;
        hash_key_lo(WO): KeyAddress, 4;
        hash_key_hi(WO): KeyAddress, 5;
    }
}

impl LockedSecSession {
    pub fn cipher_key_addr(&self) -> Option<*mut u8> {
        let alg = self.ctx().get_cipher_alg();
        tracing::debug!(target : "ipsec-engine-cache", "cipher_key_addr!");
        match alg {
            CipherAlg::AES128 | CipherAlg::AES256 => {
                let lo = self.cipher_key_lo().get() as usize;
                let hi = self.cipher_key_hi().get() as usize;
                Some(((hi << 32) | lo) as *mut u8)
            }
            _ => None,
        }
    }
    pub fn get_salt(&self) -> Vec<u8> {
        let salt_len = self.ctx().salt_len() as usize;
        if salt_len == 0 {
            vec![]
        } else {
            let salt = self.salt().get() as u32;
            salt.to_le_bytes()[..salt_len].to_vec()
        }
    }
    pub fn hash_key_addr(&self) -> Option<*mut u8> {
        let alg = self.ctx().get_hmac_alg();
        match alg {
            HmacAlg::SHA1 | HmacAlg::SHA256 | HmacAlg::SHA512 => {
                let lo = self.hash_key_lo().get() as usize;
                let hi = self.hash_key_hi().get() as usize;
                Some(((hi << 32) | lo) as *mut u8)
            }
            _ => None,
        }
    }
}

pub struct SecSessionRegs {
    pub id: usize,
    inner: LockedSecSession,
}

impl std::ops::Deref for SecSessionRegs {
    type Target = LockedSecSession;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct SecSessions<const N: usize> {
    pub sessions: [SecSessionRegs; N],
}

impl<const N: usize> SecSessions<N> {
    pub fn new() -> Self {
        SecSessions {
            sessions: array_init::array_init(|i| SecSessionRegs {
                id: i,
                inner: LockedSecSession::new(32),
            }),
        }
    }
}

impl<const N: usize> RegBus for SecSessions<N> {
    fn write(&self, addr: u64, data: u64) -> Option<()> {
        let idx = addr as usize / SEC_SESSION_REGS_SIZE;
        let offset = addr as usize % SEC_SESSION_REGS_SIZE;
        if idx < self.sessions.len() {
            self.sessions[idx].write(offset as u64, data)
        } else {
            None
        }
    }

    fn read(&self, addr: u64) -> Option<u64> {
        let idx = addr as usize / SEC_SESSION_REGS_SIZE;
        let offset = addr as usize % SEC_SESSION_REGS_SIZE;
        if idx < self.sessions.len() {
            self.sessions[idx].read(offset as u64)
        } else {
            None
        }
    }
}

impl<const N: usize> GenHeader for SecSessions<N> {
    fn render_name() -> &'static str {
        "SecSessions"
    }
    fn gen_c_header<W: std::io::Write>(header: &mut W) -> std::io::Result<()> {
        SecSession::gen_c_header(header)?;
        XformCtx::gen_c_header(header)?;

        writeln!(header, "#define SEC_SESSIONS_NUM {}", N)?;

        writeln!(
            header,
            "#define SEC_SESSION_OFFSET(base, name, i) ((base) + (SEC_SESSION_SIZE * i) + SEC_SESSION_##name##_OFFSET)",
        )?;
        Ok(())
    }
}
