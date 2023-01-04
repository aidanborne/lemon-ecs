use std::any::Any;

pub trait AsAny: 'static {
    fn as_any(&self) -> &dyn Any;
}

/// Allows for downcasting a trait object to a concrete type.
pub trait Downcast {
    fn downcast<T: 'static>(self: Box<Self>) -> Result<Box<T>, Box<Self>>;
    fn downcast_ref<T: 'static>(&self) -> Option<&T>;
}

impl<T: AsAny + ?Sized> Downcast for T {
    fn downcast<U: 'static>(self: Box<Self>) -> Result<Box<U>, Box<Self>> {
        if self.as_any().is::<U>() {
            unsafe {
                let raw_ptr = Box::into_raw(self);
                Ok(Box::from_raw(raw_ptr as *mut U))
            }
        } else {
            Err(self)
        }
    }

    fn downcast_ref<U: 'static>(&self) -> Option<&U> {
        self.as_any().downcast_ref::<U>()
    }
}
