use std::{iter::Flatten, vec};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseVec<T>(pub Vec<Option<T>>);
impl<T> SparseVec<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn set(&mut self, index: usize, value: T) {
        if self.0.len() < index + 1 {
            self.0.resize_with(index + 1, || None);
        }
        self.0[index] = Some(value);
    }
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if let Some(v) = self.0.get_mut(index) {
            v.take()
        } else {
            None
        }
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        if let Some(Some(value)) = self.0.get(index) {
            Some(value)
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if let Some(Some(value)) = self.0.get_mut(index) {
            Some(value)
        } else {
            None
        }
    }
    pub fn get_mut_or_insert_with<F: FnOnce() -> T>(&mut self, index: usize, data: F) -> &mut T {
        if index < self.0.len() && self.0[index].is_some() {
            match &mut self.0[index] {
                Some(v) => v,
                None => unreachable!(),
            }
        } else {
            self.set(index, data());
            self.0[index].as_mut().unwrap()
        }
    }
    pub fn iter_all(&self) -> impl Iterator<Item = &Option<T>> {
        self.0.iter()
    }
    pub fn iter_all_mut(&mut self) -> impl Iterator<Item = &mut Option<T>> {
        self.0.iter_mut()
    }
    pub fn into_iter_enumerate(self) -> impl Iterator<Item = (usize, T)> {
        self.0
            .into_iter()
            .enumerate()
            .filter_map(|(i, x)| x.map(|x| (i, x)))
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().filter_map(|x| match x {
            Some(x) => Some(x),
            None => None,
        })
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.iter_mut().filter_map(|x| match x {
            Some(x) => Some(x),
            None => None,
        })
    }
    pub fn len(&self) -> usize {
        self.iter().count()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T> Default for SparseVec<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> IntoIterator for SparseVec<T> {
    type Item = T;

    type IntoIter = Flatten<vec::IntoIter<Option<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().flatten()
    }
}
