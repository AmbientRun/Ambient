use std::{ffi::c_void, ptr::null_mut};

// PhysX is inherently unsafe, so we're not trying too hard to maintain the type safety
// guarantees.
#[allow(clippy::mut_from_ref)]
pub trait PxUserData {
    fn raw_user_data_mut(&self) -> &mut *mut c_void;
    fn raw_user_data(&self) -> &*mut c_void;

    fn set_user_data<T>(&self, data: T) -> Option<T> {
        let old = self.remove_user_data::<T>();
        *self.raw_user_data_mut() = Box::<T>::into_raw(Box::new(data)) as _;
        old
    }
    fn get_user_data<T: Clone>(&self) -> Option<T> {
        unsafe { ((*self.raw_user_data()) as *const T).as_ref() }.cloned()
    }
    fn remove_user_data<U>(&self) -> Option<U> {
        if !self.has_user_data() {
            return None;
        }
        unsafe {
            let p = Box::<U>::from_raw((*self.raw_user_data_mut()) as _);
            *self.raw_user_data_mut() = null_mut();
            Some(*p)
        }
    }
    fn has_user_data(&self) -> bool {
        !(*self.raw_user_data()).is_null()
    }
    fn update_user_data<U>(&self, map: &dyn Fn(&mut U)) {
        let mut ud = self.remove_user_data::<U>().unwrap();
        map(&mut ud);
        self.set_user_data(ud);
    }
}
