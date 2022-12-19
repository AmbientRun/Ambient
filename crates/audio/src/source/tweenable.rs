use tween::Tween;

#[derive(Debug, Clone)]
pub struct Tweenable<T> {
    tween: T,
    cursor: u64,
    sample_rate: u64,
}

impl<T> Tweenable<T>
where
    T: Tween<Value = f32, Time = f32>,
{
    pub fn new(tween: T, sample_rate: u64) -> Self {
        Self {
            tween,
            cursor: 0,
            sample_rate,
        }
    }

    pub fn next_value(&mut self) -> f32 {
        let time = (self.cursor as f32 / self.sample_rate as f32).min(*self.tween.range().end());
        let v = self.tween.run(time);
        self.cursor += 1;
        v
    }
}
