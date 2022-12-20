use std::{collections::VecDeque, marker::PhantomData};

use super::*;

/// Events packed into a FIFO frame queue
#[derive(Debug, Clone)]
pub struct FramedEvents<T> {
    events: VecDeque<Vec<T>>,
    start_frame: usize,
    history_size: usize,
}
impl<T> FramedEvents<T> {
    pub const HISTORY_SIZE: usize = 100;

    pub fn new() -> Self {
        Self::new_with_history_size(Self::HISTORY_SIZE)
    }
    pub fn new_with_history_size(history_size: usize) -> Self {
        Self { events: vec![Vec::new()].into(), start_frame: 0, history_size }
    }
    pub fn next_frame(&mut self) {
        if self.events.len() < self.history_size {
            self.events.push_back(Vec::new());
        } else {
            self.start_frame += 1;
            let mut buf = self.events.pop_front().unwrap();
            buf.clear(); // Re-use the same buffer, so that it's internal size is already allocated
            self.events.push_back(buf);
        }
    }
    pub fn add_event(&mut self, event: T) -> &T {
        let index = self.events.len() - 1;
        let buf = &mut self.events[index];
        buf.push(event);
        buf.last().unwrap()
    }
    pub fn add_events(&mut self, events: impl IntoIterator<Item = T>) {
        let index = self.events.len() - 1;
        let buf = &mut self.events[index];
        buf.extend(events);
    }
    pub fn get(&self, id: DBEventId) -> Option<&T> {
        if id.frame < self.start_frame {
            return None;
        }
        let index = id.frame - self.start_frame;
        self.events[index].get(id.index)
    }
    pub fn n_events(&self) -> usize {
        self.events[self.events.len() - 1].len()
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
        self.frame = events.start_frame + events.events.len() - 1;
        self.index = events.events[events.events.len() - 1].len();
    }
    pub fn iter<'a>(&mut self, events: &'a FramedEvents<T>) -> FramedEventsIterator<'a, T> {
        let it = FramedEventsIterator { frame: self.frame, index: self.index, events };
        self.move_to_end(events);
        it
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
        if self.frame < self.events.start_frame {
            panic!("Trying to read old events that have already been discarded ({} < {})", self.frame, self.events.start_frame);
        }
        let frame_index = self.frame - self.events.start_frame;
        if let Some(buf) = self.events.events.get(frame_index) {
            if let Some(event) = buf.get(self.index) {
                let event_id = DBEventId { frame: self.frame, index: self.index };
                self.index += 1;
                Some((event_id, event))
            } else {
                self.frame += 1;
                self.index = 0;
                self.next()
            }
        } else {
            None
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
