use etha_model_generator::*;
use std::marker::PhantomData;

define_reg! {
RingBase {
    fields {
        addr(RW):31, 0;
    }
}
}

define_reg! {
RingSize {
    fields {
        size(RW):30, 0;
    }
}
}

define_reg! {
RingPtr {
    fields {
        ptr(RW):30, 0;
        round(RW):31, 31;
    }
}
}

define_reg! {
RingStatus {
    fields {
        full(RW):0, 0;
        empty(RW):1, 1;
        almost_full(RW):2, 2;
        almost_empty(RW):3, 3;
    }
}
}

define_reg! {
RingCtrl {
    fields {
        enable(RW):0, 0;
    }
}
}
pub const RING_REGS_SIZE: usize = 0x10;

reg_map! {
    pub RingRegs(0x10) {
        req_base_l(RW): RingBase, 0x0;
        req_base_h(RW): RingBase, 0x1;
        resp_base_l(RW): RingBase, 0x2;
        resp_base_h(RW): RingBase, 0x3;
        size(RW): RingSize, 0x4;
        h_water_mark(RW):RingSize, 0x5;
        l_water_mark(RW):RingSize, 0x6;
        p_consumer(RO): RingPtr, 0x7;
        p_producer(RW): RingPtr, 0x8;
        status(RO):RingStatus, 0x9;
        int_mask(RW):RingStatus, 0xa;
        ctrl(RW):RingCtrl, 0xc;
        mem_size(RW):RingSize, 0xd;
    }
}

impl Ring for LockedRingRegs {
    fn r_size(&self) -> usize {
        self.size().size() as usize
    }
    fn r_req_base(&self) -> u64 {
        self.req_base_l().get() | (self.req_base_h().get() << 32)
    }
    fn r_resp_base(&self) -> Option<u64> {
        let base = self.resp_base_l().get() | (self.resp_base_h().get() << 32);
        if base == 0 {
            None
        } else {
            Some(base)
        }
    }
    fn r_c_ptr(&self) -> usize {
        self.p_consumer().get() as usize
    }
    fn r_p_ptr(&self) -> usize {
        self.p_producer().get() as usize
    }
    fn r_set_c_ptr(&self, v: usize) {
        self.p_consumer_mut().set(v as u64)
    }
    fn r_set_p_ptr(&self, v: usize) {
        self.p_producer_mut().set(v as u64)
    }
    fn r_enabled(&self) -> bool {
        self.ctrl().enable() != 0
    }
    fn r_set_enable(&self, v: bool) {
        self.ctrl_mut().set_enable(v as u64)
    }
    fn r_set_full(&self, v: bool) {
        self.status_mut().set_full(v as u64)
    }
    fn r_set_empty(&self, v: bool) {
        self.status_mut().set_empty(v as u64)
    }
    fn r_h_watermark(&self) -> usize {
        self.h_water_mark().size() as usize
    }
    fn r_l_watermark(&self) -> usize {
        self.l_water_mark().size() as usize
    }
    fn r_set_almost_full(&self, v: bool) {
        self.status_mut().set_almost_full(v as u64)
    }
    fn r_set_almost_empty(&self, v: bool) {
        self.status_mut().set_almost_empty(v as u64)
    }
    fn r_irq_pendings(&self) -> usize {
        let mask = self.int_mask().get() as usize;
        let status = self.status().get() as usize;
        mask & status
    }
}

pub trait Ring {
    fn r_size(&self) -> usize;
    fn r_req_base(&self) -> u64;
    fn r_resp_base(&self) -> Option<u64>;
    fn r_c_ptr(&self) -> usize;
    fn r_p_ptr(&self) -> usize;
    fn r_set_c_ptr(&self, v: usize);
    fn r_set_p_ptr(&self, v: usize);
    fn r_enabled(&self) -> bool;
    fn r_set_enable(&self, v: bool);
    fn r_set_full(&self, v: bool);
    fn r_set_empty(&self, v: bool);
    fn r_l_watermark(&self) -> usize;
    fn r_h_watermark(&self) -> usize;
    fn r_set_almost_full(&self, v: bool);
    fn r_set_almost_empty(&self, v: bool);
    fn r_irq_pendings(&self) -> usize;

    fn r_ptr_round(ptr: usize) -> bool {
        (ptr & 0x80000000) == 0x80000000
    }
    fn r_ptr_l(ptr: usize) -> usize {
        ptr & 0x7fffffff
    }
    fn r_incr_ptr(&self, ptr: usize, n: usize) -> usize {
        let ptr_l = Self::r_ptr_l(ptr);
        if ptr_l + n > self.r_size() - 1 {
            let round = ((!Self::r_ptr_round(ptr)) as usize) << 31;
            let ptr_l = ptr_l + n - self.r_size();
            round | ptr_l
        } else {
            ptr + n
        }
    }
    fn r_decr_ptr(&self, ptr: usize, n: usize) -> usize {
        let ptr_l = Self::r_ptr_l(ptr);
        if ptr_l < n {
            let round = ((!Self::r_ptr_round(ptr)) as usize) << 31;
            let ptr_l = self.r_size() - (n - ptr_l);
            round | ptr_l
        } else {
            ptr - n
        }
    }
    fn r_next_ptr(&self, ptr: usize) -> usize {
        self.r_incr_ptr(ptr, 1)
    }
    fn r_prev_ptr(&self, ptr: usize) -> usize {
        self.r_decr_ptr(ptr, 1)
    }
    fn r_c_ptr_unpack(&self) -> (bool, usize) {
        let ptr = self.r_c_ptr();
        (Self::r_ptr_round(ptr), Self::r_ptr_l(ptr))
    }
    fn r_p_ptr_unpack(&self) -> (bool, usize) {
        let ptr = self.r_p_ptr();
        (Self::r_ptr_round(ptr), Self::r_ptr_l(ptr))
    }
    fn r_enable(&self) -> &Self {
        self.r_set_enable(true);
        self
    }
    fn r_disable(&self) -> &Self {
        self.r_set_enable(false);
        self
    }
    fn r_full(&self) -> bool {
        let (r_c_ptr_round, r_c_ptr_l) = self.r_c_ptr_unpack();
        let (r_p_ptr_round, r_p_ptr_l) = self.r_p_ptr_unpack();
        r_c_ptr_round != r_p_ptr_round
            && r_c_ptr_l == r_p_ptr_l
            && self.r_size() != 0
            && self.r_enabled()
    }
    fn r_empty(&self) -> bool {
        let (r_c_ptr_round, r_c_ptr_l) = self.r_c_ptr_unpack();
        let (r_p_ptr_round, r_p_ptr_l) = self.r_p_ptr_unpack();
        r_c_ptr_round == r_p_ptr_round
            && r_c_ptr_l == r_p_ptr_l
            && self.r_size() != 0
            && self.r_enabled()
    }
    fn r_c_valids(&self) -> usize {
        if self.r_size() == 0 || !self.r_enabled() {
            0
        } else {
            let (r_c_ptr_round, r_c_ptr_l) = self.r_c_ptr_unpack();
            let (r_p_ptr_round, r_p_ptr_l) = self.r_p_ptr_unpack();
            if r_c_ptr_round != r_p_ptr_round {
                if r_c_ptr_l < r_p_ptr_l {
                    tracing::debug!(target: "ring", "r_c_valids: c_ptr:{:#x}, p_ptr:{:#x}", r_c_ptr_l, r_p_ptr_l);
                }
                self.r_size() - (r_c_ptr_l - r_p_ptr_l)
            } else {
                if r_c_ptr_l > r_p_ptr_l {
                    tracing::debug!(target: "ring", "r_c_valids: c_ptr:{:#x}, p_ptr:{:#x}", r_c_ptr_l, r_p_ptr_l);
                }
                r_p_ptr_l - r_c_ptr_l
            }
        }
    }
    fn r_p_valids(&self) -> usize {
        if self.r_size() == 0 || !self.r_enabled() {
            0
        } else {
            self.r_size() - self.r_c_valids()
        }
    }
    fn r_almost_full(&self) -> bool {
        self.r_c_valids() >= self.r_h_watermark()
    }
    fn r_almost_empty(&self) -> bool {
        self.r_c_valids() <= self.r_l_watermark()
    }
    fn r_update_status(&self) {
        self.r_set_full(self.r_full());
        self.r_set_empty(self.r_empty());
        self.r_set_almost_full(self.r_almost_full());
        self.r_set_almost_empty(self.r_almost_empty());
    }
    fn r_advance_p(&self) -> &Self {
        self.r_advance_p_n(1)
    }
    fn r_advance_p_n(&self, n: usize) -> &Self {
        if self.r_enabled() {
            self.r_set_p_ptr(self.r_incr_ptr(self.r_p_ptr(), n));
            self.r_update_status();
        }
        self
    }
    fn r_advance_c(&self) -> &Self {
        self.r_advance_c_n(1)
    }
    fn r_advance_c_n(&self, n: usize) -> &Self {
        if self.r_enabled() {
            self.r_set_c_ptr(self.r_incr_ptr(self.r_c_ptr(), n));
            self.r_update_status();
        }
        self
    }
}

pub trait HwRing {
    type REQ: Sized + Copy;
    type RESP: Sized + Copy;
    const REQ_SIZE: usize = std::mem::size_of::<Self::REQ>();
    const RESP_SIZE: usize = std::mem::size_of::<Self::RESP>();
    type R: Ring;
    fn get_ring(&self) -> &Self::R;

    fn r_get_req_at(&self, ptr: usize) -> *const Self::REQ {
        let r_ptr_l = <Self::R as Ring>::r_ptr_l(ptr);
        let ptr = (self.get_ring().r_req_base() + r_ptr_l as u64 * Self::REQ_SIZE as u64) as usize
            as *const Self::REQ;
        ptr
    }

    fn r_get_resp_at(&self, ptr: usize) -> Option<*mut Self::RESP> {
        let r_ptr_l = <Self::R as Ring>::r_ptr_l(ptr);
        if let Some(base) = self.get_ring().r_resp_base() {
            let ptr = (base + r_ptr_l as u64 * Self::RESP_SIZE as u64) as usize as *mut Self::RESP;
            Some(ptr)
        } else {
            None
        }
    }

    fn r_get_req(&self) -> Self::REQ {
        unsafe { *self.r_get_req_at(self.get_ring().r_c_ptr()) }
    }

    fn r_set_resp(&self, resp: &Self::RESP) -> Option<&Self> {
        self.r_get_resp_at(self.get_ring().r_c_ptr()).map(|ptr| {
            unsafe { *ptr = *resp };
            self
        })
    }

    fn entries<'a>(&'a self) -> HwRingIter<'a, Self> {
        HwRingIter {
            ring: self,
            ptr: self.get_ring().r_c_ptr(),
            end_ptr: self.get_ring().r_p_ptr(),
            _mark: PhantomData,
        }
    }
}

pub struct HwRingIter<'a, R: HwRing + ?Sized> {
    ring: &'a R,
    ptr: usize,
    end_ptr: usize,
    _mark: PhantomData<&'a R>,
}

impl<'a, R: HwRing + ?Sized> Iterator for HwRingIter<'a, R> {
    type Item = (*const <R as HwRing>::REQ, Option<*mut <R as HwRing>::RESP>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end_ptr {
            None
        } else {
            let req = self.ring.r_get_req_at(self.ptr);
            let resp = self.ring.r_get_resp_at(self.ptr);
            self.ptr = self.ring.get_ring().r_next_ptr(self.ptr);
            Some((req, resp))
        }
    }
}

#[cfg(test)]
pub mod sw_ring {
    use super::*;
    use crate::reg_if::RegBus;
    use std::sync::Arc;

    pub const RING_REQ_BASE_L_OFFSET: usize = 0x0;
    pub const RING_REQ_BASE_H_OFFSET: usize = 0x1;
    pub const RING_RESP_BASE_L_OFFSET: usize = 0x2;
    pub const RING_RESP_BASE_H_OFFSET: usize = 0x3;
    pub const RING_SIZE_OFFSET: usize = 0x4;
    pub const RING_H_WM_OFFSET: usize = 0x5;
    pub const RING_L_WM_OFFSET: usize = 0x6;
    pub const RING_PC_OFFSET: usize = 0x7;
    pub const RING_PP_OFFSET: usize = 0x8;
    pub const RING_STATUS_OFFSET: usize = 0x9;
    pub const RING_INTM_OFFSET: usize = 0xa;
    pub const RING_CTRL_OFFSET: usize = 0xc;
    pub const RING_MEM_SIZE_OFFSET: usize = 0xd;

    pub const RING_FULL_FLAG: usize = 0x1;
    pub const RING_EMPTY_FLAG: usize = 0x2;
    // pub const RING_AFULL_FLAG: usize = 0x4;
    pub const RING_AEMPTY_FLAG: usize = 0x8;

    const fn addr(offset: usize) -> u64 {
        offset as u64
    }

    pub struct SwQueue<R: RegBus, REQ: Sized, RESP: Sized> {
        pub regs: Arc<R>,
        pub base: usize,
        pub ring_size: usize,
        req_ring: Vec<REQ>,
        resp_ring: Vec<RESP>,
    }

    impl<R: RegBus, REQ: Sized, RESP: Sized> SwQueue<R, REQ, RESP> {
        pub fn new(
            regs: &Arc<R>,
            base: usize,
            ring_size: usize,
            req_ring: Vec<REQ>,
            resp_ring: Vec<RESP>,
        ) -> Self {
            SwQueue {
                regs: regs.clone(),
                base,
                ring_size,
                req_ring,
                resp_ring,
            }
        }
        pub fn init(&self) {
            let req_ring_base = self.req_ring.as_ptr() as usize as u64;
            self.regs
                .write(
                    addr(self.base + RING_REQ_BASE_L_OFFSET),
                    req_ring_base as u32 as u64,
                )
                .unwrap();
            self.regs
                .write(
                    addr(self.base + RING_REQ_BASE_H_OFFSET),
                    req_ring_base >> 32,
                )
                .unwrap();
            if self.resp_ring.len() > 0 {
                let resp_ring_base = self.resp_ring.as_ptr() as usize as u64;
                self.regs
                    .write(
                        addr(self.base + RING_RESP_BASE_L_OFFSET),
                        resp_ring_base as u32 as u64,
                    )
                    .unwrap();
                self.regs
                    .write(
                        addr(self.base + RING_RESP_BASE_H_OFFSET),
                        resp_ring_base >> 32,
                    )
                    .unwrap();
            }
            self.regs
                .write(addr(self.base + RING_SIZE_OFFSET), self.ring_size as u64)
                .unwrap();
            self.r_enable();
        }
    }

    impl<R: RegBus, REQ: Sized, RESP: Sized> Ring for SwQueue<R, REQ, RESP> {
        fn r_req_base(&self) -> u64 {
            self.req_ring.as_ptr() as usize as u64
        }
        fn r_resp_base(&self) -> Option<u64> {
            if self.resp_ring.len() > 0 {
                Some(self.resp_ring.as_ptr() as usize as u64)
            } else {
                None
            }
        }
        fn r_size(&self) -> usize {
            self.regs.read(addr(self.base + RING_SIZE_OFFSET)).unwrap() as usize
        }
        fn r_c_ptr(&self) -> usize {
            self.regs.read(addr(self.base + RING_PC_OFFSET)).unwrap() as usize
        }
        fn r_p_ptr(&self) -> usize {
            self.regs.read(addr(self.base + RING_PP_OFFSET)).unwrap() as usize
        }
        fn r_set_c_ptr(&self, v: usize) {
            self.regs
                .write(addr(self.base + RING_PC_OFFSET), v as u64)
                .unwrap();
        }
        fn r_set_p_ptr(&self, v: usize) {
            self.regs
                .write(addr(self.base + RING_PP_OFFSET), v as u64)
                .unwrap();
        }
        fn r_enabled(&self) -> bool {
            self.regs.read(addr(self.base + RING_CTRL_OFFSET)).unwrap() != 0
        }
        fn r_set_enable(&self, v: bool) {
            self.regs
                .write(addr(self.base + RING_CTRL_OFFSET), v as u64)
                .unwrap();
        }
        fn r_full(&self) -> bool {
            self.regs
                .read(addr(self.base + RING_STATUS_OFFSET))
                .unwrap() as usize
                & RING_FULL_FLAG
                != 0
        }
        fn r_empty(&self) -> bool {
            self.regs
                .read(addr(self.base + RING_STATUS_OFFSET))
                .unwrap() as usize
                & RING_EMPTY_FLAG
                != 0
        }
        fn r_set_full(&self, _v: bool) {}
        fn r_set_empty(&self, _v: bool) {}
        fn r_l_watermark(&self) -> usize {
            self.regs.read(addr(self.base + RING_L_WM_OFFSET)).unwrap() as usize
        }
        fn r_h_watermark(&self) -> usize {
            self.regs.read(addr(self.base + RING_H_WM_OFFSET)).unwrap() as usize
        }
        fn r_almost_empty(&self) -> bool {
            self.regs
                .read(addr(self.base + RING_STATUS_OFFSET))
                .unwrap() as usize
                & RING_AEMPTY_FLAG
                != 0
        }
        fn r_set_almost_full(&self, _v: bool) {}
        fn r_set_almost_empty(&self, _v: bool) {}
        fn r_irq_pendings(&self) -> usize {
            self.regs
                .read(addr(self.base + RING_STATUS_OFFSET))
                .unwrap() as usize
        }
    }

    pub trait SwRing {
        type REQ: Sized + Copy;
        type RESP: Sized + Copy;
        const REQ_SIZE: usize = std::mem::size_of::<Self::REQ>();
        const RESP_SIZE: usize = std::mem::size_of::<Self::RESP>();
        type R: Ring;
        fn get_ring(&self) -> &Self::R;

        fn r_get_req_at(&self, ptr: usize) -> *mut Self::REQ {
            let r_ptr_l = <Self::R as Ring>::r_ptr_l(ptr);
            let ptr = (self.get_ring().r_req_base() + r_ptr_l as u64 * Self::REQ_SIZE as u64)
                as usize as *mut Self::REQ;
            ptr
        }

        fn r_get_resp_at(&self, ptr: usize) -> Option<*const Self::RESP> {
            let r_ptr_l = <Self::R as Ring>::r_ptr_l(ptr);
            if let Some(base) = self.get_ring().r_resp_base() {
                let ptr =
                    (base + r_ptr_l as u64 * Self::RESP_SIZE as u64) as usize as *const Self::RESP;
                Some(ptr)
            } else {
                None
            }
        }

        fn r_push_req(&self, reqs: &[Self::REQ]) -> Option<usize> {
            if self.get_ring().r_p_valids() < reqs.len() {
                None
            } else {
                let ptr = self.get_ring().r_p_ptr();
                for (i, req) in reqs.iter().enumerate() {
                    unsafe { *self.r_get_req_at(self.get_ring().r_incr_ptr(ptr, i)) = *req };
                }
                self.get_ring().r_advance_p_n(reqs.len());
                Some(ptr)
            }
        }
    }
}
