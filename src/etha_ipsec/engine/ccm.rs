use super::IpsecEngineOpts;
use aes::{Aes128, Aes256};
use ccm::{
    aead::{Aead, KeyInit, Payload},
    consts::*,
    Ccm,
};
impl<'a> IpsecEngineOpts<'a> {
    pub(super) fn aes_ccm(&mut self) {
        macro_rules! do_aes_ccm_with_aes {
            (
                $aes:ty, $n:ty, $nonce:expr_2021
            ) => {
                if self.context.icv_len() == 16 {
                    self.do_aes_ccm(
                        Ccm::<$aes, U16, $n>::new_from_slice(
                            self.context.aes_key.as_ref().unwrap(),
                        )
                        .unwrap(),
                        $nonce,
                    )
                } else if self.context.icv_len() == 8 {
                    self.do_aes_ccm(
                        Ccm::<$aes, U8, $n>::new_from_slice(self.context.aes_key.as_ref().unwrap())
                            .unwrap(),
                        $nonce,
                    )
                } else {
                    self.status.set_invalid_session(1);
                    println!(
                        "Ipsec Engine: ccm Error! Invaid icv_len {}, valid values are 8 and 16",
                        self.context.icv_len()
                    );
                }
            };
        }

        macro_rules! do_aes_ccm_with_nonce {
            (
                $n:ty, $nonce:expr_2021
            ) => {
                if self.context.cipher_key_len() == 256 / 8 {
                    do_aes_ccm_with_aes!(Aes256, $n, $nonce)
                } else {
                    do_aes_ccm_with_aes!(Aes128, $n, $nonce)
                }
            };
        }
        let nonce = self.iv();
        match nonce.len() {
            7 => {
                do_aes_ccm_with_nonce!(U7, &nonce)
            }
            8 => {
                do_aes_ccm_with_nonce!(U8, &nonce)
            }
            9 => {
                do_aes_ccm_with_nonce!(U9, &nonce)
            }
            10 => {
                do_aes_ccm_with_nonce!(U10, &nonce)
            }
            11 => {
                do_aes_ccm_with_nonce!(U11, &nonce)
            }
            12 => {
                do_aes_ccm_with_nonce!(U12, &nonce)
            }
            13 => {
                do_aes_ccm_with_nonce!(U13, &nonce)
            }
            _ => {
                self.status.set_invalid_session(1);
                println!(
                    "Ipsec Engine: ccm Error! Invaid nonce_len {}, valid values 7 to 13",
                    nonce.len()
                );
            }
        }
    }
    fn do_aes_ccm<AES: Aead>(&mut self, aes: AES, nonce: &[u8]) {
        if self.cfg.cfg.encrypt() == 1 {
            aes.encrypt(
                nonce.into(),
                Payload {
                    aad: self.src_aad(),
                    msg: self.src_text(),
                },
            )
            .map(|r| {
                let text_len = self.cfg.cfg.text_len() as usize;
                self.set_dst_text(&r[..text_len])
                    .set_dst_icv(&r[text_len..]);
            })
            .unwrap_or_else(|e| {
                self.status.set_ciper_err(1);
                println!("Ipsec Engine: ccm Error!{:?}", e);
            })
        } else {
            aes.decrypt(
                nonce.into(),
                Payload {
                    aad: self.src_aad(),
                    msg: &[self.src_text(), self.src_icv()].concat(),
                },
            )
            .map(|r| {
                self.set_dst_text(&r);
            })
            .unwrap_or_else(|e| {
                self.status.set_auth_fail(1);
                tracing::debug!(target : "ipsec-engine-ccm", "Ipsec Engine: ccm auth failed!{:?}", e);
            })
        }
    }
}
