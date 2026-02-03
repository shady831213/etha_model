use super::*;
use pcap_parser::traits::PcapReaderIterator;
use pcap_parser::*;
use smoltcp::phy::{Device, DeviceCapabilities, Medium, PcapLinkType, PcapSink, RxToken, TxToken};
use smoltcp::time::Instant;
use smoltcp::{Error, Result};
use std::fs::File;
use std::io::Write;

const PCAP_MAX_SIZE: usize = 65536;

pub struct PcapMacRxToken<'a>(&'a mut [u8]);

impl<'a> RxToken for PcapMacRxToken<'a> {
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        f(&mut self.0)
    }
}

pub struct PcapMacTxToken<'a, S: PcapSink> {
    buffer: &'a mut [u8],
    sink: &'a mut S,
}

impl<'a, W: Write> TxToken for PcapMacTxToken<'a, W> {
    fn consume<R, F>(self, timestamp: Instant, len: usize, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        let r = f(&mut self.buffer[..len])?;
        self.sink.packet(timestamp, &self.buffer[..len]);
        Ok(r)
    }
}

pub struct PcapMacRx {
    buffer: [u8; MAC_MAX_LEN],
    reader: Box<dyn PcapReaderIterator>,
}

impl PcapMacRx {
    pub fn new(pcap_file: &str) -> Self {
        let file = File::open(pcap_file).expect(&format!("Open {} fail!", pcap_file));
        let mut reader = create_reader(PCAP_MAX_SIZE, file).expect("pcap reader");
        match reader.next() {
            Ok((offset, block)) => {
                if let PcapBlockOwned::LegacyHeader(_) = block {
                    reader.consume(offset);
                } else {
                    panic!("Can not get valid pcap header in {}!", pcap_file)
                }
            }
            Err(e) => panic!("error while reading: {:?}", e),
        };
        PcapMacRx {
            buffer: [0; MAC_MAX_LEN],
            reader,
        }
    }
}

impl<'a> RxDevice<'a> for PcapMacRx {
    type RxToken = PcapMacRxToken<'a>;

    fn receive(&'a mut self) -> Option<Self::RxToken> {
        loop {
            match self.reader.next() {
                Ok((offset, block)) => {
                    if let PcapBlockOwned::Legacy(b) = block {
                        let len = b.data.len();
                        self.buffer[..len].copy_from_slice(&b.data);
                        self.reader.consume(offset);
                        return Some(PcapMacRxToken(&mut self.buffer[..len]));
                    } else {
                        panic!("Invalid Pcap Block!")
                    }
                }
                Err(PcapError::Eof) => return None,
                Err(PcapError::Incomplete) => {
                    self.reader.refill().unwrap();
                }
                Err(e) => {
                    panic!("error while pcap reading: {:?}", e);
                }
            }
        }
    }
}
pub struct PcapMacTx {
    buffer: [u8; MAC_MAX_LEN],
    writer: File,
}

impl PcapMacTx {
    pub fn new(pcap_file: &str) -> Self {
        let mut writer = File::create(pcap_file).expect(&format!("Open {} fail!", pcap_file));
        writer.global_header(PcapLinkType::Ethernet);
        PcapMacTx {
            buffer: [0; MAC_MAX_LEN],
            writer,
        }
    }
}

impl<'a> TxDevice<'a> for PcapMacTx {
    type TxToken = PcapMacTxToken<'a, File>;

    fn token(&'a mut self) -> Self::TxToken {
        PcapMacTxToken {
            buffer: &mut self.buffer[..],
            sink: &mut self.writer,
        }
    }
}

pub struct PcapMac {
    rx: PcapMacRx,
    tx: PcapMacTx,
}

impl PcapMac {
    pub fn new(rx_pcap_file: &str, tx_pcap_file: &str) -> Self {
        PcapMac {
            rx: PcapMacRx::new(rx_pcap_file),
            tx: PcapMacTx::new(tx_pcap_file),
        }
    }
}

impl<'a> Device<'a> for PcapMac {
    type RxToken = <PcapMacRx as RxDevice<'a>>::RxToken;
    type TxToken = <PcapMacTx as TxDevice<'a>>::TxToken;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        self.rx.receive().map(|t| (t, self.tx.token()))
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        self.tx.transmit()
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = MAC_MAX_LEN;
        caps.max_burst_size = Some(1);
        caps.medium = Medium::Ethernet;
        caps
    }
}

unsafe impl Send for PcapMac {}

pub fn pcap_cmp(lhs_file: &str, rhs_file: &str, verbose: bool) -> Result<()> {
    let mut lhs = PcapMacRx::new(lhs_file);
    let mut rhs = PcapMacRx::new(rhs_file);
    let mut cnt = 0;
    if verbose {
        println!("compare lhs:{} with rhs:{}", lhs_file, rhs_file);
    }
    loop {
        if let Some(lhs_rx) = lhs.receive() {
            lhs_rx.consume(Instant::now(), |lhs_b| {
                let rhs_rx = rhs.receive().ok_or_else(|| {
                    println!(
                        "rhs packages less than lhs!\n lhs:{}\n rhs:{}",
                        lhs_file, rhs_file
                    );
                    Error::Illegal
                })?;
                rhs_rx.consume(Instant::now(), |rhs_b| {
                    if verbose {
                        println!(
                            "frame {} lhs_len = {}, rhs_len = {}",
                            cnt,
                            lhs_b.len(),
                            rhs_b.len()
                        );
                    }
                    if lhs_b != rhs_b {
                        println!(
                            "frame {} lhs != rhs !\n lhs:{}\n rhs:{}",
                            cnt, lhs_file, rhs_file
                        );
                        Err(Error::Illegal)
                    } else {
                        cnt += 1;
                        Ok(())
                    }
                })
            })?;
        } else {
            break;
        }
    }
    if rhs.receive().is_some() {
        println!(
            "rhs packages more than lhs!\n lhs:{}\n rhs:{}",
            lhs_file, rhs_file
        );
        Err(Error::Illegal)
    } else {
        if verbose {
            println!("compare lhs:{} with rhs:{} done!", lhs_file, rhs_file);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn pcap_parser_test() {
        let file = File::open("pcaps/20_ecpri_pkts.pcap").unwrap();
        let mut num_blocks = 0;
        let mut reader = create_reader(65536, file).expect("pcap reader");
        loop {
            match reader.next() {
                Ok((offset, block)) => {
                    num_blocks += 1;
                    match block {
                        PcapBlockOwned::Legacy(b) => {
                            println!("got new block");
                            println!("block size: {}", b.data.len());
                        }
                        PcapBlockOwned::LegacyHeader(h) => {
                            println!("got header");
                            println!("{:?}", h);
                        }
                        _ => panic!("Not support PcapNG fmt!"),
                    }
                    reader.consume(offset);
                }
                Err(PcapError::Eof) => break,
                Err(PcapError::Incomplete) => {
                    reader.refill().unwrap();
                }
                Err(e) => panic!("error while reading: {:?}", e),
            }
        }
        println!("num_blocks: {}", num_blocks);
        assert_eq!(num_blocks, 21)
    }

    fn pcap_check_sample_file(file: &str) {
        let mut mac = PcapMacRx::new(file);
        let mut num_blocks = 0;
        loop {
            if let Some(rx) = mac.receive() {
                rx.consume(Instant::now(), |buffer| {
                    println!("block size: {}", buffer.len());
                    num_blocks += 1;
                    Ok(())
                })
                .unwrap();
            } else {
                break;
            }
        }
        assert_eq!(num_blocks, 20)
    }

    #[test]
    fn pcap_mac_rx_test() {
        pcap_check_sample_file("pcaps/20_ecpri_pkts.pcap");
    }

    #[test]
    fn pcap_mac_tx_test() {
        let mut mac = PcapMac::new("pcaps/20_ecpri_pkts.pcap", "pcaps/tmp/test_tx.pcap");
        loop {
            let mut loopback: Vec<u8> = Vec::new();
            if let Some((rx, _)) = mac.receive() {
                rx.consume(Instant::now(), |buffer| {
                    let len = buffer.len();
                    loopback.resize(len, 0);
                    loopback.copy_from_slice(buffer);
                    Ok(())
                })
                .unwrap();
            } else {
                break;
            }
            mac.transmit()
                .unwrap()
                .consume(Instant::now(), loopback.len(), |buffer| {
                    buffer[..loopback.len()].copy_from_slice(&loopback);
                    Ok(())
                })
                .unwrap();
        }
        //check tx
        pcap_check_sample_file("pcaps/tmp/test_tx.pcap");
    }
}
