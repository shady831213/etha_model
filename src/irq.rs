use etha_model_generator::*;
pub struct IrqBit {
    id: usize,
    name: String,
    handler: Option<Box<dyn Fn(usize) + Send>>,
}

impl IrqBit {
    fn new(id: usize, name: &str) -> Self {
        IrqBit {
            id,
            name: name.to_string(),
            handler: None,
        }
    }

    fn bind_handler<F: Fn(usize) + Send + 'static>(&mut self, handler: F) {
        self.handler = Some(Box::new(handler));
    }

    fn send_irq(&self) {
        if let Some(ref handler) = self.handler {
            (*handler)(self.id);
        }
    }
}

pub struct IrqVec {
    name: String,
    v: Vec<IrqBit>,
}

impl IrqVec {
    pub fn new(name: &str) -> Self {
        IrqVec {
            name: name.to_string(),
            v: vec![],
        }
    }

    pub fn alloc(&mut self, name: &str) -> usize {
        let id = self.v.len();
        self.v.push(IrqBit::new(id, name));
        id
    }

    pub fn bind<F: Fn(usize) + Send + 'static>(&mut self, id: usize, handler: F) -> Option<()> {
        if id < self.v.len() {
            self.v[id].bind_handler(handler);
            Some(())
        } else {
            None
        }
    }

    pub fn send(&self, id: usize) {
        self.v[id].send_irq()
    }
}

pub trait WithIrq {
    fn poll_irq(&self) -> Option<usize>;
}

impl ObjGenHeader for IrqVec {
    fn gen_c_header<W: std::io::Write>(&self, header: &mut W) -> std::io::Result<()> {
        writeln!(header, "typedef enum {{")?;
        for f in self.v.iter() {
            writeln!(header, "    {} = {},", f.name, f.id)?;
        }
        writeln!(header, "}} {};", self.name)?;
        Ok(())
    }
}
