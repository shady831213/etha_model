use super::{HmacAlg, IpsecEngineOpts};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512_256};
type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512_256>;
impl<'a> IpsecEngineOpts<'a> {
    pub(super) fn hmac_digest(&mut self) {
        match self.context.hmac_alg {
            HmacAlg::SHA1 => {
                let mut mac =
                    HmacSha1::new_from_slice(self.context.hash_key.as_ref().unwrap()).unwrap();
                mac.update(&[self.dst_aad(), self.dst_text()].concat());
                self.set_dst_icv(&mac.finalize().into_bytes());
            }
            HmacAlg::SHA256 => {
                let mut mac =
                    HmacSha256::new_from_slice(self.context.hash_key.as_ref().unwrap()).unwrap();
                mac.update(&[self.dst_aad(), self.dst_text()].concat());
                self.set_dst_icv(&mac.finalize().into_bytes());
            }
            HmacAlg::SHA512 => {
                let mut mac =
                    HmacSha512::new_from_slice(self.context.hash_key.as_ref().unwrap()).unwrap();
                mac.update(&[self.dst_aad(), self.dst_text()].concat());
                self.set_dst_icv(&mac.finalize().into_bytes());
            }
            _ => {}
        };
    }
    pub(super) fn hmac_verify(&mut self) -> bool {
        if match self.context.hmac_alg {
            HmacAlg::SHA1 => {
                let mut mac =
                    HmacSha1::new_from_slice(self.context.hash_key.as_ref().unwrap()).unwrap();
                mac.update(&[self.src_aad(), self.src_text()].concat());
                mac.verify_slice(&self.src_icv())
            }
            HmacAlg::SHA256 => {
                let mut mac =
                    HmacSha256::new_from_slice(self.context.hash_key.as_ref().unwrap()).unwrap();
                mac.update(&[self.src_aad(), self.src_text()].concat());
                mac.verify_slice(&self.src_icv())
            }
            HmacAlg::SHA512 => {
                let mut mac =
                    HmacSha512::new_from_slice(self.context.hash_key.as_ref().unwrap()).unwrap();
                mac.update(&[self.src_aad(), self.src_text()].concat());
                mac.verify_slice(&self.src_icv())
            }
            _ => Ok(()),
        }
        .is_err()
        {
            self.status.set_auth_fail(1);
            false
        } else {
            true
        }
    }
}
