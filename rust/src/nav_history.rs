//! Browser-style navigation history for PDF jumps (links, go-to).
//!
//! Ordinary next/prev slide moves do not push history — only explicit jumps do,
//! matching classic pdfpc. Capacity is at least [`NAV_HISTORY_MAX`] page changes.

use std::collections::VecDeque;

/// Minimum number of page changes retained in the current session.
pub const NAV_HISTORY_MAX: usize = 100;

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

    /// Record leaving `from` before an explicit jump; clears the forward stack.
    pub fn push_jump(&mut self, from: i32) {
        self.back.push_back(from);
        trim_front(&mut self.back);
        self.forward.clear();
    }

    /// Undo the last jump: return the previous page and push `current` onto forward.
    pub fn go_back(&mut self, current: i32) -> Option<i32> {
        let target = self.back.pop_back()?;
        self.forward.push_back(current);
        trim_front(&mut self.forward);
        Some(target)
    }

    /// Redo a back: return the next forward page and push `current` onto back.
    pub fn go_forward(&mut self, current: i32) -> Option<i32> {
        let target = self.forward.pop_back()?;
        self.back.push_back(current);
        trim_front(&mut self.back);
        Some(target)
    }

}

fn trim_front(q: &mut VecDeque<i32>) {
    while q.len() > NAV_HISTORY_MAX {
        q.pop_front();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump_then_back_and_forward() {
        let mut h = NavHistory::default();
        h.push_jump(0); // leave page 0 → 5
        assert_eq!(h.go_back(5), Some(0));
        assert_eq!(h.go_forward(0), Some(5));
        assert_eq!(h.go_back(5), Some(0));
        assert_eq!(h.go_back(0), None);
    }

    #[test]
    fn new_jump_clears_forward() {
        let mut h = NavHistory::default();
        h.push_jump(0);
        let _ = h.go_back(3);
        // Now on 0 with forward=[3]. Jump elsewhere clears forward.
        h.push_jump(0);
        assert!(h.forward.is_empty());
        assert_eq!(h.go_forward(7), None);
        assert_eq!(h.go_back(7), Some(0));
    }

    #[test]
    fn retains_at_least_100_entries() {
        let mut h = NavHistory::default();
        for i in 0..150 {
            h.push_jump(i);
        }
        assert_eq!(h.back.len(), NAV_HISTORY_MAX);
        // Oldest kept is 50 (0..49 dropped).
        assert_eq!(h.go_back(150), Some(149));
        for _ in 0..(NAV_HISTORY_MAX - 1) {
            assert!(h.go_back(0).is_some());
        }
        assert_eq!(h.go_back(0), None);
    }

    #[test]
    fn clear_empties_both_stacks() {
        let mut h = NavHistory::default();
        h.push_jump(1);
        let _ = h.go_back(2);
        h.clear();
        assert!(h.back.is_empty());
        assert!(h.forward.is_empty());
    }
}
