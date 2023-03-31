use std::marker::PhantomData;

use super::*;

/// Events packed into a FIFO frame queue
#[derive(Debug, Clone)]
pub struct FramedEvents<T> {
    events: Vec<Vec<T>>,
    frame: usize,
}
impl<T> FramedEvents<T> {
    pub const HISTORY_SIZE: usize = 100;

    pub fn new() -> Self {
        Self::new_with_history_size(Self::HISTORY_SIZE)
    }
    pub fn new_with_history_size(history_size: usize) -> Self {
        Self { events: (0..history_size).map(|_| Vec::new()).collect(), frame: 0 }
    }
    fn current_events_mut(&mut self) -> &mut Vec<T> {
        self.events_mut(self.frame)
    }
    fn current_events(&self) -> &Vec<T> {
        self.events(self.frame)
    }
    fn events(&self, frame: usize) -> &Vec<T> {
        &self.events[frame % self.events.len()]
    }
    fn events_mut(&mut self, frame: usize) -> &mut Vec<T> {
        let n_events = self.events.len();
        &mut self.events[frame % n_events]
    }
    pub fn next_frame(&mut self) {
        self.frame += 1;
        self.current_events_mut().clear();
    }
    pub fn add_event(&mut self, event: T) -> &T {
        let buf = self.current_events_mut();
        buf.push(event);
        buf.last().unwrap()
    }
    pub fn add_events(&mut self, events: impl IntoIterator<Item = T>) {
        self.current_events_mut().extend(events);
    }
    pub fn frame_available(&self, frame: usize) -> bool {
        frame >= self.frame.saturating_sub(self.events.len())
    }
    pub fn get(&self, id: DBEventId) -> Option<&T> {
        if !self.frame_available(id.frame) {
            return None;
        }
        self.events[id.frame % self.events.len()].get(id.index)
    }
    /// Creates a reader and moves it to the end of the events
    pub fn reader(&self) -> FramedEventsReader<T> {
        let mut reader = FramedEventsReader::<T>::new();
        reader.move_to_end(self);
        reader
    }
    pub fn n_events(&self) -> usize {
        self.current_events().len()
    }
}
impl<T> Default for FramedEvents<T> {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone)]
pub struct FramedEventsReader<T> {
    frame: usize,
    index: usize,
    _type: PhantomData<T>,
}
impl<T> FramedEventsReader<T> {
    pub fn new() -> Self {
        Self { frame: 0, index: 0, _type: PhantomData }
    }
    pub fn move_to_end(&mut self, events: &FramedEvents<T>) {
        self.frame = events.frame;
        self.index = events.current_events().len();
    }
    pub fn iter<'a>(&mut self, events: &'a FramedEvents<T>) -> FramedEventsIterator<'a, T> {
        let it = FramedEventsIterator { frame: self.frame, index: self.index, events };
        self.move_to_end(events);
        it
    }
}
impl<T> Default for FramedEventsReader<T> {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Clone)]
pub struct FramedEventsIterator<'a, T> {
    events: &'a FramedEvents<T>,
    frame: usize,
    index: usize,
}
impl<'a, T> Iterator for FramedEventsIterator<'a, T> {
    type Item = (DBEventId, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        if !self.events.frame_available(self.frame) {
            panic!(
                "Trying to read old events that have already been discarded ({} < {})",
                self.frame,
                self.events.frame - self.events.events.len()
            );
        }
        if self.frame == self.events.frame {
            if self.index >= self.events.current_events().len() {
                return None;
            }
        }

        let buf = self.events.events(self.frame);
        if let Some(event) = buf.get(self.index) {
            let event_id = DBEventId { frame: self.frame, index: self.index };
            self.index += 1;
            Some((event_id, event))
        } else {
            self.frame += 1;
            self.index = 0;
            self.next()
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct DBEventId {
    frame: usize,
    index: usize,
}

#[test]
fn test_events() {
    let mut events = FramedEvents::new_with_history_size(5);
    let mut reader = FramedEventsReader::new();
    assert_eq!(reader.iter(&events).count(), 0);

    events.add_event("a");
    assert_eq!(&reader.iter(&events).map(|x| *x.1).collect_vec(), &["a"]);
    assert_eq!(reader.iter(&events).count(), 0);

    events.add_event("b");
    events.add_event("c");
    assert_eq!(&reader.iter(&events).map(|x| *x.1).collect_vec(), &["b", "c"]);
    assert_eq!(reader.iter(&events).count(), 0);

    events.next_frame();
    assert_eq!(reader.iter(&events).count(), 0);
    events.add_event("d");
    assert_eq!(&reader.iter(&events).map(|x| *x.1).collect_vec(), &["d"]);
    assert_eq!(reader.iter(&events).count(), 0);

    events.add_event("e");
    events.next_frame();
    assert_eq!(&reader.iter(&events).map(|x| *x.1).collect_vec(), &["e"]);
    assert_eq!(reader.iter(&events).count(), 0);

    for _ in 0..20 {
        events.next_frame();
        events.add_event("x");
        assert_eq!(&reader.iter(&events).map(|x| *x.1).collect_vec(), &["x"]);
    }
}
