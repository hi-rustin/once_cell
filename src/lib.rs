use std::cell::UnsafeCell;
mod unsync {
    use super::UnsafeCell;
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
}

mod sync {
    use super::UnsafeCell;
    use std::sync::Once;

    pub struct OnceCell<T> {
        inner: UnsafeCell<Option<T>>,
        once: Once,
    }

    unsafe impl<T> Sync for OnceCell<T> {}

    impl<T> OnceCell<T> {
        pub fn new() -> Self {
            Self {
                inner: UnsafeCell::new(None),
                once: Once::new(),
            }
        }

        pub fn get(&self) -> Option<&T> {
            if self.once.is_completed() {
                unsafe { &(*self.inner.get()) }.as_ref()
            } else {
                None
            }
        }

        pub fn set(&self, value: T) -> Result<(), T> {
            if self.once.is_completed() {
                return Err(value);
            }
            let mut value = Some(value);
            self.once.call_once(|| {
                let inner = unsafe { &mut (*self.inner.get()) };
                debug_assert!(std::mem::replace(inner, value.take()).is_none());
            });

            match value {
                None => Ok(()),
                Some(value) => {
                    debug_assert!(self.once.is_completed());
                    Err(value)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsync_works() {
        let once: unsync::OnceCell<String> = unsync::OnceCell::new();
        assert!(once.get().is_none());
        assert!(once.set(String::new()).is_ok());
        assert!(once.set(String::new()).is_err());
        assert!(once.get().is_some());
        assert!(once.get().is_some());
    }

    #[test]
    fn sync_works() {
        use std::sync::Arc;

        let once = Arc::new(sync::OnceCell::new());

        let one = Arc::clone(&once);
        std::thread::spawn(move || {
            println!("{:?}", one.set(String::from("Hello")));
        });

        let two = Arc::clone(&once);
        std::thread::spawn(move || {
            println!("{:?}", two.set(String::from("World")));
        });

        std::thread::sleep(std::time::Duration::from_millis(10));

        println!("{:?}", once.get());
    }
}
