/// Simple histogram for accummulating size stats
#[derive(Clone, Default)]
pub struct SizeHistogram(Vec<usize>);

impl SizeHistogram {
    pub fn incr(&mut self, size: usize) {
        let idx = (if size < 1024 { 1 } else { size / 1024 }).ilog2() as usize;
        if self.0.len() <= idx {
            let additional = idx - self.0.len() + 1;
            self.0.reserve(additional);
            self.0.extend(std::iter::repeat(0).take(additional));
        }
        self.0[idx] += 1;
    }

    pub fn len(&self) -> usize {
        self.0.iter().sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::fmt::Display for SizeHistogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut total = 0;
        for (i, &count) in self.0.iter().enumerate() {
            let size = 2usize.pow(i as u32);
            total += count * size;
            writeln!(f, "{:>5}KB: {:>5}", size, count)?;
        }
        writeln!(f, "Average: {}KB", total / self.len())
    }
}
