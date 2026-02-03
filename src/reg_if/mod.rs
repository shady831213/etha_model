pub mod ring;

pub trait RegBus {
    fn write(&self, addr: u64, data: u64) -> Option<()>;
    fn read(&self, addr: u64) -> Option<u64>;
}
