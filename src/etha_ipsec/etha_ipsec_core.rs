use super::ch::*;
use super::desc::req::*;
use super::desc::resp::*;
use super::engine::*;
use super::reg_if::TopRegs;
use super::STATICS_TAR;
use super::*;
use crate::aborter::*;
use crate::arbiter::*;
use crate::irq::*;
use crate::logger;
use std::sync::{Arc, Mutex};
pub struct EthaIpsecCore<A: Arbiter> {
    arbiter: A,
    chs: [EthaIpsecCh; IPSEC_CH_NUM],
    engine: IpsecEngine,
    abort: Arc<Aborter>,
    irqs: Arc<Mutex<IrqVec>>,
}

impl<A: Arbiter> EthaIpsecCore<A> {
    pub fn new(arbiter: A, regs: &TopRegs<IPSEC_CH_NUM, IPSEC_SESSION_NUM>) -> Self {
        let irqs = Arc::new(Mutex::new(IrqVec::new("EthaIpsecIrqs")));
        let chs = array_init::array_init(|i| {
            EthaIpsecCh::new(i, &regs.chs[i], &mut irqs.lock().unwrap())
        });
        EthaIpsecCore {
            arbiter,
            chs,
            engine: IpsecEngine::new(&regs.sessions),
            abort: Arc::new(Aborter::new()),
            irqs,
        }
    }
    pub fn run(&mut self) {
        let mut pipe = EthaIrqs::new(&self.chs, &self.irqs)
            .comb(EthaIpsecReqs(&self.chs))
            .comb(EthaIpsecArbit(&mut self.arbiter))
            .comb(EthaIpsecProcess {
                engine: &self.engine,
                chs: &self.chs,
            });
        loop {
            if self.abort.aborted() {
                break;
            }
            if pipe.execute(&()).is_ok() {
                tracing::debug!(target : "ipsec-core", "complete one desc!");
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

pub struct EthaIpsecReqs<'a>(&'a [EthaIpsecCh]);

impl<'a> Pipeline for EthaIpsecReqs<'a> {
    type Input = ();
    type Output = Vec<Option<IpsecReqDesc>>;
    fn execute(&mut self, _: &Self::Input) -> PipeResult<Self::Output> {
        Ok(self.0.iter().map(|r| r.req()).collect::<_>())
    }
}

pub struct EthaIpsecArbit<'a, A: Arbiter>(&'a mut A);

impl<'a, A: Arbiter> Pipeline for EthaIpsecArbit<'a, A> {
    type Input = Vec<Option<IpsecReqDesc>>;
    type Output = (usize, IpsecReqDesc);
    fn execute(&mut self, i: &Self::Input) -> PipeResult<Self::Output> {
        if let Some(r) = self.0.arbit(i) {
            Ok(r)
        } else {
            Err(PipeError::Dropped)
        }
    }
}

pub struct EthaIpsecProcess<'a> {
    engine: &'a IpsecEngine,
    chs: &'a [EthaIpsecCh],
}

impl<'a> Pipeline for EthaIpsecProcess<'a> {
    type Input = (usize, IpsecReqDesc);
    type Output = ();
    fn execute(&mut self, i: &Self::Input) -> PipeResult<Self::Output> {
        let span = tracing::span!(
            target: STATICS_TAR,
            logger::STATICS_LEVEL,
            "ipsec task",
            ch = i.0
        );
        let _enter = span.enter();
        tracing::event!(
            target: STATICS_TAR,
            logger::STATICS_LEVEL,
            name = "read desc req",
            size = std::mem::size_of::<IpsecReqDesc>(),
        );
        EthaIpsecExecute(self.engine)
            .comb(EthaIpsecResp(self.chs))
            .execute(i)
    }
}

pub struct EthaIpsecExecute<'a>(&'a IpsecEngine);

impl<'a> Pipeline for EthaIpsecExecute<'a> {
    type Input = (usize, IpsecReqDesc);
    type Output = (usize, IpsecStatusDesc);
    fn execute(&mut self, i: &Self::Input) -> PipeResult<Self::Output> {
        let (id, req) = *i;
        let span = tracing::span!(
            target: STATICS_TAR,
            logger::STATICS_LEVEL,
            "ipsec task process",
        );
        let _enter = span.enter();
        Ok((id, self.0.process(req)))
    }
}

pub struct EthaIpsecResp<'a>(&'a [EthaIpsecCh]);

impl<'a> Pipeline for EthaIpsecResp<'a> {
    type Input = (usize, IpsecStatusDesc);
    type Output = ();
    fn execute(&mut self, i: &Self::Input) -> PipeResult<Self::Output> {
        let (id, status) = *i;
        self.0[id].resp(&IpsecResultDesc { status });
        tracing::event!(
            target: STATICS_TAR,
            logger::STATICS_LEVEL,
            name = "write desc resp",
            size = std::mem::size_of::<IpsecStatusDesc>(),
        );
        Ok(())
    }
}
