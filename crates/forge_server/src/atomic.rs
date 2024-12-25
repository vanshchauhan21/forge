use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct AtomicRef<A> {
    inner: Arc<Mutex<A>>,
}

impl<A: Clone> AtomicRef<A> {
    pub fn new(request: A) -> Self {
        Self { inner: Arc::new(Mutex::new(request)) }
    }

    pub fn get(&self) -> A {
        self.inner.lock().unwrap().clone()
    }

    pub fn set(&self, update: impl FnOnce(A) -> A) -> A {
        let mut request = self.inner.lock().unwrap();
        *request = update(request.clone());
        request.clone()
    }
}
