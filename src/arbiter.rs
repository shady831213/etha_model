pub trait Arbiter {
    fn arbit<T: Clone>(&mut self, req: &[Option<T>]) -> Option<(usize, T)>;
}

pub struct RRArbiter<const N: usize> {
    next: usize,
}

impl<const N: usize> RRArbiter<N> {
    pub fn new() -> Self {
        RRArbiter { next: 0 }
    }
    fn update(&mut self, i: usize) {
        if i < N - 1 {
            self.next = i + 1;
        } else {
            self.next = 0;
        }
    }
}

impl<const N: usize> Arbiter for RRArbiter<N> {
    fn arbit<T: Clone>(&mut self, req: &[Option<T>]) -> Option<(usize, T)> {
        let mut req_v: Vec<Option<T>> = req[..N].to_vec();
        req_v.rotate_left(self.next);
        for (i, req) in req_v.iter_mut().enumerate() {
            if let Some(d) = req.take() {
                let r = (self.next + i) % N;
                self.update(r);
                return Some((r, d));
            }
        }
        None
    }
}
