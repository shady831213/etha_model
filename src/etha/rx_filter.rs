use super::l3_parser::L3Info;
use super::l4_parser::L4Info;
use super::parser::ParserInfo;
use super::reg_if::rx::*;
use super::*;
use smoltcp::wire::*;
use std::sync::Arc;

pub struct EthaRxFilter {
    regs: Arc<FilterRegs<RX_ET_FILTERS, RX_TP5_FILTERS>>,
}

impl EthaRxFilter {
    pub fn new(regs: &Arc<FilterRegs<RX_ET_FILTERS, RX_TP5_FILTERS>>) -> Self {
        EthaRxFilter { regs: regs.clone() }
    }
    pub fn pipeline<'a>(
        &'a self,
    ) -> impl Pipeline<Input = ParserInfo, Output = (ParserInfo, Option<(usize, CongestionAction)>)> + 'a
    {
        EtFilter {
            cfg: &self.regs.et_filters,
        }
        .comb(Tp5Filter {
            cfg: &self.regs.tp5_filters,
        })
    }
}

pub struct EtFilter<'a> {
    cfg: &'a [LockedEtherTypeFilterRegs],
}
impl<'a> EtFilter<'a> {
    fn filter(
        &self,
        filter: &LockedEtherTypeFilterRegs,
        etype: u16,
    ) -> Option<(usize, CongestionAction)> {
        let r_en = filter.et_filter().en();
        let r_etype = filter.et_filter().etype() as u16;
        if r_en == 1 && r_etype as u16 == etype {
            let r_queue_id = filter.et_filter().queue_id() as usize;
            let r_congestion_action = filter.et_filter().get_congestion_action();
            Some((r_queue_id, r_congestion_action))
        } else {
            None
        }
    }
}

impl<'a> Pipeline for EtFilter<'a> {
    type Input = ParserInfo;
    type Output = (ParserInfo, Option<(usize, CongestionAction)>);
    fn execute(&mut self, _: &mut [u8], i: &Self::Input) -> Result<Self::Output> {
        for f in self.cfg.iter() {
            if let Some(r) = self.filter(f, u16::from(i.l2.etype)) {
                return Ok((*i, Some(r)));
            }
        }
        Ok((*i, None))
    }
}

pub struct Tp5Filter<'a> {
    cfg: &'a [LockedTuplesFilterRegs],
}
impl<'a> Tp5Filter<'a> {
    fn filter(
        &self,
        filter: &LockedTuplesFilterRegs,
        info: &ParserInfo,
    ) -> Option<(usize, usize, CongestionAction)> {
        if filter.tp5_ctrl().en() == 1 {
            match info.l2.etype {
                EthernetProtocol::Ipv4 if filter.tp5_ctrl().ipv6() == 0 => {
                    self.ipv4_filter(filter, &info.l3, &info.l4)
                }
                EthernetProtocol::Ipv6 if filter.tp5_ctrl().ipv6() == 1 => {
                    self.ipv6_filter(filter, &info.l3, &info.l4)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn ipv4_filter(
        &self,
        filter: &LockedTuplesFilterRegs,
        l3: &L3Info,
        l4: &L4Info,
    ) -> Option<(usize, usize, CongestionAction)> {
        if self.ipv4_addr_match(
            Ipv4Address::from_bytes(l3.src.as_bytes()),
            filter.tp5_src().get() as u32,
            filter.tp5_ctrl().src_mask() != 0,
        ) && self.ipv4_addr_match(
            Ipv4Address::from_bytes(l3.dst.as_bytes()),
            filter.tp5_dst().get() as u32,
            filter.tp5_ctrl().dst_mask() != 0,
        ) && self.protocal_match(
            l3.protocol,
            filter.tp5_ctrl().protocol() as u8,
            filter.tp5_ctrl().protocol_mask() != 0,
        ) && self.port_match(
            l3.protocol,
            l4.src,
            filter.tp5_port().src() as u16,
            filter.tp5_ctrl().src_port_mask() != 0,
        ) && self.port_match(
            l3.protocol,
            l4.dst,
            filter.tp5_port().dst() as u16,
            filter.tp5_ctrl().dst_port_mask() != 0,
        ) {
            let r_queue_id = filter.tp5_ctrl().queue_id() as usize;
            let r_congestion_action = filter.tp5_ctrl().get_congestion_action();
            let r_pri = filter.tp5_ctrl().pri() as usize;
            Some((r_pri, r_queue_id, r_congestion_action))
        } else {
            None
        }
    }

    fn ipv6_filter(
        &self,
        filter: &LockedTuplesFilterRegs,
        l3: &L3Info,
        l4: &L4Info,
    ) -> Option<(usize, usize, CongestionAction)> {
        if self.ipv6_addr_match(
            Ipv6Address::from_bytes(l3.src.as_bytes()),
            &[
                filter.tp5_src().get() as u32,
                filter.tp5_v6_src_1().get() as u32,
                filter.tp5_v6_src_2().get() as u32,
                filter.tp5_v6_src_3().get() as u32,
            ],
            filter.tp5_ctrl().src_mask() != 0,
        ) && self.ipv6_addr_match(
            Ipv6Address::from_bytes(l3.dst.as_bytes()),
            &[
                filter.tp5_dst().get() as u32,
                filter.tp5_v6_dst_1().get() as u32,
                filter.tp5_v6_dst_2().get() as u32,
                filter.tp5_v6_dst_3().get() as u32,
            ],
            filter.tp5_ctrl().dst_mask() != 0,
        ) && self.protocal_match(
            l3.protocol,
            filter.tp5_ctrl().protocol() as u8,
            filter.tp5_ctrl().protocol_mask() != 0,
        ) && self.port_match(
            l3.protocol,
            l4.src,
            filter.tp5_port().src() as u16,
            filter.tp5_ctrl().src_port_mask() != 0,
        ) && self.port_match(
            l3.protocol,
            l4.dst,
            filter.tp5_port().dst() as u16,
            filter.tp5_ctrl().dst_port_mask() != 0,
        ) {
            let r_queue_id = filter.tp5_ctrl().queue_id() as usize;
            let r_congestion_action = filter.tp5_ctrl().get_congestion_action();
            let r_pri = filter.tp5_ctrl().pri() as usize;
            Some((r_pri, r_queue_id, r_congestion_action))
        } else {
            None
        }
    }

    fn ipv4_addr_match(&self, addr: Ipv4Address, filter: u32, mask: bool) -> bool {
        mask || addr == Ipv4Address::from_bytes(&filter.to_le_bytes())
    }

    fn ipv6_addr_match(&self, addr: Ipv6Address, filter: &[u32; 4], mask: bool) -> bool {
        let array: Vec<u8> = filter.iter().flat_map(|v| v.to_le_bytes()).collect();
        mask || addr == Ipv6Address::from_bytes(&array)
    }

    fn protocal_match(&self, protocol: IpProtocol, filter: u8, mask: bool) -> bool {
        mask || u8::from(protocol) == filter
    }

    fn port_match(&self, protocol: IpProtocol, port: u16, filter: u16, mask: bool) -> bool {
        mask || match protocol {
            IpProtocol::Tcp | IpProtocol::Udp => port == filter,
            _ => true,
        }
    }
}

impl<'a> Pipeline for Tp5Filter<'a> {
    type Input = (ParserInfo, Option<(usize, CongestionAction)>);
    type Output = (ParserInfo, Option<(usize, CongestionAction)>);
    fn execute(&mut self, _: &mut [u8], i: &Self::Input) -> Result<Self::Output> {
        let (info, i) = i;
        Ok((
            *info,
            i.or_else(|| {
                if let EthernetProtocol::Unknown(_) = info.l2.etype {
                    return None;
                }
                let mut candidates = self
                    .cfg
                    .iter()
                    .filter_map(|f| self.filter(f, info))
                    .collect::<Vec<_>>();
                if candidates.is_empty() {
                    None
                } else {
                    candidates.sort_by(|a, b| a.0.cmp(&b.0));
                    let (_, queue_id, congestion_action) = candidates[0];
                    Some((queue_id, congestion_action))
                }
            }),
        ))
    }
}
