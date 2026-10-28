//! Browser-style navigation history for PDF jumps (links, go-to).
//!
//! Ordinary next/prev slide moves do not push history — only explicit jumps do,
//! matching classic pdfpc. Capacity is at least [`NAV_HISTORY_MAX`] page changes.
//!
//! A page is only recorded when the caller decides the dwell threshold was met
//! (see [`crate::session`] / [`HISTORY_DWELL_DEFAULT`]).

use std::collections::VecDeque;
use std::time::Duration;

/// Minimum number of page changes retained in the current session.
pub const NAV_HISTORY_MAX: usize = 100;

/// Default minimum time on a page before it may enter history (5 seconds).
pub const HISTORY_DWELL_DEFAULT: Duration = Duration::from_secs(5);

/// Maximum entries shown in the jump-target picker (keys `1`–`9`).
pub const HISTORY_PICKER_MAX: usize = 9;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NavHistory {
    back: VecDeque<i32>,
    forward: VecDeque<i32>,
}

impl NavHistory {
    pub fn clear(&mut self) {
        self.back.clear();
        self.forward.clear();
    }

    /// Clear only the forward stack (used when jumping without recording `from`).
    pub fn clear_forward(&mut self) {
        self.forward.clear();
    }

    /// Record leaving `from` before an explicit jump; clears the forward stack.
    pub fn push_jump(&mut self, from: i32) {
        self.back.push_back(from);
        trim_front(&mut self.back);
        self.forward.clear();
    }

    /// Undo the last jump. If `record_current` is set, push it onto forward.
    pub fn go_back(&mut self, record_current: Option<i32>) -> Option<i32> {
        let target = self.back.pop_back()?;
        if let Some(current) = record_current {
            self.forward.push_back(current);
            trim_front(&mut self.forward);
        }
        Some(target)
    }

    /// Redo a back. If `record_current` is set, push it onto back.
    pub fn go_forward(&mut self, record_current: Option<i32>) -> Option<i32> {
        let target = self.forward.pop_back()?;
        if let Some(current) = record_current {
            self.back.push_back(current);
            trim_front(&mut self.back);
        }
        Some(target)
    }

    /// Jump targets for the picker: back stack, most recent first (up to max).
    pub fn picker_targets(&self, max: usize) -> Vec<i32> {
        self.back.iter().rev().copied().take(max).collect()
    }
}

fn trim_front(q: &mut VecDeque<i32>) {
    while q.len() > NAV_HISTORY_MAX {
        q.pop_front();
    }
}

/// Whether `entered_at`..`now` meets the dwell threshold for recording history.
pub fn dwell_allows_record(entered_at: std::time::Instant, now: std::time::Instant, min: Duration) -> bool {
    now.duration_since(entered_at) >= min
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn jump_then_back_and_forward() {
        let mut h = NavHistory::default();
        h.push_jump(0); // leave page 0 → 5
        assert_eq!(h.go_back(Some(5)), Some(0));
        assert_eq!(h.go_forward(Some(0)), Some(5));
        assert_eq!(h.go_back(Some(5)), Some(0));
        assert_eq!(h.go_back(Some(0)), None);
    }

    #[test]
    fn back_without_recording_current() {
        let mut h = NavHistory::default();
        h.push_jump(0);
        h.push_jump(2);
        // Leave page 4 quickly: do not push onto forward.
        assert_eq!(h.go_back(None), Some(2));
        assert!(h.forward.is_empty());
        assert_eq!(h.go_forward(Some(4)), None);
    }

    #[test]
    fn new_jump_clears_forward() {
        let mut h = NavHistory::default();
        h.push_jump(0);
        let _ = h.go_back(Some(3));
        // Now on 0 with forward=[3]. Jump elsewhere clears forward.
        h.push_jump(0);
        assert!(h.forward.is_empty());
        assert_eq!(h.go_forward(Some(7)), None);
        assert_eq!(h.go_back(Some(7)), Some(0));
    }

    #[test]
    fn jump_without_record_still_clears_forward() {
        let mut h = NavHistory::default();
        h.push_jump(0);
        let _ = h.go_back(Some(3));
        h.clear_forward();
        assert!(h.forward.is_empty());
        assert!(h.back.is_empty());
    }

    #[test]
    fn retains_at_least_100_entries() {
        let mut h = NavHistory::default();
        for i in 0..150 {
            h.push_jump(i);
        }
        assert_eq!(h.back.len(), NAV_HISTORY_MAX);
        assert_eq!(h.go_back(Some(150)), Some(149));
        for _ in 0..(NAV_HISTORY_MAX - 1) {
            assert!(h.go_back(Some(0)).is_some());
        }
        assert_eq!(h.go_back(Some(0)), None);
    }

    #[test]
    fn picker_targets_most_recent_first() {
        let mut h = NavHistory::default();
        h.push_jump(1);
        h.push_jump(3);
        h.push_jump(5);
        assert_eq!(h.picker_targets(9), vec![5, 3, 1]);
        assert_eq!(h.picker_targets(2), vec![5, 3]);
    }

    #[test]
    fn clear_empties_both_stacks() {
        let mut h = NavHistory::default();
        h.push_jump(1);
        let _ = h.go_back(Some(2));
        h.clear();
        assert!(h.back.is_empty());
        assert!(h.forward.is_empty());
    }

    #[test]
    fn dwell_gate() {
        let start = Instant::now();
        assert!(!dwell_allows_record(start, start + Duration::from_secs(4), HISTORY_DWELL_DEFAULT));
        assert!(dwell_allows_record(start, start + Duration::from_secs(5), HISTORY_DWELL_DEFAULT));
        assert!(dwell_allows_record(start, start + Duration::from_millis(1), Duration::ZERO));
    }
}
