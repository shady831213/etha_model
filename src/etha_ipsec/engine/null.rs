use super::IpsecEngineOpts;
impl<'a> IpsecEngineOpts<'a> {
    pub(super) fn null(&mut self) {
        if self.cfg.cfg.encrypt() == 1 {
            self.copy_text();
            self.hmac_digest();
        } else {
            if self.hmac_verify() {
                self.copy_text();
            }
        }
    }
}
