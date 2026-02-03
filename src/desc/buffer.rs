use super::*;
use etha_model_generator::*;
use std::convert::{From, TryFrom};
use std::io::{self, Read, Write};
#[derive(Debug)]
pub struct MemBlock {
    pub addr: u64,
    pub size: usize,
}

impl MemBlock {
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.addr as usize as *const u8, self.size) }
    }
}

impl<'a> From<&'a [u8]> for MemBlock {
    fn from(b: &'a [u8]) -> Self {
        MemBlock {
            addr: b.as_ptr() as u64,
            size: b.len(),
        }
    }
}

impl Write for MemBlock {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        tracing::debug!(target: "buffer", "MemBlock begin write to {:#x}, size: {}", self.addr, self.size);
        if buf.len() > self.size {
            return Err(io::Error::from(io::ErrorKind::OutOfMemory));
        }
        unsafe {
            std::slice::from_raw_parts_mut(self.addr as usize as *mut u8, buf.len())
                .copy_from_slice(&buf[..buf.len()])
        }
        tracing::debug!(target: "buffer", "MemBlock end write to {:#x}, size: {}", self.addr, self.size);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for MemBlock {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        tracing::debug!(target: "buffer", "MemBlock begin read from {:#x}, size: {}", self.addr, self.size);
        if buf.len() < self.size {
            return Err(io::Error::from(io::ErrorKind::OutOfMemory));
        }
        unsafe {
            buf[..self.size].copy_from_slice(std::slice::from_raw_parts(
                self.addr as usize as *const u8,
                self.size,
            ))
        }
        tracing::debug!(target: "buffer", "MemBlock end read from {:#x}, size: {}", self.addr, self.size);
        Ok(self.size)
    }
}

const FRAME_DESC_SIZE: usize = 12;
mod bitfields {
    use super::*;
    use bitfield::bitfield;
    #[desc_gen]
    bitfield! {
        #[repr(C)]
        #[derive(Default, Copy, Clone)]
        pub struct SCFrameDesc([DescEntryT]);
        impl Debug;
        DescEntryT;
        pub addr_lo, set_addr_lo: 31, 0;
        pub addr_hi, set_addr_hi: 63, 32;
        pub total_size, set_total_size: 87, 64;
        pub n_blocks, set_n_blocks: 95, 88;
    }
}
pub type SCFrameDesc = bitfields::SCFrameDesc<[DescEntryT; FRAME_DESC_SIZE / DESC_ENTRY_SIZE]>;

impl SCFrameDesc {
    pub fn full_addr(&self) -> u64 {
        self.addr_lo() as u64 | ((self.addr_hi() as u64) << 32)
    }
    pub fn is_linear(&self) -> bool {
        self.n_blocks() == 0
    }
    pub fn to_vec(&self) -> Vec<MemBlock> {
        if self.is_linear() {
            vec![MemBlock {
                addr: self.full_addr(),
                size: self.total_size() as usize,
            }]
        } else {
            unsafe {
                std::slice::from_raw_parts(
                    self.full_addr() as usize as *const SCBufferEntry,
                    self.n_blocks() as usize + 1,
                )
            }
            .iter()
            .map(|b| MemBlock {
                addr: b.addr,
                size: b.size as usize,
            })
            .collect::<Vec<_>>()
        }
    }
    pub fn read_with<G: Fn(u64, usize), F: Fn(&MemBlock)>(
        &mut self,
        buf: &mut [u8],
        f: F,
        g: G,
    ) -> io::Result<usize> {
        let mut pos = 0;
        let blocks = self.to_vec();
        g(self.full_addr(), blocks.len());
        for mut b in blocks {
            f(&b);
            pos += b.read(&mut buf[pos..])?;
        }
        if pos == self.total_size() as usize {
            Ok(pos)
        } else {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }
    }

    pub fn write_with<G: Fn(u64, usize), F: Fn(&MemBlock)>(
        &mut self,
        buf: &[u8],
        f: F,
        g: G,
    ) -> io::Result<usize> {
        let mut pos = 0;
        let blocks = self.to_vec();
        g(self.full_addr(), blocks.len());
        for mut b in blocks {
            pos += b.write(&buf[pos..pos + b.size])?;
            f(&b);
            if pos == buf.len() {
                break;
            }
        }
        Ok(pos)
    }
}

impl Read for SCFrameDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_with(buf, |_| {}, |_, _| {})
    }
}

impl Write for SCFrameDesc {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_with(buf, |_| {}, |_, _| {})
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl From<MemBlock> for SCFrameDesc {
    fn from(b: MemBlock) -> Self {
        let mut d = Self::default();
        d.set_addr_lo(b.addr as DescEntryT);
        d.set_addr_hi((b.addr >> 32) as DescEntryT);
        d.set_total_size(b.size as DescEntryT);
        d
    }
}

impl<'a> TryFrom<&'a [SCBufferEntry]> for SCFrameDesc {
    type Error = &'static str;
    fn try_from(b: &'a [SCBufferEntry]) -> Result<Self, Self::Error> {
        if b.len() == 0 {
            Err("empty sc list!")
        } else {
            let mut d = Self::default();
            if b.len() == 1 {
                d.set_addr_lo(b[0].addr as u32);
                d.set_addr_hi((b[0].addr >> 32) as u32);
                d.set_total_size(b[0].size as u32);
            } else {
                let addr = b.as_ptr() as u64;
                d.set_addr_lo(addr as DescEntryT);
                d.set_addr_hi((addr >> 32) as DescEntryT);
                let size = b.iter().map(|a| a.size).reduce(|acc, a| acc + a).unwrap();
                d.set_total_size(size);
                d.set_n_blocks(b.len() as u32 - 1);
            }
            Ok(d)
        }
    }
}

#[desc_gen]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SCBufferEntry {
    pub addr: u64,
    pub size: u32,
    pub reserved: u32,
}

impl Into<MemBlock> for SCBufferEntry {
    fn into(self) -> MemBlock {
        MemBlock {
            addr: self.addr,
            size: self.size as usize,
        }
    }
}

impl From<MemBlock> for SCBufferEntry {
    fn from(b: MemBlock) -> Self {
        SCBufferEntry {
            addr: b.addr,
            size: b.size as u32,
            reserved: 0,
        }
    }
}
