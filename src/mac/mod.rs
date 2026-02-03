mod pcap_mac;
pub use pcap_mac::{pcap_cmp, PcapMac, PcapMacRx, PcapMacTx};
use smoltcp::phy::{RxToken, TxToken};

pub const MAC_MAX_LEN: usize = 0x4000;

pub trait RxDevice<'a> {
    type RxToken: RxToken + 'a;
    fn receive(&'a mut self) -> Option<Self::RxToken>;
}

pub trait TxDevice<'a> {
    type TxToken: TxToken + 'a;
    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(self.token())
    }
    fn token(&'a mut self) -> Self::TxToken;
}
