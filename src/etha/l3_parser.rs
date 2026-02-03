use super::desc::rx::RxResultL3Desc;
use super::l2_parser::L2Parser;
use super::*;
use smoltcp::wire::EthernetProtocol;
use smoltcp::wire::IpAddress;
use smoltcp::wire::IpProtocol;
use smoltcp::wire::Ipv4Address;
use smoltcp::wire::Ipv4Packet;
use smoltcp::wire::Ipv6Packet;
use std::convert::Into;
#[derive(Debug, Copy, Clone)]
pub struct L3Info {
    pub src: IpAddress,
    pub dst: IpAddress,
    pub protocol: IpProtocol,
    pub header_len: usize,
}

impl Default for L3Info {
    fn default() -> Self {
        L3Info {
            src: IpAddress::Ipv4(Ipv4Address::from_bytes(&[0; 4])),
            dst: IpAddress::Ipv4(Ipv4Address::from_bytes(&[0; 4])),
            protocol: IpProtocol::Unknown(0xff),
            header_len: 0,
        }
    }
}

impl Into<RxResultL3Desc> for L3Info {
    fn into(self) -> RxResultL3Desc {
        let mut desc = RxResultL3Desc::default();
        match self.src {
            IpAddress::Ipv4(addr) => {
                desc.set_l3_src(u32::from_le_bytes(addr.as_bytes().try_into().unwrap()));
                desc.set_l3_version(4);
            }
            IpAddress::Ipv6(addr) => {
                desc.set_l3_src(u32::from_le_bytes(
                    addr.as_bytes()[0..4].try_into().unwrap(),
                ));
                desc.set_l3_src1(u32::from_le_bytes(
                    addr.as_bytes()[4..8].try_into().unwrap(),
                ));
                desc.set_l3_src2(u32::from_le_bytes(
                    addr.as_bytes()[8..12].try_into().unwrap(),
                ));
                desc.set_l3_src3(u32::from_le_bytes(
                    addr.as_bytes()[12..16].try_into().unwrap(),
                ));
                desc.set_l3_version(4);
            }
            _ => {}
        }
        match self.dst {
            IpAddress::Ipv4(addr) => {
                desc.set_l3_dst(u32::from_le_bytes(addr.as_bytes().try_into().unwrap()))
            }
            IpAddress::Ipv6(addr) => {
                desc.set_l3_dst(u32::from_le_bytes(
                    addr.as_bytes()[0..4].try_into().unwrap(),
                ));
                desc.set_l3_dst1(u32::from_le_bytes(
                    addr.as_bytes()[4..8].try_into().unwrap(),
                ));
                desc.set_l3_dst2(u32::from_le_bytes(
                    addr.as_bytes()[8..12].try_into().unwrap(),
                ));
                desc.set_l3_dst3(u32::from_le_bytes(
                    addr.as_bytes()[12..16].try_into().unwrap(),
                ));
            }
            _ => {}
        }
        desc.set_l3_protocol(u8::from(self.protocol) as u32);
        desc.set_l3_header_len(self.header_len as u32);
        desc
    }
}

struct Ipv4Parser;

impl Ipv4Parser {
    fn parse(&self, buffer: &[u8]) -> Result<L3Info> {
        let packet = Ipv4Packet::new_checked(buffer)?;
        Ok(L3Info {
            src: IpAddress::Ipv4(packet.src_addr()),
            dst: IpAddress::Ipv4(packet.dst_addr()),
            protocol: packet.protocol(),
            header_len: packet.header_len() as usize,
        })
    }
}

struct Ipv6Parser;

impl Ipv6Parser {
    fn parse(&self, buffer: &[u8]) -> Result<L3Info> {
        let packet = Ipv6Packet::new_checked(buffer)?;
        Ok(L3Info {
            src: IpAddress::Ipv6(packet.src_addr()),
            dst: IpAddress::Ipv6(packet.dst_addr()),
            protocol: packet.next_header(),
            header_len: packet.header_len(),
        })
    }
}

struct UnknownParser;

impl UnknownParser {
    fn parse(&self, _buffer: &[u8]) -> Result<L3Info> {
        Ok(L3Info::default())
    }
}

enum L3ParserInner {
    Ipv4(Ipv4Parser),
    Ipv6(Ipv6Parser),
    Unknown(UnknownParser),
}

impl L3ParserInner {
    fn new(etype: &EthernetProtocol) -> L3ParserInner {
        match etype {
            EthernetProtocol::Ipv4 => L3ParserInner::Ipv4(Ipv4Parser),
            EthernetProtocol::Ipv6 => L3ParserInner::Ipv6(Ipv6Parser),
            _ => L3ParserInner::Unknown(UnknownParser),
        }
    }

    fn parse(&self, buffer: &[u8]) -> Result<L3Info> {
        match self {
            L3ParserInner::Ipv4(parser) => parser.parse(buffer),
            L3ParserInner::Ipv6(parser) => parser.parse(buffer),
            L3ParserInner::Unknown(parser) => parser.parse(buffer),
        }
    }
}

pub struct L3Parser;

impl Pipeline for L3Parser {
    type Input = <L2Parser as Pipeline>::Output;
    type Output = (Self::Input, L3Info);
    fn execute(&mut self, buffer: &mut [u8], i: &Self::Input) -> Result<Self::Output> {
        L3ParserInner::new(&i.etype)
            .parse(&buffer[i.header_len..])
            .map(|r| (*i, r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mac::*;
    use smoltcp::phy::RxToken;
    use smoltcp::time::Instant;

    #[test]
    fn l3_parser_test() {
        let mut mac = PcapMacRx::new("pcaps/vlan.cap");
        loop {
            if let Some(rx) = mac.receive() {
                let (l2_info, l3_info) = rx
                    .consume(Instant::now(), |buffer| {
                        Ok(L2Parser.comb(L3Parser).execute(buffer, &())?)
                    })
                    .unwrap();
                println!("{:#x?}", l3_info);
                match l2_info.etype {
                    EthernetProtocol::Ipv4 | EthernetProtocol::Ipv6 => {
                        assert_ne!(l3_info.header_len, 0);
                        if let IpProtocol::Unknown(_) = l3_info.protocol {
                            assert!(false)
                        }
                    }
                    _ => {
                        assert_eq!(l3_info.header_len, 0);
                        assert_eq!(l3_info.protocol, IpProtocol::Unknown(0xff))
                    }
                }
            } else {
                break;
            }
        }
    }
}
