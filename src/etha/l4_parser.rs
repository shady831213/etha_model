use super::desc::rx::RxResultL4Desc;
use super::l3_parser::L3Parser;
use super::*;
use smoltcp::wire::IpProtocol;
use smoltcp::wire::TcpPacket;
use smoltcp::wire::UdpPacket;
use smoltcp::wire::TCP_HEADER_LEN;
use smoltcp::wire::UDP_HEADER_LEN;
use std::convert::Into;
#[derive(Debug, Copy, Clone, Default)]
pub struct L4Info {
    pub src: u16,
    pub dst: u16,
    pub header_len: usize,
}

impl Into<RxResultL4Desc> for L4Info {
    fn into(self) -> RxResultL4Desc {
        let mut desc = RxResultL4Desc::default();
        desc.set_l4_src_port(self.src as u32);
        desc.set_l4_dst_port(self.dst as u32);
        desc.set_l4_header_len(self.header_len as u32);
        desc
    }
}

struct TcpParser;

impl TcpParser {
    fn parse(&self, buffer: &[u8]) -> Result<L4Info> {
        let packet = TcpPacket::new_checked(buffer)?;
        Ok(L4Info {
            src: packet.src_port(),
            dst: packet.dst_port(),
            header_len: TCP_HEADER_LEN,
        })
    }
}

struct UdpParser;

impl UdpParser {
    fn parse(&self, buffer: &[u8]) -> Result<L4Info> {
        let packet = UdpPacket::new_checked(buffer)?;
        Ok(L4Info {
            src: packet.src_port(),
            dst: packet.dst_port(),
            header_len: UDP_HEADER_LEN,
        })
    }
}

struct UnknownParser;

impl UnknownParser {
    fn parse(&self, _buffer: &[u8]) -> Result<L4Info> {
        Ok(L4Info::default())
    }
}

enum L4ParserInner {
    Tcp(TcpParser),
    Udp(UdpParser),
    Unknown(UnknownParser),
}

impl L4ParserInner {
    fn new(etype: &IpProtocol) -> L4ParserInner {
        match etype {
            IpProtocol::Tcp => L4ParserInner::Tcp(TcpParser),
            IpProtocol::Udp => L4ParserInner::Udp(UdpParser),
            _ => L4ParserInner::Unknown(UnknownParser),
        }
    }

    fn parse(&self, buffer: &[u8]) -> Result<L4Info> {
        match self {
            L4ParserInner::Tcp(parser) => parser.parse(buffer),
            L4ParserInner::Udp(parser) => parser.parse(buffer),
            L4ParserInner::Unknown(parser) => parser.parse(buffer),
        }
    }
}

pub struct L4Parser;

impl Pipeline for L4Parser {
    type Input = <L3Parser as Pipeline>::Output;
    type Output = (Self::Input, L4Info);
    fn execute(&mut self, buffer: &mut [u8], i: &Self::Input) -> Result<Self::Output> {
        let (l2, l3) = i;
        L4ParserInner::new(&l3.protocol)
            .parse(&buffer[l2.header_len + l3.header_len..])
            .map(|r| (*i, r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::etha::l2_parser::*;
    use crate::mac::*;
    use smoltcp::phy::RxToken;
    use smoltcp::time::Instant;

    #[test]
    fn l4_parser_test() {
        let mut mac = PcapMacRx::new("pcaps/vlan.cap");
        loop {
            if let Some(rx) = mac.receive() {
                let (_, l3_info, l4_info) = rx
                    .consume(Instant::now(), |buffer| {
                        let ((l2_info, l3_info), l4_info) = L2Parser
                            .comb(L3Parser)
                            .comb(L4Parser)
                            .execute(buffer, &())?;
                        Ok((l2_info, l3_info, l4_info))
                    })
                    .unwrap();
                println!("{:#x?}", l4_info);
                match l3_info.protocol {
                    IpProtocol::Tcp | IpProtocol::Udp => {
                        assert_ne!(l4_info.header_len, 0);
                    }
                    _ => {
                        assert_eq!(l4_info.header_len, 0);
                        assert_eq!(l4_info.src, 0);
                        assert_eq!(l4_info.dst, 0);
                    }
                }
            } else {
                break;
            }
        }
    }
}
