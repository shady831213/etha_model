use super::reg_if::{global::LockedEthaGlobalRegs, TopRegs};
use super::rx_datapath::EthaRxDataPath;
use super::tx_datapath::EthaTxDataPath;
use super::tx_sequencer::TxLoadInfo;
use super::*;
use crate::aborter::*;
use crate::arbiter::*;
use crate::irq::*;
use smoltcp::phy::{Device, RxToken, TxToken};
use smoltcp::time::Instant;
use std::sync::{Arc, Mutex};

pub struct EthaCore<A: Arbiter, M: for<'a> Device<'a>> {
    tx: EthaTxDataPath<A>,
    tx_frame: Option<Vec<u8>>,
    rx: EthaRxDataPath,
    rx_frame: Option<Vec<u8>>,
    mac: M,
    regs: Arc<LockedEthaGlobalRegs>,
    abort: Arc<Aborter>,
    irqs: Arc<Mutex<IrqVec>>,
}

impl<A: Arbiter, M: for<'a> Device<'a>> EthaCore<A, M> {
    pub fn new(
        arbiter: A,
        mac: M,
        regs: &Arc<TopRegs<CHS, RX_ET_FILTERS, RX_TP5_FILTERS>>,
    ) -> Self {
        let irqs = Arc::new(Mutex::new(IrqVec::new("EthaIrqs")));
        let rx = EthaRxDataPath::new(regs, &irqs);
        let tx = EthaTxDataPath::new(arbiter, regs, &irqs);
        EthaCore {
            tx,
            tx_frame: None,
            rx,
            rx_frame: None,
            mac,
            regs: regs.global.clone(),
            abort: Arc::new(Aborter::new()),
            irqs,
        }
    }

    fn tx_update_frame(mac: &mut M, frame: &[u8]) -> Option<()> {
        mac.transmit().take().map_or(Some(()), |token| {
            token
                .consume(Instant::now(), frame.len(), |buffer| {
                    buffer[..frame.len()].copy_from_slice(&frame);
                    Ok(())
                })
                .unwrap();
            None
        })
    }

    fn tx<P: Pipeline<Input = (), Output = TxLoadInfo>>(
        pipe: &mut P,
        mac: &mut M,
        frame: Option<Vec<u8>>,
    ) -> Option<Vec<u8>> {
        frame
            .or_else(|| {
                let mut f = vec![0u8; crate::mac::MAC_MAX_LEN];
                let r = pipe.execute(&mut f, &());
                match r {
                    Ok(info) => {
                        f.truncate(info.len);
                        Some(f)
                    }
                    Err(Error::ParseError(e)) => panic!("{:?}", e),
                    _ => None,
                }
            })
            .map(|f| Self::tx_update_frame(mac, &f).map(|_| f))
            .flatten()
    }

    fn rx_update_frame(mac: &mut M) -> Option<Vec<u8>> {
        mac.receive().take().map(|(token, _)| {
            let mut frame = vec![0u8; crate::mac::MAC_MAX_LEN];
            token
                .consume(Instant::now(), |buffer| {
                    tracing::debug!(target : "core", "rx_update_frame!");
                    frame[..buffer.len()].copy_from_slice(&buffer[..]);
                    frame.truncate(buffer.len());
                    Ok(())
                })
                .unwrap();
            frame
        })
    }

    fn rx<P: Pipeline<Input = (), Output = ()>>(
        pipe: &mut P,
        mac: &mut M,
        frame: Option<Vec<u8>>,
    ) -> Option<Vec<u8>> {
        frame
            .or_else(|| Self::rx_update_frame(mac))
            .map(|mut f| {
                tracing::debug!(target : "core", "rx received!");
                let r = pipe.execute(&mut f, &());
                tracing::debug!(target : "core", "rx pipe {} bytes, result {:?}", f.len(), r);
                match r {
                    Ok(_) | Err(Error::Dropped) => Self::rx_update_frame(mac),
                    Err(Error::Blocking) => Some(f),
                    Err(e) => panic!("{:?}", e),
                }
            })
            .flatten()
    }

    pub fn run(&mut self) {
        let mut tx_pipe = self.tx.pipeline();
        let mut rx_pipe = self.rx.pipeline();
        loop {
            if self.abort.aborted() {
                break;
            }
            if self.regs.tx_en().en() == 1 {
                self.tx_frame = Self::tx(&mut tx_pipe, &mut self.mac, self.tx_frame.take());
            }
            if self.regs.rx_en().en() == 1 {
                self.rx_frame = Self::rx(&mut rx_pipe, &mut self.mac, self.rx_frame.take());
            }
        }
    }

    pub fn abort(&self) -> Arc<Aborter> {
        self.abort.clone()
    }
    pub fn irqs(&self) -> Arc<Mutex<IrqVec>> {
        self.irqs.clone()
    }
}
