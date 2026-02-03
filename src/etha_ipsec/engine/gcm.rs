use super::IpsecEngineOpts;
use aes_gcm::{
    aead::{Aead, Payload},
    Aes128Gcm, Aes256Gcm, KeyInit,
};
impl<'a> IpsecEngineOpts<'a> {
    pub(super) fn aes_gcm(&mut self) {
        if self.context.cipher_key_len() == 256 / 8 {
            self.do_aes_gcm(
                Aes256Gcm::new_from_slice(self.context.aes_key.as_ref().unwrap()).unwrap(),
            )
        } else {
            self.do_aes_gcm(
                Aes128Gcm::new_from_slice(self.context.aes_key.as_ref().unwrap()).unwrap(),
            )
        }
    }
    fn do_aes_gcm<AES: Aead>(&mut self, aes: AES) {
        let nonce = self.iv();
        if nonce.len() != 12 {
            self.status.set_ciper_err(1);
            println!(
                "Ipsec Engine: gcm Error! Nonce len is {}, which is expect to be 12",
                nonce.len()
            );
            return;
        }
        if self.cfg.cfg.encrypt() == 1 {
            aes.encrypt(
                (&nonce[..]).into(),
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
                println!("Ipsec Engine: gcm Error!{:?}", e);
            })
        } else {
            aes.decrypt(
                (&nonce[..]).into(),
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
                tracing::debug!(target : "ipsec-engine-gcm", "Ipsec Engine: gcm auth failed!{:?}", e);
            })
        }
    }
}
