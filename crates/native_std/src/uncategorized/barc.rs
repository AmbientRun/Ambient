use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

// Loosely based on https://github.com/mbrubeck/by_address/blob/master/src/lib.rs
// Works the same, except I just wanted a shorter name (i.e. Barc<T> instead of ByAddress<Arc<T>>)

/// A ByAddres Arc; this works identically to an Arc, except it also supports
/// Eq and Hash, which are based on the object/address the Arc points to rather
/// than the content of that object. I.e. two Barcs that point to the same address
/// are considered equal
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Barc<T: ?Sized>(pub Arc<T>);

impl<T> Barc<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }
}
impl<T: ?Sized> Clone for Barc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> PartialEq for Barc<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<T: ?Sized> Eq for Barc<T> {}

impl<T: ?Sized> Hash for Barc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.0), state)
    }
}

impl<T: ?Sized> Deref for Barc<T> {
    type Target = <Arc<T> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: ?Sized> AsRef<T> for Barc<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> From<T> for Barc<T> {
    fn from(t: T) -> Barc<T> {
        Barc::new(t)
    }
}

// pub trait BarcVecExt<T: Clone> {
//     fn push_im(&self, value: T) -> Self;
// }
// impl<T: Clone> BarcVecExt<T> for Barc<Vec<T>> {
//     fn push_im(&self, value: T) -> Self {
//         let mut x = self.0.as_ref().clone();
//         x.push(value);
//         Barc::new(x)
//     }
// }

// pub trait BarcHashMapExt<K: Clone, V: Clone> {
//     fn insert_im(&self, key: K, value: V) -> Self;
// }
// impl<K: Clone + std::cmp::Eq + std::hash::Hash, V: Clone> BarcHashMapExt<K, V> for Barc<HashMap<K, V>> {
//     fn insert_im(&self, key: K, value: V) -> Self {
//         let mut x = self.0.as_ref().clone();
//         x.insert(key, value);
//         Barc::new(x)
//     }
// }
