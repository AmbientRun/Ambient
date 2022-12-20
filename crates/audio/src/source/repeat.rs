use crate::Source;

#[derive(Debug, Clone)]
pub struct Repeat<S> {
    orig: S,
    source: S,
}

impl<S> Repeat<S>
where
    S: Clone,
{
    pub fn new(source: S) -> Self {
        Self {
            orig: source.clone(),
            source,
        }
    }
}

impl<S> Source for Repeat<S>
where
    S: Source + Clone,
{
    fn next_sample(&mut self) -> Option<crate::Frame> {
        loop {
            match self.source.next_sample() {
                Some(v) => return Some(v),
                None => {
                    self.source = self.orig.clone();
                }
            }
        }
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.source.sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        self.source.sample_count()
    }
}
