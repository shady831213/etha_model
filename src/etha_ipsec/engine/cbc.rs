use super::IpsecEngineOpts;
use aes::{
    cipher::{block_padding::NoPadding, BlockDecryptMut, BlockEncryptMut, KeyIvInit},
    Aes128, Aes256,
};
use cbc::{Decryptor, Encryptor};
type Aes128CbcEnc = Encryptor<Aes128>;
type Aes128CbcDec = Decryptor<Aes128>;
type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;
impl<'a> IpsecEngineOpts<'a> {
    pub(super) fn aes_cbc(&mut self) {
        let iv = self.iv();
        if iv.len() != 16 {
            self.status.set_ciper_err(1);
            println!(
                "Ipsec Engine: cbc Error! iv len is {}, which is expect to be 16",
                iv.len()
            );
            return;
        }
        if self.context.cipher_key_len() == 256 / 8 {
            if self.cfg.cfg.encrypt() == 1 {
                self.do_aes_cbc_encrypt(
                    Aes256CbcEnc::new_from_slices(self.context.aes_key.as_ref().unwrap(), &iv)
                        .unwrap(),
                )
            } else {
                self.do_aes_cbc_decrypt(
                    Aes256CbcDec::new_from_slices(self.context.aes_key.as_ref().unwrap(), &iv)
                        .unwrap(),
                )
            }
        } else {
            if self.cfg.cfg.encrypt() == 1 {
                self.do_aes_cbc_encrypt(
                    Aes128CbcEnc::new_from_slices(self.context.aes_key.as_ref().unwrap(), &iv)
                        .unwrap(),
                )
            } else {
                self.do_aes_cbc_decrypt(
                    Aes128CbcDec::new_from_slices(self.context.aes_key.as_ref().unwrap(), &iv)
                        .unwrap(),
                )
            }
        }
    }
    fn do_aes_cbc_encrypt<AES: BlockEncryptMut>(&mut self, aes: AES) {
        let r = aes.encrypt_padded_vec_mut::<NoPadding>(&self.src_text());
        self.set_dst_text(&r);
        self.hmac_digest();
    }
    fn do_aes_cbc_decrypt<AES: BlockDecryptMut>(&mut self, aes: AES) {
        if self.hmac_verify() {
            aes.decrypt_padded_vec_mut::<NoPadding>(&self.src_text())
                .map(|r| {
                    self.set_dst_text(&r);
                })
                .unwrap_or_else(|e| {
                    self.status.set_ciper_err(1);
                    println!("Ipsec Engine: cbc Error!{:?}", e);
                })
        }
    }
}
