use super::desc::tx::*;
use super::reg_if::TopRegs;
use super::tx_ch::*;
use super::Error;
use super::STATICS_TAR;
use super::*;
use crate::arbiter::*;
use crate::irq::*;
use crate::logger;
use std::sync::{Arc, Mutex};
#[derive(Debug, Copy, Clone, Default)]
pub struct TxLoadInfo {
    pub too_large: bool,
    pub too_small: bool,
    pub resp_en: bool,
    pub len: usize,
    pub ch_id: usize,
}

impl TxLoadInfo {
    pub fn dropped(&self) -> bool {
        self.too_large | self.too_small
    }
}

pub struct EthaTxSequencer<A: Arbiter> {
    irqs: Arc<Mutex<IrqVec>>,
    pub arbiter: A,
    pub chs: [EthaTxCh; CHS],
}
impl<A: Arbiter> EthaTxSequencer<A> {
    pub fn new(
        arbiter: A,
        regs: &TopRegs<CHS, RX_ET_FILTERS, RX_TP5_FILTERS>,
        irqs: &Arc<Mutex<IrqVec>>,
    ) -> Self {
        let chs = array_init::array_init(|i| {
            EthaTxCh::new(i, &regs.chs[i].tx, &mut irqs.lock().unwrap())
        });
        EthaTxSequencer {
            irqs: irqs.clone(),
            arbiter,
            chs,
        }
    }
    pub fn pipeline<'a>(&'a mut self) -> impl Pipeline<Input = (), Output = TxLoadInfo> + 'a {
        EthaIrqs::new(&self.chs, &self.irqs)
            .comb(EthaTxReqs(&self.chs))
            .comb(EthaTxArbit(&mut self.arbiter))
            .comb(EthaTxProcss(&self.chs))
    }
}

pub struct EthaTxReqs<'a>(&'a [EthaTxCh]);

impl<'a> Pipeline for EthaTxReqs<'a> {
    type Input = ();
    type Output = Vec<Option<TxReqDesc>>;
    fn execute(&mut self, _: &mut [u8], _: &Self::Input) -> Result<Self::Output> {
        Ok(self.0.iter().map(|r| r.req()).collect::<_>())
    }
}

pub struct EthaTxArbit<'a, A: Arbiter>(&'a mut A);

impl<'a, A: Arbiter> Pipeline for EthaTxArbit<'a, A> {
    type Input = Vec<Option<TxReqDesc>>;
    type Output = (usize, TxReqDesc);
    fn execute(&mut self, _: &mut [u8], i: &Self::Input) -> Result<Self::Output> {
        if let Some(r) = self.0.arbit(i) {
            Ok(r)
        } else {
            Err(Error::Dropped)
        }
    }
}

pub struct EthaTxProcss<'a>(&'a [EthaTxCh]);
impl<'a> Pipeline for EthaTxProcss<'a> {
    type Input = (usize, TxReqDesc);
    type Output = TxLoadInfo;
    fn execute(&mut self, buffer: &mut [u8], i: &Self::Input) -> Result<Self::Output> {
        let span = tracing::span!(
            target: STATICS_TAR,
            logger::STATICS_LEVEL,
            "tx send packet",
            id = i.0
        );
        let _enter = span.enter();
        let r = EthaTxLoadFrame(self.0)
            .comb(EthaTxStoreResp(self.0))
            .execute(buffer, i);
        match &r {
            Ok(r) => tracing::event!(
                target: STATICS_TAR,
                logger::STATICS_LEVEL,
                name = "tx send done",
                size = r.len
            ),
            Err(e) => tracing::event!(
                target: STATICS_TAR,
                logger::STATICS_LEVEL,
                name = "tx send err",
                action = ?e
            ),
        }
        r
    }
}

pub struct EthaTxLoadFrame<'a>(&'a [EthaTxCh]);

impl<'a> Pipeline for EthaTxLoadFrame<'a> {
    type Input = (usize, TxReqDesc);
    type Output = TxLoadInfo;
    fn execute(&mut self, buffer: &mut [u8], i: &Self::Input) -> Result<Self::Output> {
        let (id, req) = *i;
        let mut info = TxLoadInfo::default();
        info.len = req.frame.total_size() as usize;
        info.too_large = info.len > buffer.len();
        info.too_small = info.len < MIN_FRAME_LEN;
        info.resp_en = req.ctrl.resp_en() == 1;
        info.ch_id = id;
        if !info.dropped() {
            self.0[id].read(buffer);
        }
        Ok(info)
    }
}

pub struct EthaTxStoreResp<'a>(&'a [EthaTxCh]);

impl<'a> Pipeline for EthaTxStoreResp<'a> {
    type Input = TxLoadInfo;
    type Output = TxLoadInfo;
    fn execute(&mut self, _: &mut [u8], i: &Self::Input) -> Result<Self::Output> {
        let resp = if i.resp_en {
            let mut resp = TxResultDesc::default();
            resp.set_too_large(i.too_large as u32);
            resp.set_too_small(i.too_small as u32);
            Some(resp)
        } else {
            None
        };
        self.0[i.ch_id].write_resp(&resp);
        if i.dropped() {
            Err(Error::Dropped)
        } else {
            Ok(*i)
        }
    }
}
