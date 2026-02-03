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
pub struct EthaRohcCore<A: Arbiter> {
    arbiter: A,
    chs: [EthaRohcCh; ROHC_CH_NUM],
    engine: RohcEngine,
    abort: Arc<Aborter>,
    irqs: Arc<Mutex<IrqVec>>,
}

impl<A: Arbiter> EthaRohcCore<A> {
    pub fn new(arbiter: A, regs: &TopRegs<ROHC_CH_NUM>) -> Self {
        let irqs = Arc::new(Mutex::new(IrqVec::new("EthaRohcIrqs")));
        let chs =
            array_init::array_init(|i| EthaRohcCh::new(i, &regs.chs[i], &mut irqs.lock().unwrap()));
        EthaRohcCore {
            arbiter,
            chs,
            engine: RohcEngine::new(),
            abort: Arc::new(Aborter::new()),
            irqs,
        }
    }
    pub fn run(&mut self) {
        let mut pipe = EthaIrqs::new(&self.chs, &self.irqs)
            .comb(EthaRohcReqs(&self.chs))
            .comb(EthaRohcArbit(&mut self.arbiter))
            .comb(EthaRohcProcess {
                engine: &self.engine,
                chs: &self.chs,
            });
        loop {
            if self.abort.aborted() {
                break;
            }
            if pipe.execute(&()).is_ok() {
                tracing::debug!(target : "rohc-core", "complete one desc!");
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

pub struct EthaRohcReqs<'a>(&'a [EthaRohcCh]);

impl<'a> Pipeline for EthaRohcReqs<'a> {
    type Input = ();
    type Output = Vec<Option<RohcReqDesc>>;
    fn execute(&mut self, _: &Self::Input) -> PipeResult<Self::Output> {
        Ok(self.0.iter().map(|r| r.req()).collect::<_>())
    }
}

pub struct EthaRohcArbit<'a, A: Arbiter>(&'a mut A);

impl<'a, A: Arbiter> Pipeline for EthaRohcArbit<'a, A> {
    type Input = Vec<Option<RohcReqDesc>>;
    type Output = (usize, RohcReqDesc);
    fn execute(&mut self, i: &Self::Input) -> PipeResult<Self::Output> {
        if let Some(r) = self.0.arbit(i) {
            Ok(r)
        } else {
            Err(PipeError::Dropped)
        }
    }
}

pub struct EthaRohcProcess<'a> {
    engine: &'a RohcEngine,
    chs: &'a [EthaRohcCh],
}

impl<'a> Pipeline for EthaRohcProcess<'a> {
    type Input = (usize, RohcReqDesc);
    type Output = ();
    fn execute(&mut self, i: &Self::Input) -> PipeResult<Self::Output> {
        let span = tracing::span!(
            target: STATICS_TAR,
            logger::STATICS_LEVEL,
            "rohc task",
            ch = i.0
        );
        let _enter = span.enter();
        tracing::event!(
            target: STATICS_TAR,
            logger::STATICS_LEVEL,
            name = "read desc req",
            size = std::mem::size_of::<RohcReqDesc>(),
        );
        EthaRohcExecute(self.engine)
            .comb(EthaRohcResp(self.chs))
            .execute(i)
    }
}

pub struct EthaRohcExecute<'a>(&'a RohcEngine);

impl<'a> Pipeline for EthaRohcExecute<'a> {
    type Input = (usize, RohcReqDesc);
    type Output = (usize, RohcStatusDesc);
    fn execute(&mut self, i: &Self::Input) -> PipeResult<Self::Output> {
        let (id, req) = *i;
        let span = tracing::span!(
            target: STATICS_TAR,
            logger::STATICS_LEVEL,
            "rohc task process",
        );
        let _enter = span.enter();
        Ok((id, self.0.process(req)))
    }
}

pub struct EthaRohcResp<'a>(&'a [EthaRohcCh]);

impl<'a> Pipeline for EthaRohcResp<'a> {
    type Input = (usize, RohcStatusDesc);
    type Output = ();
    fn execute(&mut self, i: &Self::Input) -> PipeResult<Self::Output> {
        let (id, status) = *i;
        self.0[id].resp(&RohcResultDesc { status });
        tracing::event!(
            target: STATICS_TAR,
            logger::STATICS_LEVEL,
            name = "write desc resp",
            size = std::mem::size_of::<RohcStatusDesc>(),
        );
        Ok(())
    }
}
