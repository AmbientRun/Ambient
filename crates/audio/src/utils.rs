use std::ops::{AddAssign, Div};

/// Averages consecutive elements of at most `stride`
pub struct AvgIter<I> {
    iter: I,
    stride: usize,
}

impl<I> AvgIter<I> {
    pub fn new(iter: I, stride: usize) -> Self {
        Self { iter, stride }
    }
}

impl<I, T> Iterator for AvgIter<I>
where
    I: Iterator<Item = T>,
    T: Default + AddAssign<T> + Div<f32, Output = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut acc = T::default();
        let mut count = 0;
        for v in self.iter.by_ref().take(self.stride) {
            acc += v;
            count += 1;
        }

        if count == 0 {
            return None;
        }

        Some(acc / count as f32)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.iter.size_hint();

        if self.stride == 0 {
            return (0, Some(0));
        }

        (
            div_ceil(min, self.stride),
            max.map(|v| div_ceil(v, self.stride)),
        )
    }
}

pub fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

impl<I, T> ExactSizeIterator for AvgIter<I>
where
    I: ExactSizeIterator<Item = T>,
    T: Default + AddAssign<T> + Div<f32, Output = T>,
{
}
