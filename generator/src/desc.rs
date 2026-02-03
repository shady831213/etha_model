#[derive(Debug)]
pub struct DescField {
    pub name: &'static str,
    pub lsb: usize,
    pub msb: usize,
}
