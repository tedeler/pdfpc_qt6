//! Browser-style navigation history for PDF jumps (links, go-to, J-menu).
//!
//! The trail is a line with a cursor, stored as a back stack plus a forward
//! tail. Going back or forward only moves along that line. A new jump (link,
//! go-to, or a pick from the J-menu) truncates the forward tail and continues
//! from the current position — it does not wipe the pages already behind you.
//!
//! Ordinary next/prev slide moves do not push history. A page is recorded on
//! a new jump only when the caller says the dwell threshold was met.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Minimum number of page changes retained in the current session.
pub const NAV_HISTORY_MAX: usize = 100;

/// Default minimum time on a page before it may enter history (5 seconds).
pub const HISTORY_DWELL_DEFAULT: Duration = Duration::from_secs(5);

/// Maximum entries shown in the jump-target picker (keys `1`–`9`).
pub const HISTORY_PICKER_MAX: usize = 9;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpTarget {
    pub page: i32,
    pub last_visited: Instant,
}

#[derive(Debug, Default, Clone)]
pub struct NavHistory {
    back: VecDeque<i32>,
    forward: VecDeque<i32>,
    /// When the user was last on this page (arrival or departure).
    last_visited: HashMap<i32, Instant>,
}

impl NavHistory {
    pub fn clear(&mut self) {
        self.back.clear();
        self.forward.clear();
        self.last_visited.clear();
    }

    /// Remember that `page` was visited at `now`.
    pub fn note_visit(&mut self, page: i32, now: Instant) {
        self.last_visited.insert(page, now);
    }

    /// New jump away from `from`. Drops only the forward tail.
    /// `from` is kept on the back stack when `record_from` is set (dwell met).
    pub fn begin_new_jump(&mut self, from: i32, now: Instant, record_from: bool) {
        self.forward.clear();
        self.note_visit(from, now);
        if record_from {
            self.back.push_back(from);
            trim_front(&mut self.back);
        }
    }

    /// Step back. The page being left stays on the forward tail.
    pub fn go_back(&mut self, current: i32, now: Instant) -> Option<i32> {
        let target = self.back.pop_back()?;
        self.note_visit(current, now);
        self.forward.push_back(current);
        trim_front(&mut self.forward);
        self.note_visit(target, now);
        Some(target)
    }

    /// Step forward. The page being left stays on the back stack.
    pub fn go_forward(&mut self, current: i32, now: Instant) -> Option<i32> {
        let target = self.forward.pop_back()?;
        self.note_visit(current, now);
        self.back.push_back(current);
        trim_front(&mut self.back);
        self.note_visit(target, now);
        Some(target)
    }

    /// Up to `max` jump targets, most recently visited first, skipping `current`.
    pub fn picker_targets(&self, max: usize, current: i32) -> Vec<JumpTarget> {
        if max == 0 {
            return Vec::new();
        }
        let mut rank_of: HashMap<i32, usize> = HashMap::new();
        let mut rank = 0usize;
        for page in self.back.iter().rev().chain(self.forward.iter().rev()) {
            if *page == current {
                continue;
            }
            if rank_of.contains_key(page) {
                continue;
            }
            rank_of.insert(*page, rank);
            rank += 1;
        }
        let mut pages: Vec<i32> = rank_of.keys().copied().collect();
        pages.sort_by(|a, b| {
            let ta = self.last_visited.get(a).copied();
            let tb = self.last_visited.get(b).copied();
            tb.cmp(&ta).then_with(|| rank_of[a].cmp(&rank_of[b]))
        });
        pages.truncate(max);
        pages
            .into_iter()
            .map(|page| JumpTarget {
                page,
                last_visited: self
                    .last_visited
                    .get(&page)
                    .copied()
                    .unwrap_or_else(Instant::now),
            })
            .collect()
    }
}

fn trim_front(q: &mut VecDeque<i32>) {
    while q.len() > NAV_HISTORY_MAX {
        q.pop_front();
    }
}

/// Elapsed time since a page was last visited.
/// Under a minute (including exactly 60s) this is seconds; after that, whole minutes.
pub fn format_elapsed(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    if secs > 60 {
        format!("{} min", secs / 60)
    } else {
        format!("{secs} s")
    }
}

/// Whether `entered_at`..`now` meets the dwell threshold for recording history.
pub fn dwell_allows_record(entered_at: Instant, now: Instant, min: Duration) -> bool {
    now.duration_since(entered_at) >= min
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(base: Instant, secs: u64) -> Instant {
        base + Duration::from_secs(secs)
    }

    #[test]
    fn back_then_forward_returns_to_the_page_you_left() {
        let t = Instant::now();
        let mut h = NavHistory::default();
        h.begin_new_jump(0, at(t, 0), true);
        assert_eq!(h.go_back(5, at(t, 1)), Some(0));
        assert_eq!(h.forward.iter().copied().collect::<Vec<_>>(), vec![5]);
        assert_eq!(h.go_forward(0, at(t, 2)), Some(5));
        assert!(h.forward.is_empty());
        assert_eq!(h.go_back(5, at(t, 3)), Some(0));
        assert_eq!(h.go_back(0, at(t, 4)), None);
    }

    #[test]
    fn going_back_does_not_drop_the_forward_trail() {
        let t = Instant::now();
        let mut h = NavHistory::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.begin_new_jump(2, at(t, 1), true);
        h.begin_new_jump(4, at(t, 2), true);
        // On page 6. Two steps back must keep 6 and then 4 ahead of us.
        assert_eq!(h.go_back(6, at(t, 3)), Some(4));
        assert_eq!(h.go_back(4, at(t, 4)), Some(2));
        assert_eq!(h.back.iter().copied().collect::<Vec<_>>(), vec![0]);
        assert_eq!(h.forward.iter().copied().collect::<Vec<_>>(), vec![6, 4]);
        assert_eq!(h.go_forward(2, at(t, 5)), Some(4));
        assert_eq!(h.go_forward(4, at(t, 6)), Some(6));
        assert!(h.forward.is_empty());
        assert_eq!(h.back.iter().copied().collect::<Vec<_>>(), vec![0, 2, 4]);
    }

    #[test]
    fn new_jump_truncates_only_the_forward_tail() {
        let t = Instant::now();
        let mut h = NavHistory::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.begin_new_jump(2, at(t, 1), true);
        h.begin_new_jump(4, at(t, 2), true);
        assert_eq!(h.go_back(6, at(t, 3)), Some(4));
        assert_eq!(h.go_back(4, at(t, 4)), Some(2));
        // On 2, with 4 and 6 still ahead. Jumping to 9 abandons that tail only.
        h.begin_new_jump(2, at(t, 5), true);
        assert!(h.forward.is_empty());
        assert_eq!(h.back.iter().copied().collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(h.go_forward(9, at(t, 6)), None);
        assert_eq!(h.go_back(9, at(t, 7)), Some(2));
        assert_eq!(h.go_back(2, at(t, 8)), Some(0));
    }

    #[test]
    fn new_jump_without_dwell_still_truncates_forward_only() {
        let t = Instant::now();
        let mut h = NavHistory::default();
        h.begin_new_jump(0, at(t, 0), true);
        assert_eq!(h.go_back(3, at(t, 1)), Some(0));
        h.begin_new_jump(0, at(t, 2), false);
        assert!(h.forward.is_empty());
        assert!(h.back.is_empty());
    }

    #[test]
    fn retains_at_least_100_entries() {
        let t = Instant::now();
        let mut h = NavHistory::default();
        for i in 0..150 {
            h.begin_new_jump(i, at(t, i as u64), true);
        }
        assert_eq!(h.back.len(), NAV_HISTORY_MAX);
        assert_eq!(h.go_back(150, at(t, 200)), Some(149));
        for n in 0..(NAV_HISTORY_MAX - 1) {
            assert!(h.go_back(0, at(t, 201 + n as u64)).is_some());
        }
        assert_eq!(h.go_back(0, at(t, 400)), None);
    }

    #[test]
    fn picker_is_capped_at_nine_most_recent_pages() {
        let t = Instant::now();
        let mut h = NavHistory::default();
        for i in 0..12 {
            h.begin_new_jump(i, at(t, i as u64), true);
        }
        let targets = h.picker_targets(HISTORY_PICKER_MAX, 99);
        assert_eq!(HISTORY_PICKER_MAX, 9);
        assert_eq!(targets.len(), 9);
        let pages: Vec<i32> = targets.iter().map(|item| item.page).collect();
        assert_eq!(pages, vec![11, 10, 9, 8, 7, 6, 5, 4, 3]);
    }

    #[test]
    fn picker_skips_the_current_page_and_follows_last_visit() {
        let t = Instant::now();
        let mut h = NavHistory::default();
        h.begin_new_jump(1, at(t, 0), true);
        h.begin_new_jump(3, at(t, 10), true);
        h.begin_new_jump(5, at(t, 20), true);
        h.note_visit(1, at(t, 50));
        let targets = h.picker_targets(9, 5);
        let pages: Vec<i32> = targets.iter().map(|item| item.page).collect();
        assert_eq!(pages, vec![1, 3]);
        assert_eq!(targets[0].last_visited, at(t, 50));
        assert!(targets.iter().all(|item| item.page != 5));
    }

    #[test]
    fn forward_pages_stay_in_the_menu_after_going_back() {
        let t = Instant::now();
        let mut h = NavHistory::default();
        h.begin_new_jump(1, at(t, 0), true);
        h.begin_new_jump(3, at(t, 5), true);
        h.begin_new_jump(5, at(t, 10), true);
        assert_eq!(h.go_back(8, at(t, 15)), Some(5));
        let pages: Vec<i32> = h
            .picker_targets(9, 5)
            .iter()
            .map(|item| item.page)
            .collect();
        assert!(
            pages.contains(&8),
            "page we left must stay reachable: {pages:?}"
        );
        assert!(pages.contains(&3));
        assert!(pages.contains(&1));
    }

    #[test]
    fn elapsed_is_seconds_through_60_then_minutes() {
        assert_eq!(format_elapsed(Duration::from_secs(0)), "0 s");
        assert_eq!(format_elapsed(Duration::from_secs(12)), "12 s");
        assert_eq!(format_elapsed(Duration::from_secs(60)), "60 s");
        assert_eq!(format_elapsed(Duration::from_secs(61)), "1 min");
        assert_eq!(format_elapsed(Duration::from_secs(119)), "1 min");
        assert_eq!(format_elapsed(Duration::from_secs(180)), "3 min");
    }

    #[test]
    fn clear_empties_both_directions_and_timestamps() {
        let t = Instant::now();
        let mut h = NavHistory::default();
        h.begin_new_jump(1, at(t, 0), true);
        let _ = h.go_back(2, at(t, 1));
        h.clear();
        assert!(h.back.is_empty());
        assert!(h.forward.is_empty());
        assert!(h.last_visited.get(&1).is_none());
    }

    #[test]
    fn dwell_gate() {
        let start = Instant::now();
        assert!(!dwell_allows_record(
            start,
            start + Duration::from_secs(4),
            HISTORY_DWELL_DEFAULT
        ));
        assert!(dwell_allows_record(
            start,
            start + Duration::from_secs(5),
            HISTORY_DWELL_DEFAULT
        ));
        assert!(dwell_allows_record(
            start,
            start + Duration::from_millis(1),
            Duration::ZERO
        ));
    }

    #[test]
    fn history_menu_text_is_kept_inside_the_panel() {
        let qml = include_str!("../qml/Presenter.qml");
        let start = qml.find("id: historyOverlay").expect("history overlay");
        let end = qml[start..]
            .find("id: recentOverlay")
            .map(|offset| start + offset)
            .expect("recent overlay");
        let block = &qml[start..end];
        assert!(
            block.contains("id: historyPanel"),
            "panel must size itself, not grow with the raw text"
        );
        assert!(
            !block.contains("Math.max(historyTitle.implicitWidth, historyList.implicitWidth)"),
            "column must not be as wide as the unwrapped text"
        );
        assert!(
            block.contains("elide: Text.ElideRight"),
            "entry labels must elide instead of painting past the border"
        );
        assert!(
            block.contains("wrapMode: Text.WordWrap"),
            "hint and footer must wrap inside the panel"
        );
        assert!(
            block.contains("historyPanel.innerWidth")
                || block.contains("width: historyPanel.width")
        );
    }
}
