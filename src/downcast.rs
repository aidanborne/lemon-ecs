use std::any::Any;

pub trait AsAny: 'static + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Sized> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

macro_rules! impl_downcast {
    (dyn $ident:ident) => {
        impl dyn $ident {
            pub fn downcast<U: 'static>(self: Box<Self>) -> Result<Box<U>, Box<Self>> {
                if (*self).as_any().is::<U>() {
                    unsafe {
                        let raw_ptr = Box::into_raw(self);
                        Ok(Box::from_raw(raw_ptr as *mut U))
                    }
                } else {
                    Err(self)
                }
            }

            pub fn downcast_ref<U: 'static>(&self) -> Option<&U> {
                (*self).as_any().downcast_ref::<U>()
            }

            pub fn downcast_mut<U: 'static>(&mut self) -> Option<&mut U> {
                (*self).as_any_mut().downcast_mut::<U>()
            }
        }
    };
}

pub(crate) use impl_downcast;
