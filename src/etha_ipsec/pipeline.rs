use crate::irq::*;
use std::sync::{Arc, Mutex};
#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Blocking,
    Dropped,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Pipeline: Sized {
    type Input;
    type Output;
    fn execute(&mut self, i: &Self::Input) -> Result<Self::Output>;
    fn comb<B: Pipeline<Input = Self::Output>>(self, b: B) -> PipelineComb<Self, B> {
        PipelineComb {
            a: self,
            b,
            blocking_a: None,
        }
    }
}

pub struct PipelineComb<A, B>
where
    A: Pipeline,
    B: Pipeline<Input = A::Output>,
{
    a: A,
    b: B,
    blocking_a: Option<A::Output>,
}
impl<A, B> Pipeline for PipelineComb<A, B>
where
    A: Pipeline,
    B: Pipeline<Input = A::Output>,
{
    type Input = <A as Pipeline>::Input;
    type Output = <B as Pipeline>::Output;
    fn execute(&mut self, i: &Self::Input) -> Result<Self::Output> {
        let a = match self.blocking_a.take() { Some(a) => {
            a
        } _ => {
            self.a.execute(i)?
        }};
        match self.b.execute(&a) {
            Err(Error::Blocking) => {
                self.blocking_a = Some(a);
                Err(Error::Blocking)
            }
            Ok(r) => Ok(r),
            Err(e) => Err(e),
        }
    }
}

pub struct EthaIrqs<'a, R: WithIrq> {
    chs: &'a [R],
    irqs: &'a Arc<Mutex<IrqVec>>,
}
impl<'a, R: WithIrq> EthaIrqs<'a, R> {
    pub fn new(chs: &'a [R], irqs: &'a Arc<Mutex<IrqVec>>) -> Self {
        EthaIrqs { chs, irqs }
    }
}

impl<'a, R: WithIrq> Pipeline for EthaIrqs<'a, R> {
    type Input = ();
    type Output = ();
    fn execute(&mut self, _: &Self::Input) -> Result<Self::Output> {
        let irqs = self.irqs.lock().unwrap();
        for irq in self.chs.iter().filter_map(|ch| ch.poll_irq()) {
            irqs.send(irq)
        }
        Ok(())
    }
}
