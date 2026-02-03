use super::l2_parser::*;
use super::l3_parser::*;
use super::l4_parser::*;
use super::Result;
use super::*;

#[derive(Debug, Copy, Clone)]
pub struct ParserInfo {
    pub l2: L2Info,
    pub l3: L3Info,
    pub l4: L4Info,
}

pub struct EthaRxParser;
impl EthaRxParser {
    pub fn pipeline<'a>(&'a self) -> impl Pipeline<Input = (), Output = ParserInfo> + 'a {
        L2Parser.comb(L3Parser).comb(L4Parser).comb(EthaRxParserFmt)
    }
}

pub struct EthaRxParserFmt;

impl Pipeline for EthaRxParserFmt {
    type Input = <L4Parser as Pipeline>::Output;
    type Output = ParserInfo;
    fn execute(&mut self, _: &mut [u8], i: &Self::Input) -> Result<Self::Output> {
        let ((l2, l3), l4) = *i;
        Ok(ParserInfo { l2, l3, l4 })
    }
}
