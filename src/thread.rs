//! Mock implementation of `std::thread`.

use crate::rt;
pub use crate::rt::yield_now;
pub use std::thread::AccessError;

use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

/// Mock implementation of `std::thread::JoinHandle`.
pub struct JoinHandle<T> {
    result: Rc<RefCell<Option<std::thread::Result<T>>>>,
    notify: rt::Notify,
}

/// Mock implementation of `std::thread::LocalKey`.
pub struct LocalKey<T> {
    // Sadly, these fields have to be public, since function pointers in const
    // fns are unstable. When fn pointer arguments to const fns stabilize, these
    // should be made private and replaced with a `const fn new`.
    //
    // User code should not rely on the existence of these fields.
    #[doc(hidden)]
    pub init: fn() -> T,
    #[doc(hidden)]
    pub _p: PhantomData<fn(T)>,
}

/// Mock implementation of `std::thread::spawn`.
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: 'static,
    T: 'static,
{
    let result = Rc::new(RefCell::new(None));
    let notify = rt::Notify::new(true);

    {
        let result = result.clone();
        rt::spawn(move || {
            *result.borrow_mut() = Some(Ok(f()));
            notify.notify();
        });
    }

    JoinHandle { result, notify }
}

impl<T> JoinHandle<T> {
    /// Waits for the associated thread to finish.
    pub fn join(self) -> std::thread::Result<T> {
        self.notify.wait();
        self.result.borrow_mut().take().unwrap()
    }
}

impl<T: fmt::Debug> fmt::Debug for JoinHandle<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("JoinHandle").finish()
    }
}

impl<T: 'static> LocalKey<T> {
    /// Mock implementation of `std::thread::LocalKey::with`.
    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        rt::execution(|execution| {
            let value = execution.threads.local(self);
            f(value)
        })
    }

    /// Mock implementation of `std::thread::LocalKey::try_with`.
    pub fn try_with<F, R>(&'static self, f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&T) -> R,
    {
        // TODO(eliza): handle destructors
        Ok(self.with(f))
    }
}

impl<T: 'static> fmt::Debug for LocalKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("LocalKey { .. }")
    }
}
