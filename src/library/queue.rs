// Dynamic Playback Queue Manager
// Handles Next, Previous, History, Fisher-Yates Shuffle, and Repeat modes (Off, One, All).

use crate::library::metadata::TrackMetadata;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

impl RepeatMode {
    pub fn cycle(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }
}

// Lightweight 64-bit XorShift pseudo-random generator (avoids bulky rand crate)
struct XorShift64(u64);

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x5EED_CAFE_BABE_D00D } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn gen_range(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            0
        } else {
            (self.next_u64() % (upper as u64)) as usize
        }
    }
}

pub struct PlayQueue {
    items: Vec<TrackMetadata>,
    current_index: Option<usize>,
    history: Vec<usize>,
    shuffle: bool,
    shuffled_order: Vec<usize>,
    shuffle_pos: usize,
    repeat: RepeatMode,
    rng: XorShift64,
}

impl PlayQueue {
    pub fn new() -> Self {
        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(12345);

        Self {
            items: Vec::new(),
            current_index: None,
            history: Vec::new(),
            shuffle: false,
            shuffled_order: Vec::new(),
            shuffle_pos: 0,
            repeat: RepeatMode::Off,
            rng: XorShift64::new(now_nanos),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn current_track(&self) -> Option<&TrackMetadata> {
        self.current_index.and_then(|idx| self.items.get(idx))
    }

    pub fn items(&self) -> &[TrackMetadata] {
        &self.items
    }

    pub fn is_shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat
    }

    pub fn set_queue(&mut self, tracks: Vec<TrackMetadata>, start_idx: usize) {
        self.items = tracks;
        self.history.clear();
        if !self.items.is_empty() {
            let clamped_idx = start_idx.min(self.items.len() - 1);
            self.current_index = Some(clamped_idx);
            self.history.push(clamped_idx);
            self.rebuild_shuffle_order();
        } else {
            self.current_index = None;
            self.shuffled_order.clear();
            self.shuffle_pos = 0;
        }
    }

    pub fn add_track(&mut self, track: TrackMetadata) {
        self.items.push(track);
        if self.current_index.is_none() {
            self.current_index = Some(0);
            self.history.push(0);
        }
        self.rebuild_shuffle_order();
    }

    pub fn remove_track(&mut self, index: usize) -> Option<TrackMetadata> {
        if index >= self.items.len() {
            return None;
        }

        let removed = self.items.remove(index);

        if let Some(cur) = self.current_index {
            if cur == index {
                if self.items.is_empty() {
                    self.current_index = None;
                } else {
                    self.current_index = Some(cur.min(self.items.len() - 1));
                }
            } else if cur > index {
                self.current_index = Some(cur - 1);
            }
        }

        self.rebuild_shuffle_order();
        Some(removed)
    }

    pub fn play_index(&mut self, index: usize) -> Option<&TrackMetadata> {
        if index < self.items.len() {
            self.current_index = Some(index);
            self.history.push(index);
            if self.shuffle {
                if let Some(pos) = self.shuffled_order.iter().position(|&i| i == index) {
                    self.shuffle_pos = pos;
                }
            }
            self.items.get(index)
        } else {
            None
        }
    }

    /// Resolves next track according to current Repeat and Shuffle settings.
    pub fn next(&mut self) -> Option<&TrackMetadata> {
        if self.items.is_empty() {
            return None;
        }

        // Repeat One: replay current track
        if self.repeat == RepeatMode::One {
            return self.current_track();
        }

        if self.shuffle {
            if self.shuffle_pos + 1 < self.shuffled_order.len() {
                self.shuffle_pos += 1;
                let next_idx = self.shuffled_order[self.shuffle_pos];
                self.current_index = Some(next_idx);
                self.history.push(next_idx);
                return self.items.get(next_idx);
            } else if self.repeat == RepeatMode::All {
                self.rebuild_shuffle_order();
                self.shuffle_pos = 0;
                if let Some(&first_idx) = self.shuffled_order.first() {
                    self.current_index = Some(first_idx);
                    self.history.push(first_idx);
                    return self.items.get(first_idx);
                }
            }
            None
        } else {
            let cur = self.current_index.unwrap_or(0);
            if cur + 1 < self.items.len() {
                let next_idx = cur + 1;
                self.current_index = Some(next_idx);
                self.history.push(next_idx);
                self.items.get(next_idx)
            } else if self.repeat == RepeatMode::All {
                self.current_index = Some(0);
                self.history.push(0);
                self.items.get(0)
            } else {
                None
            }
        }
    }

    /// Resolves previous track from history or previous index.
    pub fn previous(&mut self) -> Option<&TrackMetadata> {
        if self.items.is_empty() {
            return None;
        }

        if self.history.len() > 1 {
            // Pop current
            self.history.pop();
            // Get previous
            if let Some(&prev_idx) = self.history.last() {
                if prev_idx < self.items.len() {
                    self.current_index = Some(prev_idx);
                    return self.items.get(prev_idx);
                }
            }
        }

        // Fallback: previous sequential
        let cur = self.current_index.unwrap_or(0);
        let prev_idx = if cur > 0 { cur - 1 } else { 0 };
        self.current_index = Some(prev_idx);
        self.items.get(prev_idx)
    }

    pub fn set_shuffle(&mut self, enabled: bool) {
        self.shuffle = enabled;
        if enabled {
            self.rebuild_shuffle_order();
        }
    }

    pub fn toggle_shuffle(&mut self) -> bool {
        self.set_shuffle(!self.shuffle);
        self.shuffle
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;
    }

    pub fn toggle_repeat(&mut self) -> RepeatMode {
        self.repeat = self.repeat.cycle();
        self.repeat
    }

    fn rebuild_shuffle_order(&mut self) {
        let n = self.items.len();
        self.shuffled_order = (0..n).collect();
        if n <= 1 {
            self.shuffle_pos = 0;
            return;
        }

        // Fisher-Yates shuffle
        for i in (1..n).rev() {
            let j = self.rng.gen_range(i + 1);
            self.shuffled_order.swap(i, j);
        }

        // Place current track at current shuffle_pos
        if let Some(cur) = self.current_index {
            if let Some(pos) = self.shuffled_order.iter().position(|&x| x == cur) {
                self.shuffled_order.swap(0, pos);
                self.shuffle_pos = 0;
            }
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.current_index = None;
        self.history.clear();
        self.shuffled_order.clear();
        self.shuffle_pos = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dummy_track(title: &str) -> TrackMetadata {
        let mut m = TrackMetadata::new(PathBuf::from(format!("{}.mp3", title)));
        m.title = title.to_string();
        m
    }

    #[test]
    fn test_queue_sequential_navigation() {
        let mut q = PlayQueue::new();
        q.set_queue(vec![dummy_track("T1"), dummy_track("T2"), dummy_track("T3")], 0);

        assert_eq!(q.current_track().unwrap().title, "T1");

        let next1 = q.next().unwrap();
        assert_eq!(next1.title, "T2");

        let next2 = q.next().unwrap();
        assert_eq!(next2.title, "T3");

        // End of queue with repeat off
        assert!(q.next().is_none());

        // Test previous
        let prev = q.previous().unwrap();
        assert_eq!(prev.title, "T2");
    }

    #[test]
    fn test_queue_repeat_all() {
        let mut q = PlayQueue::new();
        q.set_queue(vec![dummy_track("A"), dummy_track("B")], 1);
        q.set_repeat(RepeatMode::All);

        assert_eq!(q.current_track().unwrap().title, "B");
        let next = q.next().unwrap();
        assert_eq!(next.title, "A"); // Wraps to beginning
    }

    #[test]
    fn test_queue_repeat_one() {
        let mut q = PlayQueue::new();
        q.set_queue(vec![dummy_track("A"), dummy_track("B")], 0);
        q.set_repeat(RepeatMode::One);

        assert_eq!(q.next().unwrap().title, "A");
        assert_eq!(q.next().unwrap().title, "A");
    }

    #[test]
    fn test_queue_shuffle() {
        let mut q = PlayQueue::new();
        let tracks: Vec<_> = (0..20).map(|i| dummy_track(&format!("T{}", i))).collect();
        q.set_queue(tracks, 0);
        q.set_shuffle(true);

        assert_eq!(q.shuffled_order.len(), 20);
        // Current track should be first in shuffled order
        assert_eq!(q.shuffled_order[0], 0);
    }
}
