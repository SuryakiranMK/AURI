// AURI Playback Queue Manager
// Dynamic queue supporting sequential, shuffle, and repeat modes for local and streaming tracks

use std::sync::Mutex;
use crate::provider::TrackItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

pub struct QueueManager {
    items: Mutex<Vec<TrackItem>>,
    current_index: Mutex<Option<usize>>,
    shuffle: Mutex<bool>,
    shuffled_order: Mutex<Vec<usize>>,
    repeat_mode: Mutex<RepeatMode>,
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
            current_index: Mutex::new(None),
            shuffle: Mutex::new(false),
            shuffled_order: Mutex::new(Vec::new()),
            repeat_mode: Mutex::new(RepeatMode::Off),
        }
    }

    pub fn set_items(&self, items: Vec<TrackItem>, start_index: Option<usize>) {
        let len = items.len();
        *self.items.lock().unwrap() = items;
        *self.current_index.lock().unwrap() = start_index;

        if *self.shuffle.lock().unwrap() {
            self.regenerate_shuffle(len, start_index);
        }
    }

    pub fn add_item(&self, item: TrackItem) {
        let mut items = self.items.lock().unwrap();
        items.push(item);
        let len = items.len();
        let mut cur = self.current_index.lock().unwrap();
        if cur.is_none() {
            *cur = Some(0);
        }
        if *self.shuffle.lock().unwrap() {
            self.regenerate_shuffle(len, *cur);
        }
    }

    pub fn current_track(&self) -> Option<TrackItem> {
        let items = self.items.lock().unwrap();
        let cur = *self.current_index.lock().unwrap();
        cur.and_then(|idx| items.get(idx).cloned())
    }

    pub fn next(&self) -> Option<TrackItem> {
        let items = self.items.lock().unwrap();
        if items.is_empty() {
            return None;
        }

        let repeat = *self.repeat_mode.lock().unwrap();
        let mut cur = self.current_index.lock().unwrap();

        if repeat == RepeatMode::One {
            return cur.and_then(|idx| items.get(idx).cloned());
        }

        let is_shuffled = *self.shuffle.lock().unwrap();
        let next_idx = if is_shuffled {
            let order = self.shuffled_order.lock().unwrap();
            let cur_pos = cur.and_then(|c| order.iter().position(|&x| x == c)).unwrap_or(0);
            if cur_pos + 1 < order.len() {
                Some(order[cur_pos + 1])
            } else if repeat == RepeatMode::All {
                order.first().copied()
            } else {
                None
            }
        } else {
            let current = cur.unwrap_or(0);
            if current + 1 < items.len() {
                Some(current + 1)
            } else if repeat == RepeatMode::All {
                Some(0)
            } else {
                None
            }
        };

        if let Some(idx) = next_idx {
            *cur = Some(idx);
            items.get(idx).cloned()
        } else {
            None
        }
    }

    pub fn previous(&self) -> Option<TrackItem> {
        let items = self.items.lock().unwrap();
        if items.is_empty() {
            return None;
        }

        let mut cur = self.current_index.lock().unwrap();
        let current = cur.unwrap_or(0);

        let prev_idx = if current > 0 {
            Some(current - 1)
        } else if *self.repeat_mode.lock().unwrap() == RepeatMode::All {
            Some(items.len() - 1)
        } else {
            Some(0)
        };

        *cur = prev_idx;
        prev_idx.and_then(|idx| items.get(idx).cloned())
    }

    pub fn toggle_shuffle(&self) -> bool {
        let mut shuf = self.shuffle.lock().unwrap();
        *shuf = !*shuf;
        let is_active = *shuf;

        let items_len = self.items.lock().unwrap().len();
        let cur = *self.current_index.lock().unwrap();

        if is_active {
            self.regenerate_shuffle(items_len, cur);
        }
        is_active
    }

    pub fn toggle_repeat(&self) -> RepeatMode {
        let mut rep = self.repeat_mode.lock().unwrap();
        let next_rep = match *rep {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        };
        *rep = next_rep;
        next_rep
    }

    pub fn get_items(&self) -> Vec<TrackItem> {
        self.items.lock().unwrap().clone()
    }

    pub fn len(&self) -> usize {
        self.items.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.lock().unwrap().is_empty()
    }

    fn regenerate_shuffle(&self, len: usize, current_idx: Option<usize>) {
        let mut order: Vec<usize> = (0..len).collect();
        // Fisher-Yates with XorShift64
        let mut seed = 0xDEADBEEFCAFE1234u64;
        for i in (1..len).rev() {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let j = (seed as usize) % (i + 1);
            order.swap(i, j);
        }

        // Put current playing song first in shuffle order
        if let Some(cur) = current_idx {
            if let Some(pos) = order.iter().position(|&x| x == cur) {
                order.swap(0, pos);
            }
        }

        *self.shuffled_order.lock().unwrap() = order;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_navigation() {
        let queue = QueueManager::new();
        let tracks = vec![
            TrackItem { title: "Track 1".to_string(), ..Default::default() },
            TrackItem { title: "Track 2".to_string(), ..Default::default() },
            TrackItem { title: "Track 3".to_string(), ..Default::default() },
        ];

        queue.set_items(tracks, Some(0));
        assert_eq!(queue.current_track().unwrap().title, "Track 1");

        let next1 = queue.next().unwrap();
        assert_eq!(next1.title, "Track 2");

        let next2 = queue.next().unwrap();
        assert_eq!(next2.title, "Track 3");

        // Repeat off -> at end returns None
        assert!(queue.next().is_none());

        // Previous
        let prev = queue.previous().unwrap();
        assert_eq!(prev.title, "Track 2");
    }

    #[test]
    fn test_repeat_one() {
        let queue = QueueManager::new();
        let tracks = vec![
            TrackItem { title: "Track 1".to_string(), ..Default::default() },
            TrackItem { title: "Track 2".to_string(), ..Default::default() },
        ];
        queue.set_items(tracks, Some(0));
        queue.toggle_repeat(); // All
        queue.toggle_repeat(); // One

        let n1 = queue.next().unwrap();
        assert_eq!(n1.title, "Track 1");
        let n2 = queue.next().unwrap();
        assert_eq!(n2.title, "Track 1");
    }
}

