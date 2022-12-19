use crate::{Frame, Source};

/// A pseudo-source which allows you to peek one frame ahead
#[derive(Debug, Clone)]
pub struct Peek<S> {
    source: S,
    peeked: Option<Option<Frame>>,
}

impl<S> std::ops::Deref for Peek<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

impl<S> Peek<S>
where
    S: Source,
{
    pub fn new(source: S) -> Self {
        Self {
            source,
            peeked: None,
        }
    }

    pub fn peek(&mut self) -> Option<Frame> {
        match self.peeked {
            Some(v) => v,
            None => *self.peeked.insert(self.source.next_sample()),
        }
    }

    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.source
    }
}

impl<S> Source for Peek<S>
where
    S: Source,
{
    fn next_sample(&mut self) -> Option<Frame> {
        match self.peeked.take() {
            Some(v) => v,
            None => self.source.next_sample(),
        }
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}
