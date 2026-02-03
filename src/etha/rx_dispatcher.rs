use super::parser::ParserInfo;
use super::reg_if::rx::*;
use super::reg_if::TopRegs;
use super::rx_ch::EthaRxCh;
use super::STATICS_TAR;
use super::*;
use crate::irq::*;
use crate::logger;
use std::sync::{Arc, Mutex};
pub struct EthaRxDispatcher {
    irqs: Arc<Mutex<IrqVec>>,
    chs: [EthaRxCh; CHS],
    default_q: Arc<LockedDefaultFilterRegs>,
}

impl EthaRxDispatcher {
    pub fn new(
        regs: &TopRegs<CHS, RX_ET_FILTERS, RX_TP5_FILTERS>,
        irqs: &Arc<Mutex<IrqVec>>,
    ) -> Self {
        let chs = array_init::array_init(|i| {
            EthaRxCh::new(i, &regs.chs[i].rx, &mut irqs.lock().unwrap())
        });
        EthaRxDispatcher {
            irqs: irqs.clone(),
            chs,
            default_q: regs.rx.default_q.clone(),
        }
    }

    pub fn pipeline<'a>(
        &'a self,
    ) -> impl Pipeline<Input = (ParserInfo, Option<(usize, CongestionAction)>), Output = ()> + 'a
    {
        EthaRxDispatcherPipe {
            chs: &self.chs,
            default_q: &self.default_q,
        }
        .comb(EthaIrqs::new(&self.chs, &self.irqs))
    }
}

pub struct EthaRxDispatcherPipe<'a> {
    chs: &'a [EthaRxCh],
    default_q: &'a Arc<LockedDefaultFilterRegs>,
}

impl<'a> EthaRxDispatcherPipe<'a> {
    fn get_default_q(&self) -> Option<(usize, CongestionAction)> {
        if self.default_q.default_q().en() == 1 {
            let id = self.default_q.default_q().queue_id() as usize;
            if id < CHS {
                Some((id, self.default_q.default_q().get_congestion_action()))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn send_to_default_q(&self, info: ParserInfo, data: &[u8]) -> Result<()> {
        if let Some((id, action)) = self.get_default_q() {
            self.chs[id].write(info, data).ok_or(match action {
                CongestionAction::Drop => Error::Dropped,
                CongestionAction::Blocking => Error::Blocking,
                _ => unreachable!("Invalid default queue congestion action!"),
            })
        } else {
            Err(Error::Dropped)
        }
    }
}

impl<'a> Pipeline for EthaRxDispatcherPipe<'a> {
    type Input = (ParserInfo, Option<(usize, CongestionAction)>);
    type Output = ();
    fn execute(&mut self, buffer: &mut [u8], i: &Self::Input) -> Result<Self::Output> {
        let (info, filter_result) = *i;
        let span = tracing::span!(
            target: STATICS_TAR,
            logger::STATICS_LEVEL,
            "rx packet arrived",
        );
        let _enter = span.enter();
        tracing::debug!(target : "rx_dspatcher", "fitler result: {:?}", filter_result);
        let r = if let Some((id, action)) = filter_result {
            if id < CHS {
                if self.chs[id].write(info, buffer).is_none() {
                    match action {
                        CongestionAction::Drop => Err(Error::Dropped),
                        CongestionAction::Blocking => Err(Error::Blocking),
                        CongestionAction::Default => self.send_to_default_q(info, buffer),
                        _ => unreachable!("Invalid filter congestion action!"),
                    }
                } else {
                    Ok(())
                }
            } else {
                self.send_to_default_q(info, buffer)
            }
        } else {
            self.send_to_default_q(info, buffer)
        };
        if let Err(e) = &r {
            tracing::event!(
                target: STATICS_TAR,
                logger::STATICS_LEVEL,
                name = "rx packet congestion",
                action = ?e
            );
        }
        r
    }
}
