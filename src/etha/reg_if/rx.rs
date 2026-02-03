use crate::reg_if::RegBus;
use etha_model_generator::*;
use std::sync::Arc;
pub const fn set_filter_queue_id(id: usize) -> usize {
    (id & 0xff) << 16
}

pub const fn filter_queue_id(v: usize) -> usize {
    (v >> 16) & 0xff
}

pub const fn set_filter_congestion_action(action: CongestionAction) -> usize {
    (action as u8 as usize & 0x3) << 29
}

pub const fn filter_congestion_action(v: usize) -> CongestionAction {
    match v {
        0 => CongestionAction::Blocking,
        1 => CongestionAction::Drop,
        2 => CongestionAction::Default,
        _ => CongestionAction::Unknown,
    }
}

pub const fn set_filter_en(en: bool) -> usize {
    (en as usize) << 31
}

pub const fn filter_en(v: usize) -> bool {
    (v >> 31) != 0
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum CongestionAction {
    Blocking = 0,
    Drop = 1,
    Default = 2,
    Unknown = 3,
}

impl std::convert::From<u8> for CongestionAction {
    fn from(value: u8) -> Self {
        match value {
            0 => CongestionAction::Blocking,
            1 => CongestionAction::Drop,
            2 => CongestionAction::Default,
            _ => CongestionAction::Unknown,
        }
    }
}

pub const fn set_filter_etype(etype: u16) -> usize {
    etype as usize
}

pub const fn filter_etype(v: usize) -> u16 {
    v as u16
}

define_reg! {
    EtherTypeFilter {
        fields {
            etype(RW): 15, 0;
            queue_id(RW): 23, 16;
            congestion_action(RW){blocking:0, drop:1, default:2}:30, 29;
            en(RW): 31, 31;
        }
    }
}

impl LockedEtherTypeFilter {
    pub fn get_congestion_action(&self) -> CongestionAction {
        match self.congestion_action() {
            1 => CongestionAction::Drop,
            2 => CongestionAction::Default,
            _ => CongestionAction::Blocking,
        }
    }
}

define_reg! {
    TuplesFilterAddress {
        fields {
            addr(RW): 31, 0;
        }
    }
}

define_reg! {
    TuplesFilterPort {
        fields {
            src(RW): 15, 0;
            dst(RW): 31, 16;
        }
    }
}

define_reg! {
    TuplesFilterCtrl {
        fields {
            protocol(RW): 7, 0;
            pri(RW): 10, 8;
            ipv6(RW): 11, 11;
            queue_id(RW): 23, 16;
            src_mask(RW):24, 24;
            dst_mask(RW):25, 25;
            protocol_mask(RW):26, 26;
            src_port_mask(RW):27, 27;
            dst_port_mask(RW):28, 28;
            congestion_action(RW){blocking:0, drop:1, default:2}:30, 29;
            en(RW): 31, 31;
        }
    }
}

impl LockedTuplesFilterCtrl {
    pub fn get_congestion_action(&self) -> CongestionAction {
        match self.congestion_action() {
            1 => CongestionAction::Drop,
            2 => CongestionAction::Default,
            _ => CongestionAction::Blocking,
        }
    }
}

define_reg! {
    DefaultQueue {
        fields {
            queue_id(RW): 23, 16;
            congestion_action(RW){blocking:0, drop:1}:29, 29;
            en(RW): 31, 31;
        }
    }
}

impl LockedDefaultQueue {
    pub fn get_congestion_action(&self) -> CongestionAction {
        match self.congestion_action() {
            1 => CongestionAction::Drop,
            _ => CongestionAction::Blocking,
        }
    }
}

reg_map! {
    pub EtherTypeFilterRegs(1) {
        et_filter(RW): EtherTypeFilter, 0;
    }
}

pub const TP5_FILETER_REGS_SIZE: usize = 0x10;

reg_map! {
    pub TuplesFilterRegs(0x10) {
        tp5_src(RW): TuplesFilterAddress, 0;
        tp5_v6_src_1(RW): TuplesFilterAddress, 1;
        tp5_v6_src_2(RW): TuplesFilterAddress, 2;
        tp5_v6_src_3(RW): TuplesFilterAddress, 3;
        tp5_dst(RW): TuplesFilterAddress, 4;
        tp5_v6_dst_1(RW): TuplesFilterAddress, 5;
        tp5_v6_dst_2(RW): TuplesFilterAddress, 6;
        tp5_v6_dst_3(RW): TuplesFilterAddress, 7;
        tp5_port(RW): TuplesFilterPort, 8;
        tp5_ctrl(RW): TuplesFilterCtrl, 9;
    }
}

reg_map! {
    pub DefaultFilterRegs(1) {
        default_q(RW): DefaultQueue, 0;
    }
}

pub const ET_FILTER_APERTURE: usize = 32;
pub const TP5_FILTER_APERTURE: usize = TP5_FILETER_REGS_SIZE * 32;

pub struct FilterRegs<const ET_FILTER_N: usize, const TP5_FILTER_N: usize> {
    pub et_filters: [LockedEtherTypeFilterRegs; ET_FILTER_N],
    pub tp5_filters: [LockedTuplesFilterRegs; TP5_FILTER_N],
}

impl<const ET_FILTER_N: usize, const TP5_FILTER_N: usize> FilterRegs<ET_FILTER_N, TP5_FILTER_N> {
    pub fn new() -> Self {
        FilterRegs {
            et_filters: array_init::array_init(|_| LockedEtherTypeFilterRegs::new(32)),
            tp5_filters: array_init::array_init(|_| LockedTuplesFilterRegs::new(32)),
        }
    }
    const TP5_FILETER_RANGE: std::ops::Range<u64> =
        0..((TP5_FILTER_N * TP5_FILETER_REGS_SIZE) as u64);
    const ET_FILETER_RANGE: std::ops::Range<u64> =
        (TP5_FILTER_APERTURE as u64)..((TP5_FILTER_APERTURE + ET_FILTER_N) as u64);
}

impl<const ET_FILTER_N: usize, const TP5_FILTER_N: usize> RegBus
    for FilterRegs<ET_FILTER_N, TP5_FILTER_N>
{
    fn write(&self, addr: u64, data: u64) -> Option<()> {
        if Self::ET_FILETER_RANGE.contains(&addr) {
            let offset = (addr - Self::ET_FILETER_RANGE.start) as usize;
            self.et_filters[offset].write(0, data)
        } else if Self::TP5_FILETER_RANGE.contains(&addr) {
            let offset = (addr - Self::TP5_FILETER_RANGE.start) as usize;
            self.et_filters[offset / TP5_FILETER_REGS_SIZE]
                .write((offset % TP5_FILETER_REGS_SIZE) as u64, data)
        } else {
            None
        }
    }

    fn read(&self, addr: u64) -> Option<u64> {
        if Self::ET_FILETER_RANGE.contains(&addr) {
            let offset = (addr - Self::ET_FILETER_RANGE.start) as usize;
            self.et_filters[offset].read(0)
        } else if Self::TP5_FILETER_RANGE.contains(&addr) {
            let offset = (addr - Self::TP5_FILETER_RANGE.start) as usize;
            self.et_filters[offset / TP5_FILETER_REGS_SIZE]
                .read((offset % TP5_FILETER_REGS_SIZE) as u64)
        } else {
            None
        }
    }
}

impl<const ET_FILTER_N: usize, const TP5_FILTER_N: usize> GenHeader
    for FilterRegs<ET_FILTER_N, TP5_FILTER_N>
{
    fn render_name() -> &'static str {
        "FilterRegs"
    }
    fn gen_c_header<W: std::io::Write>(header: &mut W) -> std::io::Result<()> {
        EtherTypeFilterRegs::gen_c_header(header)?;
        EtherTypeFilter::gen_c_header(header)?;
        TuplesFilterRegs::gen_c_header(header)?;
        TuplesFilterPort::gen_c_header(header)?;
        TuplesFilterCtrl::gen_c_header(header)?;

        writeln!(header, "#define TP5_FILTERS_NUM {}", TP5_FILTER_N)?;

        writeln!(
            header,
            "#define TP5_FILTERS_OFFSET {:#x}",
            Self::TP5_FILETER_RANGE.start
        )?;

        writeln!(header, "#define ET_FILTERS_NUM {}", ET_FILTER_N)?;

        writeln!(
            header,
            "#define ET_FILTERS_OFFSET {:#x}",
            Self::ET_FILETER_RANGE.start
        )?;
        writeln!(
            header,
            "#define TP5_FILTER_OFFSET(base, name, i) ((base) + TP5_FILTERS_OFFSET + (TUPLES_FILETER_REGS_SIZE * i) + TUPLES_FILTER_REGS_##name##_OFFSET)",
        )?;
        writeln!(
            header,
            "#define ET_FILTER_OFFSET(base, i) ((base) + ET_FILTERS_OFFSET + (ETHER_TYPE_FILTER_REGS_SIZE * i))",
        )?;
        Ok(())
    }
}

pub struct RxRegs<const ET_FILTER_N: usize, const TP5_FILTER_N: usize> {
    pub filters: Arc<FilterRegs<ET_FILTER_N, TP5_FILTER_N>>,
    pub default_q: Arc<LockedDefaultFilterRegs>,
}

impl<const ET_FILTER_N: usize, const TP5_FILTER_N: usize> RxRegs<ET_FILTER_N, TP5_FILTER_N> {
    pub fn new() -> Self {
        RxRegs {
            filters: Arc::new(FilterRegs::new()),
            default_q: Arc::new(LockedDefaultFilterRegs::new(32)),
        }
    }
    const DEFAULT_Q_RANGE: std::ops::Range<u64> =
        FilterRegs::<ET_FILTER_N, TP5_FILTER_N>::ET_FILETER_RANGE.end
            ..(FilterRegs::<ET_FILTER_N, TP5_FILTER_N>::ET_FILETER_RANGE.end + 1);

    pub const fn default_filter_offset(&self) -> usize {
        Self::DEFAULT_Q_RANGE.start as usize
    }

    pub const fn et_filter_offset(&self, i: usize) -> usize {
        FilterRegs::<ET_FILTER_N, TP5_FILTER_N>::ET_FILETER_RANGE.start as usize + i
    }
}

impl<const ET_FILTER_N: usize, const TP5_FILTER_N: usize> RegBus
    for RxRegs<ET_FILTER_N, TP5_FILTER_N>
{
    fn write(&self, addr: u64, data: u64) -> Option<()> {
        if Self::DEFAULT_Q_RANGE.contains(&addr) {
            self.default_q.write(0, data)
        } else {
            self.filters.write(addr, data)
        }
    }

    fn read(&self, addr: u64) -> Option<u64> {
        if Self::DEFAULT_Q_RANGE.contains(&addr) {
            self.default_q.read(0)
        } else {
            self.filters.read(addr)
        }
    }
}

impl<const ET_FILTER_N: usize, const TP5_FILTER_N: usize> GenHeader
    for RxRegs<ET_FILTER_N, TP5_FILTER_N>
{
    fn render_name() -> &'static str {
        "RxRegs"
    }
    fn gen_c_header<W: std::io::Write>(header: &mut W) -> std::io::Result<()> {
        FilterRegs::<ET_FILTER_N, TP5_FILTER_N>::gen_c_header(header)?;
        DefaultFilterRegs::gen_c_header(header)?;
        DefaultQueue::gen_c_header(header)?;

        writeln!(
            header,
            "#define DEFAULT_Q_OFFSET(base) ((base) + {:#x})",
            Self::DEFAULT_Q_RANGE.start
        )?;
        Ok(())
    }
}
