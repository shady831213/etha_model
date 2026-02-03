use super::etha_ipsec_core::EthaIpsecCore;
use super::reg_if::TopRegs;
use super::*;
use crate::aborter::*;
use crate::arbiter::*;
use crate::irq::*;
use core_affinity::{set_for_current, CoreId};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct EthaIpsec<A: Arbiter> {
    core: EthaIpsecCore<A>,
    regs: Arc<TopRegs<IPSEC_CH_NUM, IPSEC_SESSION_NUM>>,
}

impl<A: Arbiter> EthaIpsec<A> {
    pub fn new(arbiter: A) -> Self {
        let regs = Arc::new(TopRegs::new());
        EthaIpsec {
            core: EthaIpsecCore::new(arbiter, &regs),
            regs,
        }
    }

    pub fn abort(&self) -> Arc<Aborter> {
        self.core.abort()
    }
    pub fn regs(&self) -> Arc<TopRegs<IPSEC_CH_NUM, IPSEC_SESSION_NUM>> {
        self.regs.clone()
    }
    pub fn irqs(&self) -> Arc<Mutex<IrqVec>> {
        self.core.irqs()
    }
}

impl<A: Arbiter + Send + 'static> EthaIpsec<A> {
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
    use hex_literal::hex;
    macro_rules! algm_test {
        (
         $name:ident (
         key: $key: expr_2021,
         salt: $salt: expr_2021,
         hash_key: $hash_key: expr_2021,
         alg: $alg: expr_2021,
         mode: $mode: expr_2021,
         hmac: $hmac: expr_2021,
         pt: $pt: expr_2021,
         aad: $aad: expr_2021,
         ct: $ct: expr_2021,
         icv: $icv: expr_2021,
         iv: $iv: expr_2021
         )
        ) => {
            #[test_log::test]
            fn $name() {
                let etha_ipsec = EthaIpsec::new(RRArbiter::<IPSEC_CH_NUM>::new());
                let abort = etha_ipsec.abort();
                let reg = etha_ipsec.regs();
                let t = etha_ipsec.spawn(Some(CoreId { id: 0 }));
                let driver = SwIpsec::new(&reg);
                let mut ch0 = driver.alloc_ch(1);
                let mut ch1 = driver.alloc_ch(1);
                let plaintext = $pt;
                let aad = $aad;
                let ciphertext = $ct;
                let icv = $icv;
                let iv = $iv;
                let sa =
                    driver.alloc_session(&$key, &$salt, &$hash_key, iv.len(), $alg, $mode, $hmac);
                let mut cipher_result: Vec<u8> = vec![0; iv.len() + plaintext.len() + icv.len()];
                let r = ch0.xform(
                    &[&aad[..], &iv[..], &plaintext[..]],
                    &[&mut cipher_result[..]],
                    {
                        let mut cfg = IpsecFrameCfgDesc::default();
                        cfg.set_aad_len(aad.len() as u32);
                        cfg.set_session_id(sa.id as u32);
                        cfg.set_text_len(plaintext.len() as u32);
                        cfg.set_resp_en(1);
                        cfg.set_encrypt(1);
                        cfg.set_iv_copy(1);
                        let mut src = IpsecFrameFmtDesc::default();
                        src.set_iv_offset(aad.len() as u32);
                        src.set_text_offset((aad.len() + iv.len()) as u32);
                        let mut dst = IpsecFrameFmtDesc::default();
                        dst.set_text_offset(iv.len() as u32);
                        dst.set_icv_offset((iv.len() + plaintext.len()) as u32);
                        IpsecCfgDesc { src, dst, cfg }
                    },
                );
                println!("{:?}", r);
                assert!(!r.is_err());
                println!("{:x?}", &cipher_result);
                assert_eq!(
                    &cipher_result,
                    &[&iv[..], &ciphertext[..], &icv[..]].concat()
                );

                let mut plain_result: Vec<u8> = vec![0; aad.len() + plaintext.len()];
                let r = ch1.xform(&[&aad[..], &cipher_result[..]], &[&mut plain_result[..]], {
                    let mut cfg = IpsecFrameCfgDesc::default();
                    cfg.set_aad_len(aad.len() as u32);
                    cfg.set_session_id(sa.id as u32);
                    cfg.set_text_len(ciphertext.len() as u32);
                    cfg.set_resp_en(1);
                    cfg.set_aad_copy(1);
                    let mut src = IpsecFrameFmtDesc::default();
                    src.set_iv_offset(aad.len() as u32);
                    src.set_text_offset((aad.len() + iv.len()) as u32);
                    src.set_icv_offset((aad.len() + iv.len() + plaintext.len()) as u32);
                    let mut dst = IpsecFrameFmtDesc::default();
                    dst.set_text_offset(aad.len() as u32);
                    IpsecCfgDesc { src, dst, cfg }
                });
                println!("{:?}", r);
                assert!(!r.is_err());
                println!("{:x?}", &plain_result);
                assert_eq!(&plain_result, &[&aad[..], &plaintext[..]].concat());

                abort.abort();
                t.join().unwrap();
            }
        };
    }
    //testvector from https://github.com/RustCrypto/AEADs/blob/master/aes-gcm/tests/aes256gcm.rs#L1417
    algm_test!(
        gcm_test(
            key: hex!("dd73670fb221f7ee185f5818065e22dda3780fc900fc02ef00232c661d7bffce"),
            salt:hex!("c33de653"),
            hash_key: [],
            alg: CipherAlg::AES256,
            mode: CipherMode::GCM,
            hmac: HmacAlg::Null,
            pt:  hex!("ada4d98147b30e5a901229952a"),
            aad: hex!("e1a5e52427f1c5b887575a6f2c445429"),
            ct: hex!("6ed4e4bd1f953d47c5288c48f4"),
            icv: hex!("404e3a9b9f5ddab9ee169a7c7c2cf7af"),
            iv: hex!("44cfbf228e1652bd")
        )
    );

    //testvector from https://www.rfc-editor.org/rfc/rfc3610 Packet Vector #2
    algm_test!(
        ccm_test(
            key: hex!("C0 C1 C2 C3  C4 C5 C6 C7  C8 C9 CA CB  CC CD CE CF"),
            salt: [],
            hash_key: [],
            alg: CipherAlg::AES128,
            mode: CipherMode::CCM,
            hmac: HmacAlg::Null,
            pt:  hex!(
                "08 09 0A 0B  0C 0D 0E 0F  10 11 12 13  14 15 16 17  18 19 1A 1B  1C 1D 1E 1F"
            ),
            aad: hex!("00 01 02 03  04 05 06 07"),
            ct: hex!(
                "72 C9 1A 36  E1 35 F8 CF  29 1C A8 94  08 5C 87 E3  CC 15 C4 39  C9 E4 3A 3B"
            ),
            icv: hex!("A0 91 D5 6E  10 40 09 16"),
            iv: hex!("00 00 00 04  03 02 01 A0  A1 A2 A3 A4  A5")
        )
    );

    //testvector from https://www.rfc-editor.org/rfc/rfc3602 Case #5
    algm_test!(
        cbc_test(
            key: hex!("90d382b4 10eeba7a d938c46c ec1a82bf"),
            salt: [],
            hash_key: [],
            alg: CipherAlg::AES128,
            mode: CipherMode::CBC,
            hmac: HmacAlg::Null,
            pt:  hex!(
                "08000ebd a70a0000 8e9c083d b95b0700 08090a0b 0c0d0e0f 10111213 14151617
                18191a1b 1c1d1e1f 20212223 24252627 28292a2b 2c2d2e2f 30313233 34353637
                01020304 05060708 090a0b0c 0d0e0e01"
            ),
            aad: [],
            ct: hex!(
                "f663c25d 325c18c6 a9453e19 4e120849 a4870b66 cc6b9965 330013b4 898dc856
                a4699e52 3a55db08 0b59ec3a 8e4b7e52 775b07d1 db34ed9c 538ab50c 551b874a
                a269add0 47ad2d59 13ac19b7 cfbad4a6"
            ),
            icv: hex!(""),
            iv: hex!("e96e8c08 ab465763 fd098d45 dd3ff893")
        )
    );

    //testvector from https://www.rfc-editor.org/rfc/rfc4868 Test Case PRF-2
    algm_test!(
        hmac_test(
            key: [],
            salt: [],
            hash_key: hex!(
                "4a656665 00000000 00000000 00000000
                00000000 00000000 00000000 00000000
                00000000 00000000 00000000 00000000
                00000000 00000000 00000000 00000000"
            ),
            alg: CipherAlg::Null,
            //mode can be any
            mode: CipherMode::CBC,
            hmac: HmacAlg::SHA256,
            pt:  hex!(
                "7768617420646f2079612077616e7420666f72206e6f7468696e673f"
            ),
            aad: [],
            ct: hex!(
                "7768617420646f2079612077616e7420666f72206e6f7468696e673f"
            ),
            icv: hex!("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"),
            iv: []
        )
    );

    //testvector from https://github.com/RustCrypto/AEADs/blob/master/aes-gcm/tests/aes256gcm.rs#L609
    algm_test!(
        gmac_test(
            key: hex!("dd95259bc8eefa3e493cb1a6ba1d8ee2b341d5230d50363094a2cc3433b3d9b9"),
            salt:hex!("a1a6ced0"),
            hash_key: [],
            alg: CipherAlg::AES256,
            mode: CipherMode::GCM,
            hmac: HmacAlg::Null,
            pt:  [],
            aad: hex!("d46db90e13684b26149cb3b7f776e228a0538fa1892c418aaad07aa08d3076f4a52bee8f130ff560db2b8d1009e9260fa6233fc22733e050c9e4f7cc699062765e261dffff1159e9060b26c8065dfab04055b58c82c340d987c9"),
            ct: [],
            icv: hex!("9e120b01899fe2cb3e3a0b0c05045940"),
            iv: hex!("84f4f13990750a9e")
        )
    );
}

#[cfg(test)]
mod tests_driver_helper {
    use super::*;
    pub(super) use crate::desc::*;
    pub(super) use crate::etha_ipsec::desc::req::*;
    pub(super) use crate::etha_ipsec::desc::resp::*;
    pub(super) use crate::etha_ipsec::reg_if::sessions::*;
    pub(super) use crate::etha_ipsec::reg_if::*;
    pub(super) use crate::reg_if::{
        ring::{sw_ring::*, *},
        RegBus,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub(super) const fn addr(offset: usize) -> u64 {
        offset as u64
    }
    type RegT = TopRegs<IPSEC_CH_NUM, IPSEC_SESSION_NUM>;

    pub(super) struct SwIpsecQueue {
        id: usize,
        resp_ptr: usize,
        ring: SwQueue<
            RegT,
            [DescEntryT; IPSEC_REQ_ENTRY_SIZE / DESC_ENTRY_SIZE],
            [DescEntryT; IPSEC_RESULT_ENTRY_SIZE / DESC_ENTRY_SIZE],
        >,
    }

    impl SwIpsecQueue {
        pub(super) fn new(regs: &Arc<RegT>, id: usize, ring_size: usize) -> Self {
            SwIpsecQueue {
                id,
                resp_ptr: 0,
                ring: SwQueue::new(
                    regs,
                    queue_base(id),
                    ring_size,
                    vec![[0; IPSEC_REQ_ENTRY_SIZE / DESC_ENTRY_SIZE]; ring_size],
                    vec![[0; IPSEC_RESULT_ENTRY_SIZE / DESC_ENTRY_SIZE]; ring_size],
                ),
            }
        }
        pub(super) fn init(&self) {
            self.ring.init()
        }
        pub(super) fn init_check(&self) {
            println!(
                "SwQueue[{}]: producer_valids: {}",
                self.id,
                self.r_p_valids()
            );
            assert_eq!(self.r_p_valids(), self.ring_size);
            println!("SwQueue[{}]: empty: {}", self.id, self.r_empty());
            assert_eq!(self.r_empty(), true);
        }
        pub(super) fn xform(
            &mut self,
            src: &[&[u8]],
            dst: &[&mut [u8]],
            cfg: IpsecCfgDesc,
        ) -> IpsecResultDesc {
            loop {
                if !self.r_full() {
                    break;
                }
            }
            let src = src
                .iter()
                .map(|v| {
                    SCBufferEntry::from(MemBlock {
                        addr: v.as_ptr() as usize as u64,
                        size: v.len(),
                    })
                })
                .collect::<Vec<_>>();
            let dst = dst
                .iter()
                .map(|v| {
                    SCBufferEntry::from(MemBlock {
                        addr: v.as_ptr() as usize as u64,
                        size: v.len(),
                    })
                })
                .collect::<Vec<_>>();
            let req = IpsecReqDesc {
                src: SCFrameDesc::try_from(&src[..]).unwrap(),
                dst: SCFrameDesc::try_from(&dst[..]).unwrap(),
                cfg,
            };
            self.r_push_req(&[req]).unwrap();
            let resp_ptr = self.resp_ptr;
            self.resp_ptr = self.r_p_ptr();
            loop {
                if self.r_empty() {
                    break;
                }
            }
            unsafe { *self.r_get_resp_at(resp_ptr).unwrap() }
        }
    }

    impl SwRing for SwIpsecQueue {
        type REQ = IpsecReqDesc;
        type RESP = IpsecResultDesc;
        type R = SwQueue<
            RegT,
            [DescEntryT; IPSEC_REQ_ENTRY_SIZE / DESC_ENTRY_SIZE],
            [DescEntryT; IPSEC_RESULT_ENTRY_SIZE / DESC_ENTRY_SIZE],
        >;
        const REQ_SIZE: usize = IPSEC_REQ_ENTRY_SIZE;
        const RESP_SIZE: usize = IPSEC_RESULT_ENTRY_SIZE;

        fn get_ring(&self) -> &Self::R {
            &self.ring
        }
    }

    impl std::ops::Deref for SwIpsecQueue {
        type Target = SwQueue<
            RegT,
            [DescEntryT; IPSEC_REQ_ENTRY_SIZE / DESC_ENTRY_SIZE],
            [DescEntryT; IPSEC_RESULT_ENTRY_SIZE / DESC_ENTRY_SIZE],
        >;
        fn deref(&self) -> &Self::Target {
            self.get_ring()
        }
    }

    const SESS_CTX_OFFSET: usize = 0;
    const SESS_SALT_OFFSET: usize = 1;
    const SESS_CIPHER_KEY_LO_OFFSET: usize = 2;
    const SESS_CIPHER_KEY_HI_OFFSET: usize = 3;
    const SESS_HASH_KEY_LO_OFFSET: usize = 4;
    const SESS_HASH_KEY_HI_OFFSET: usize = 5;
    const fn sess_valid(v: u64) -> u64 {
        v & 0x1
    }
    const fn sess_cipher_alg(v: u64) -> u64 {
        (v & 0x3) << 1
    }
    const fn sess_cipher_mode(v: u64) -> u64 {
        (v & 0x3) << 3
    }
    const fn sess_hmac_alg(v: u64) -> u64 {
        (v & 0x3) << 5
    }
    const fn sess_salt_len(v: u64) -> u64 {
        (v & 0x7) << 7
    }
    const fn sess_iv_len(v: u64) -> u64 {
        (v & 0x3f) << 10
    }

    pub(super) struct SwSession {
        regs: Arc<RegT>,
        pub id: usize,
        base: usize,
        cipher_key: &'static [u8],
        salt: &'static [u8],
        hash_key: &'static [u8],
        iv_len: usize,
        pub cipher_alg: CipherAlg,
        pub cipher_mode: CipherMode,
        pub hmac_alg: HmacAlg,
    }
    impl SwSession {
        fn new(
            regs: &Arc<RegT>,
            id: usize,
            cipher_key: &'static [u8],
            salt: &'static [u8],
            hash_key: &'static [u8],
            iv_len: usize,
            cipher_alg: CipherAlg,
            cipher_mode: CipherMode,
            hmac_alg: HmacAlg,
        ) -> Self {
            SwSession {
                regs: regs.clone(),
                id,
                base: SESSION_REGS_RANGE.start + id * SEC_SESSION_REGS_SIZE,
                cipher_key,
                salt,
                hash_key,
                iv_len,
                cipher_alg,
                cipher_mode,
                hmac_alg,
            }
        }

        pub(super) fn invalid(&self) {
            self.regs
                .write(addr(self.base + SESS_CTX_OFFSET), 0)
                .unwrap();
        }
        pub(super) fn init(&self) {
            self.invalid();
            if !self.cipher_key.is_empty() {
                self.regs
                    .write(
                        addr(self.base + SESS_CIPHER_KEY_LO_OFFSET),
                        self.cipher_key.as_ptr() as usize as u32 as u64,
                    )
                    .unwrap();
                self.regs
                    .write(
                        addr(self.base + SESS_CIPHER_KEY_HI_OFFSET),
                        (self.cipher_key.as_ptr() as usize >> 32) as u32 as u64,
                    )
                    .unwrap();
            }
            if !self.hash_key.is_empty() {
                self.regs
                    .write(
                        addr(self.base + SESS_HASH_KEY_LO_OFFSET),
                        self.hash_key.as_ptr() as usize as u32 as u64,
                    )
                    .unwrap();
                self.regs
                    .write(
                        addr(self.base + SESS_HASH_KEY_HI_OFFSET),
                        (self.hash_key.as_ptr() as usize >> 32) as u32 as u64,
                    )
                    .unwrap();
            }
            if !self.salt.is_empty() {
                let mut salt: u32 = 0;
                let len = if self.salt.len() > 4 {
                    4
                } else {
                    self.salt.len()
                };
                for i in 0..len {
                    salt |= (self.salt[i] as u32) << (i << 3)
                }
                self.regs
                    .write(addr(self.base + SESS_SALT_OFFSET), salt as u64)
                    .unwrap();
                self.regs
                    .write(
                        addr(self.base + SESS_CTX_OFFSET),
                        sess_valid(1)
                            | sess_cipher_alg(self.cipher_alg as u8 as u64)
                            | sess_cipher_mode(self.cipher_mode as u8 as u64)
                            | sess_hmac_alg(self.hmac_alg as u8 as u64)
                            | sess_salt_len(self.salt.len() as u64)
                            | sess_iv_len(self.iv_len as u64),
                    )
                    .unwrap();
            } else {
                self.regs
                    .write(
                        addr(self.base + SESS_CTX_OFFSET),
                        sess_valid(1)
                            | sess_cipher_alg(self.cipher_alg as u8 as u64)
                            | sess_cipher_mode(self.cipher_mode as u8 as u64)
                            | sess_hmac_alg(self.hmac_alg as u8 as u64)
                            | sess_iv_len(self.iv_len as u64),
                    )
                    .unwrap();
            }
        }
    }

    pub(super) struct SwIpsec {
        regs: Arc<RegT>,
        ch_id: AtomicUsize,
        sess_id: AtomicUsize,
    }
    impl SwIpsec {
        pub(super) fn new(regs: &Arc<RegT>) -> Self {
            SwIpsec {
                regs: regs.clone(),
                ch_id: AtomicUsize::new(0),
                sess_id: AtomicUsize::new(0),
            }
        }
        fn alloc_ch_id(&self) -> usize {
            let id = self.ch_id.fetch_add(1, Ordering::SeqCst);
            assert!(id < IPSEC_CH_NUM);
            id
        }
        fn alloc_sess_id(&self) -> usize {
            let id = self.sess_id.fetch_add(1, Ordering::SeqCst);
            assert!(id < IPSEC_SESSION_NUM);
            id
        }
        pub(super) fn alloc_ch(&self, size: usize) -> SwIpsecQueue {
            let id = self.alloc_ch_id();
            let ch = SwIpsecQueue::new(&self.regs, id, size);
            ch.init();
            ch.init_check();
            ch
        }
        pub(super) fn alloc_session(
            &self,
            cipher_key: &'static [u8],
            salt: &'static [u8],
            hash_key: &'static [u8],
            iv_len: usize,
            cipher_alg: CipherAlg,
            cipher_mode: CipherMode,
            hmac_alg: HmacAlg,
        ) -> SwSession {
            let id = self.alloc_sess_id();
            let sess = SwSession::new(
                &self.regs,
                id,
                cipher_key,
                salt,
                hash_key,
                iv_len,
                cipher_alg,
                cipher_mode,
                hmac_alg,
            );
            sess.init();
            sess
        }
    }
}
