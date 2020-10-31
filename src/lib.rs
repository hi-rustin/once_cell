use std::cell::UnsafeCell;

pub struct OnceCell<T> {
    inner: UnsafeCell<Option<T>>,
}

impl<T> OnceCell<T> {
    pub fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    pub fn get(&self) -> Option<&T> {
        let ptr = self.inner.get();
        // SAFETY:
        // We're sure that the pointer is valid
        // We're in a single thread and so no race condition is possible
        // We're always returning a &T not a &mut T
        unsafe { &*ptr }.as_ref()
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        if self.get().is_some() {
            return Err(value);
        }
        // SAFETY:
        // * we have exclusive access. We must write the value
        let r = unsafe { &mut *self.inner.get() };
        let old = std::mem::replace(r, Some(value));
        debug_assert!(old.is_none());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let once: OnceCell<String> = OnceCell::new();
        assert!(once.get().is_none());
        assert!(once.set(String::new()).is_ok());
        assert!(once.set(String::new()).is_err());
        assert!(once.get().is_some());
        assert!(once.get().is_some());
    }
}
