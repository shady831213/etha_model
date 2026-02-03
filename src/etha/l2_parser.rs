use super::desc::rx::RxResultL2Desc;
use super::*;
use smoltcp::wire::EthernetAddress;
use smoltcp::wire::EthernetFrame;
use smoltcp::wire::EthernetProtocol;
use smoltcp::wire::ETHERNET_HEADER_LEN;
use std::convert::Into;
const VLAN_SIZE: usize = 4;
const VLAN_TYPE: u16 = 0x8100;
#[derive(Debug, Copy, Clone)]
pub struct VlanInfo {
    pub flags: u8,
    pub vid: u16,
}
#[derive(Debug, Copy, Clone)]
pub struct L2Info {
    pub src: EthernetAddress,
    pub dst: EthernetAddress,
    pub etype: EthernetProtocol,
    pub header_len: usize,
    pub vlan: Option<VlanInfo>,
}

impl Into<RxResultL2Desc> for L2Info {
    fn into(self) -> RxResultL2Desc {
        let mut desc = RxResultL2Desc::default();
        desc.set_l2_src_lo(u32::from_le_bytes(
            self.src.as_bytes()[0..4].try_into().unwrap(),
        ));
        desc.set_l2_src_hi(
            u16::from_le_bytes(self.src.as_bytes()[4..6].try_into().unwrap()) as u32,
        );
        desc.set_l2_dst_lo(u32::from_le_bytes(
            self.dst.as_bytes()[0..4].try_into().unwrap(),
        ));
        desc.set_l2_dst_hi(
            u16::from_le_bytes(self.dst.as_bytes()[4..6].try_into().unwrap()) as u32,
        );
        desc.set_l2_etype(u16::from(self.etype) as u32);
        desc.set_l2_header_len(self.header_len as u32);
        if let Some(info) = self.vlan {
            desc.set_l2_is_vlan(1);
            desc.set_l2_vlan_flags(info.flags as u32);
            desc.set_l2_vlan_vid(info.vid as u32);
        }
        desc
    }
}

pub struct L2Parser;

impl L2Parser {
    fn parse(&self, buffer: &[u8]) -> Result<L2Info> {
        let frame = EthernetFrame::new_checked(buffer)?;
        match frame.ethertype() {
            EthernetProtocol::Unknown(p) if p == VLAN_TYPE => self.parse_vlan(buffer),
            _ => Ok(L2Info {
                src: frame.src_addr(),
                dst: frame.dst_addr(),
                etype: frame.ethertype(),
                vlan: None,
                header_len: EthernetFrame::<&[u8]>::header_len(),
            }),
        }
    }

    fn parse_vlan(&self, buffer: &[u8]) -> Result<L2Info> {
        let buffer_novlan = [
            &buffer[..ETHERNET_HEADER_LEN - 2],
            &buffer[ETHERNET_HEADER_LEN + 2..],
        ]
        .concat();
        let frame = EthernetFrame::new_checked(&buffer_novlan)?;
        Ok(L2Info {
            src: frame.src_addr(),
            dst: frame.dst_addr(),
            etype: frame.ethertype(),
            vlan: Some(VlanInfo {
                flags: buffer[ETHERNET_HEADER_LEN] >> 4,
                vid: (buffer[ETHERNET_HEADER_LEN] as u16 & 0xf) << 8
                    | buffer[ETHERNET_HEADER_LEN + 1] as u16,
            }),
            header_len: EthernetFrame::<&[u8]>::header_len() + VLAN_SIZE,
        })
    }
}

impl Pipeline for L2Parser {
    type Input = ();
    type Output = L2Info;
    fn execute(&mut self, buffer: &mut [u8], _i: &Self::Input) -> Result<Self::Output> {
        self.parse(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mac::*;
    use smoltcp::phy::RxToken;
    use smoltcp::time::Instant;

    #[test]
    fn l2_parser_vlan_test() {
        let mut mac = PcapMacRx::new("pcaps/vlan.cap");
        loop {
            if let Some(rx) = mac.receive() {
                let info = rx
                    .consume(Instant::now(), |buffer| {
                        let info = L2Parser.execute(buffer, &())?;
                        let check_frame = EthernetFrame::new_checked(buffer)?;
                        match check_frame.ethertype() {
                            EthernetProtocol::Unknown(p) if p == VLAN_TYPE => {
                                assert!(info.vlan.is_some())
                            }
                            _ => assert!(info.vlan.is_none()),
                        }
                        Ok(info)
                    })
                    .unwrap();
                println!("{:#x?}", info);
            } else {
                break;
            }
        }
    }
}
