//! Browser-style navigation history for PDF jumps.
//!
//! The trail is a line with a cursor, stored as a back stack plus a forward
//! tail. Going back or forward only moves along that line. A new jump (a PDF
//! link, go-to, or a pick from the J-menu) truncates the forward tail and
//! continues from the current position — it does not wipe the pages already
//! behind you.
//!
//! A page is added to the stack only when the caller records it: go-to, a PDF
//! link, or the explicit store key. Sequential paging does not. Walking the
//! trail still parks the page you leave on the other side, so forward survives
//! a step back.

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Maximum number of entries kept on each side of the trail.
pub const TRAIL_MAX: usize = 10;

/// Default minimum time on a page before it may enter history (5 seconds).
pub const HISTORY_DWELL_DEFAULT: Duration = Duration::from_secs(5);

/// Number of boxes in the trail strip under the preview.
pub const TRAIL_STRIP_SLOTS: usize = 10;

/// Maximum entries shown in the jump-target picker (keys `1`–`9`).
pub const HISTORY_PICKER_MAX: usize = 9;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpTarget {
    pub page: i32,
    pub last_visited: Instant,
}

/// Fixed-size view of the trail: back pages, the cursor, then forward pages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrailStrip {
    /// One entry per box, left to right. `None` is an empty box.
    pub pages: Vec<Option<i32>>,
    /// The current position sits before `pages[cursor]`: boxes left of it are
    /// Back, boxes from it onwards are Forward.
    pub cursor: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Trail {
    trail_back: VecDeque<i32>,
    trail_forward: VecDeque<i32>,
    /// When the user was last on this page (arrival or departure).
    last_visited: HashMap<i32, Instant>,
    /// Pages placed by go-to, a PDF link, or the store key.
    /// Trail walks can also leave a page in `trail_back` or `trail_forward`.
    marked: HashSet<i32>,
}

impl Trail {
    pub fn clear(&mut self) {
        self.trail_back.clear();
        self.trail_forward.clear();
        self.last_visited.clear();
        self.marked.clear();
    }

    /// True when `page` is on the stack (marked, or sitting in either trail).
    pub fn contains_page(&self, page: i32) -> bool {
        self.marked.contains(&page) || self.trail_back.contains(&page) || self.trail_forward.contains(&page)
    }

    /// Remember that `page` was visited at `now`. Does not push the stack.
    pub fn note_visit(&mut self, page: i32, now: Instant) {
        self.last_visited.insert(page, now);
    }

    /// New jump away from `from`. Drops only the forward tail.
    /// `from` is kept on the back stack when `record_from` is set
    /// (go-to or a PDF link — not a sequential step).
    pub fn begin_new_jump(&mut self, from: i32, now: Instant, record_from: bool) {
        let dropped: Vec<i32> = self.trail_forward.drain(..).collect();
        self.note_visit(from, now);
        if record_from {
            self.remember(from);
        }
        for page in dropped {
            self.forget_if_untrailed(page);
        }
    }

    /// Put `page` on the stack without moving and without dropping the forward tail.
    pub fn mark_page(&mut self, page: i32, now: Instant) {
        self.note_visit(page, now);
        self.remember(page);
    }

    pub fn toggle_mark(&mut self, page: i32, now: Instant) {
        if self.contains_page(page) {
            // Seite ist im Stack: restlos entfernen
            self.marked.remove(&page);
            self.trail_back.retain(|&p| p != page);
            self.trail_forward.retain(|&p| p != page);
        } else {
            // Seite ist nicht im Stack: speichern
            self.note_visit(page, now);
            self.remember(page);
        }
    }

    /// Step back. The page being left stays on the forward tail.
    /// A stored page that is still the current page is not a real step back.
    pub fn step_trail_back(&mut self, current: i32, now: Instant) -> Option<i32> {
        let peeled = self.trail_back.back() == Some(&current);
        if peeled {
            self.trail_back.pop_back();
        }
        let Some(target) = self.trail_back.pop_back() else {
            if peeled {
                self.trail_back.push_back(current);
            }
            return None;
        };
        self.note_visit(current, now);
        self.trail_forward.retain(|&p| p != current);
        self.trail_forward.push_back(current);
        trim_front(&mut self.trail_forward, &mut self.marked, &self.trail_back);
        self.note_visit(target, now);
        Some(target)
    }

    /// Step forward. The page being left stays on the back stack.
    pub fn step_trail_forward(&mut self, current: i32, now: Instant) -> Option<i32> {
        // Peel entries for the page we are already on (reached by stepping).
        while self.trail_forward.back() == Some(&current) {
            self.trail_forward.pop_back();
        }
        let target = self.trail_forward.pop_back()?;
        self.note_visit(current, now);
        self.trail_back.retain(|&p| p != current);
        self.trail_back.push_back(current);
        trim_front(&mut self.trail_back, &mut self.marked, &self.trail_forward);
        self.note_visit(target, now);
        Some(target)
    }

    fn remember(&mut self, page: i32) {
        self.marked.insert(page);
        if self.trail_back.back() == Some(&page) || self.trail_forward.contains(&page) {
            return;
        }
        if self.trail_back.contains(&page) {
            return;
        }
        self.trail_back.push_back(page);
        trim_front(&mut self.trail_back, &mut self.marked, &self.trail_forward);
    }

    fn forget_if_untrailed(&mut self, page: i32) {
        if !self.trail_back.contains(&page) && !self.trail_forward.contains(&page) {
            self.marked.remove(&page);
        }
    }

    /// Lay the trail out in `slots` boxes: oldest back page on the left, the
    /// nearest forward page right after the cursor. The current page itself is
    /// never a box. Unused slots on one side go to the other side; leftovers
    /// stay empty on the right.
    pub fn strip(&self, slots: usize, current: i32) -> TrailStrip {
        let back: Vec<i32> = self
            .trail_back
            .iter()
            .copied()
            .filter(|&p| p != current)
            .collect();
        let forward: Vec<i32> = self
            .trail_forward
            .iter()
            .rev()
            .copied()
            .filter(|&p| p != current)
            .collect();
        let half = slots / 2;
        let nb = back.len().min(slots.saturating_sub(forward.len().min(half)).max(half));
        let nf = forward.len().min(slots - nb);
        let mut pages: Vec<Option<i32>> = back[back.len() - nb..].iter().map(|&p| Some(p)).collect();
        pages.extend(forward[..nf].iter().map(|&p| Some(p)));
        pages.resize(slots, None);
        TrailStrip { pages, cursor: nb }
    }

    /// Up to `max` jump targets, most recently visited first, skipping `current`.
    pub fn picker_targets(&self, max: usize, current: i32) -> Vec<JumpTarget> {
        if max == 0 {
            return Vec::new();
        }
        let mut rank_of: HashMap<i32, usize> = HashMap::new();
        let mut rank = 0usize;
        for page in self.trail_back.iter().rev().chain(self.trail_forward.iter().rev()) {
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

fn trim_front(q: &mut VecDeque<i32>, marked: &mut HashSet<i32>, other: &VecDeque<i32>) {
    while q.len() > TRAIL_MAX {
        let Some(page) = q.pop_front() else {
            break;
        };
        if !q.contains(&page) && !other.contains(&page) {
            marked.remove(&page);
        }
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
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        assert_eq!(h.step_trail_back(5, at(t, 1)), Some(0));
        assert_eq!(h.trail_forward.iter().copied().collect::<Vec<_>>(), vec![5]);
        assert_eq!(h.step_trail_forward(0, at(t, 2)), Some(5));
        assert!(h.trail_forward.is_empty());
        assert_eq!(h.step_trail_back(5, at(t, 3)), Some(0));
        assert_eq!(h.step_trail_back(0, at(t, 4)), None);
    }

    #[test]
    fn going_back_does_not_drop_the_forward_trail() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.begin_new_jump(2, at(t, 1), true);
        h.begin_new_jump(4, at(t, 2), true);
        // On page 6. Two steps back must keep 6 and then 4 ahead of us.
        assert_eq!(h.step_trail_back(6, at(t, 3)), Some(4));
        assert_eq!(h.step_trail_back(4, at(t, 4)), Some(2));
        assert_eq!(h.trail_back.iter().copied().collect::<Vec<_>>(), vec![0]);
        assert_eq!(h.trail_forward.iter().copied().collect::<Vec<_>>(), vec![6, 4]);
        assert_eq!(h.step_trail_forward(2, at(t, 5)), Some(4));
        assert_eq!(h.step_trail_forward(4, at(t, 6)), Some(6));
        assert!(h.trail_forward.is_empty());
        assert_eq!(h.trail_back.iter().copied().collect::<Vec<_>>(), vec![0, 2, 4]);
    }

    #[test]
    fn new_jump_truncates_only_the_forward_tail() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.begin_new_jump(2, at(t, 1), true);
        h.begin_new_jump(4, at(t, 2), true);
        assert_eq!(h.step_trail_back(6, at(t, 3)), Some(4));
        assert_eq!(h.step_trail_back(4, at(t, 4)), Some(2));
        // On 2, with 4 and 6 still ahead. Jumping to 9 abandons that tail only.
        h.begin_new_jump(2, at(t, 5), true);
        assert!(h.trail_forward.is_empty());
        assert_eq!(h.trail_back.iter().copied().collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(h.step_trail_forward(9, at(t, 6)), None);
        assert_eq!(h.step_trail_back(9, at(t, 7)), Some(2));
        assert_eq!(h.step_trail_back(2, at(t, 8)), Some(0));
    }

    #[test]
    fn new_jump_without_dwell_still_truncates_forward_only() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        assert_eq!(h.step_trail_back(3, at(t, 1)), Some(0));
        h.begin_new_jump(0, at(t, 2), false);
        assert!(h.trail_forward.is_empty());
        assert!(h.trail_back.is_empty());
    }

    #[test]
    fn keeps_only_the_last_10_entries() {
        let t = Instant::now();
        let mut h = Trail::default();
        for i in 0..30 {
            h.begin_new_jump(i, at(t, i as u64), true);
        }
        assert_eq!(h.trail_back.len(), TRAIL_MAX);
        assert_eq!(h.step_trail_back(30, at(t, 200)), Some(29));
        for n in 0..(TRAIL_MAX - 1) {
            assert!(h.step_trail_back(0, at(t, 201 + n as u64)).is_some());
        }
        assert_eq!(h.step_trail_back(0, at(t, 400)), None);
    }

    #[test]
    fn picker_is_capped_at_nine_most_recent_pages() {
        let t = Instant::now();
        let mut h = Trail::default();
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
        let mut h = Trail::default();
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
        let mut h = Trail::default();
        h.begin_new_jump(1, at(t, 0), true);
        h.begin_new_jump(3, at(t, 5), true);
        h.begin_new_jump(5, at(t, 10), true);
        assert_eq!(h.step_trail_back(8, at(t, 15)), Some(5));
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

    // Scenario tests. Notation: `back [..]` is trail_back (oldest first),
    // `fwd [..]` is trail_forward (next page last).

    fn v(q: &VecDeque<i32>) -> Vec<i32> {
        q.iter().copied().collect()
    }

    #[test]
    fn scenario_goto_chain_then_walk_back_and_forward() {
        let t = Instant::now();
        let mut h = Trail::default();
        // Start on 0, g→10, g→20, g→30.
        h.begin_new_jump(0, at(t, 0), true);
        h.begin_new_jump(10, at(t, 1), true);
        h.begin_new_jump(20, at(t, 2), true);
        assert_eq!(v(&h.trail_back), vec![0, 10, 20]);
        // Ctrl+← from 30 → 20 → 10; 30 and 20 wait ahead.
        assert_eq!(h.step_trail_back(30, at(t, 3)), Some(20));
        assert_eq!(h.step_trail_back(20, at(t, 4)), Some(10));
        assert_eq!(v(&h.trail_back), vec![0]);
        assert_eq!(v(&h.trail_forward), vec![30, 20]);
        // Ctrl+→ retraces exactly.
        assert_eq!(h.step_trail_forward(10, at(t, 5)), Some(20));
        assert_eq!(h.step_trail_forward(20, at(t, 6)), Some(30));
        assert_eq!(h.step_trail_forward(30, at(t, 7)), None);
    }

    #[test]
    fn scenario_forward_after_stepping_onto_the_next_page_is_a_noop() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        assert_eq!(h.step_trail_back(5, at(t, 1)), Some(0));
        // User pages 0→…→5 by hand, then presses Ctrl+→: nothing to walk to.
        assert_eq!(h.step_trail_forward(5, at(t, 2)), None);
        assert!(h.trail_forward.is_empty());
        assert!(h.trail_back.is_empty());
    }

    #[test]
    fn scenario_walking_never_duplicates_pages() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.begin_new_jump(2, at(t, 1), true);
        for _ in 0..3 {
            assert_eq!(h.step_trail_back(2, at(t, 2)), Some(0));
            assert_eq!(h.step_trail_forward(0, at(t, 3)), Some(2));
        }
        assert_eq!(v(&h.trail_back), vec![0]);
        assert!(h.trail_forward.is_empty());
    }

    #[test]
    fn scenario_marking_a_page_ahead_keeps_forward_unique() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.begin_new_jump(1, at(t, 1), true);
        assert_eq!(h.step_trail_back(6, at(t, 2)), Some(1));
        // Paged to 6 by hand and marked it with `m`, then Ctrl+←.
        h.mark_page(6, at(t, 3));
        assert_eq!(h.step_trail_back(6, at(t, 4)), Some(0));
        assert_eq!(v(&h.trail_forward), vec![6]);
    }

    #[test]
    fn scenario_menu_jump_drops_forward_but_does_not_mark_the_page_left() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.begin_new_jump(4, at(t, 1), true);
        assert_eq!(h.step_trail_back(8, at(t, 2)), Some(4));
        h.begin_new_jump(4, at(t, 3), false);
        assert!(h.trail_forward.is_empty());
        assert_eq!(v(&h.trail_back), vec![0]);
        assert!(!h.contains_page(8));
    }

    #[test]
    fn scenario_toggle_mark_adds_and_removes_everywhere() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.toggle_mark(3, at(t, 0));
        assert!(h.contains_page(3));
        assert_eq!(v(&h.trail_back), vec![3]);
        h.toggle_mark(3, at(t, 1));
        assert!(!h.contains_page(3));
        assert!(h.trail_back.is_empty());
    }

    #[test]
    fn scenario_back_on_the_only_marked_page_stays_put() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.mark_page(3, at(t, 0));
        assert_eq!(h.step_trail_back(3, at(t, 1)), None);
        assert!(h.contains_page(3));
        assert!(h.trail_forward.is_empty());
    }

    #[test]
    fn scenario_clear_forgets_marks_trail_and_picker_entries() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.mark_page(4, at(t, 1));
        assert_eq!(h.step_trail_back(7, at(t, 2)), Some(4));
        h.clear();
        assert!(!h.contains_page(0) && !h.contains_page(4) && !h.contains_page(7));
        assert!(h.picker_targets(9, 1).is_empty());
        assert_eq!(h.step_trail_back(1, at(t, 3)), None);
        assert_eq!(h.step_trail_forward(1, at(t, 4)), None);
    }

    #[test]
    fn history_menu_legend_names_ctrl_x() {
        let qml = include_str!("../qml/Presenter.qml");
        assert!(qml.contains("Ctrl+X clears all"));
        let session = include_str!("session.rs");
        assert!(session.contains("ctrl && key_letter(key) == Some('x')"));
    }

    fn strip_of(h: &Trail, current: i32) -> (Vec<i32>, usize) {
        let s = h.strip(TRAIL_STRIP_SLOTS, current);
        assert_eq!(s.pages.len(), TRAIL_STRIP_SLOTS);
        (s.pages.iter().map(|p| p.unwrap_or(-1)).collect(), s.cursor)
    }

    #[test]
    fn strip_empty_trail_has_ten_empty_boxes_and_cursor_at_left() {
        let h = Trail::default();
        let (pages, cursor) = strip_of(&h, 0);
        assert_eq!(pages, vec![-1; 10]);
        assert_eq!(cursor, 0);
    }

    #[test]
    fn strip_back_left_of_cursor_forward_right_of_it() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.begin_new_jump(2, at(t, 1), true);
        h.begin_new_jump(4, at(t, 2), true);
        // On 6, back = [0,2,4], forward empty: arrow right of the last Back box.
        let (pages, cursor) = strip_of(&h, 6);
        assert_eq!(&pages[..4], &[0, 2, 4, -1]);
        assert_eq!(cursor, 3);
        // Two steps back: on 2, forward holds 4 then 6 (nearest first).
        assert_eq!(h.step_trail_back(6, at(t, 3)), Some(4));
        assert_eq!(h.step_trail_back(4, at(t, 4)), Some(2));
        let (pages, cursor) = strip_of(&h, 2);
        assert_eq!(&pages[..4], &[0, 4, 6, -1]);
        assert_eq!(cursor, 1);
    }

    #[test]
    fn strip_never_shows_the_current_page() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.mark_page(3, at(t, 0));
        let (pages, cursor) = strip_of(&h, 3);
        assert_eq!(pages, vec![-1; 10]);
        assert_eq!(cursor, 0);
    }

    #[test]
    fn strip_full_trail_fits_exactly_ten_boxes() {
        let t = Instant::now();
        let mut h = Trail::default();
        for i in 0..30 {
            h.begin_new_jump(i, at(t, i as u64), true);
        }
        let (pages, cursor) = strip_of(&h, 99);
        assert_eq!(pages, (20..30).collect::<Vec<_>>());
        assert_eq!(cursor, 10);
        // Walk back 8 to page 22: two Back boxes, the rest of the strip is Forward.
        let mut cur = 99;
        for n in 0..8 {
            cur = h.step_trail_back(cur, at(t, 100 + n)).unwrap();
        }
        assert_eq!(cur, 22);
        let (pages, cursor) = strip_of(&h, cur);
        assert_eq!(pages, vec![20, 21, 23, 24, 25, 26, 27, 28, 29, 99]);
        assert_eq!(cursor, 2);
    }

    #[test]
    fn trail_strip_sits_between_preview_and_status_bar() {
        let qml = include_str!("../qml/Presenter.qml");
        let top = qml.find("id: topRow").expect("preview row");
        let strip = qml.find("id: trailStrip").expect("trail strip");
        let bar = qml.find("id: bottomBar").expect("status bar");
        assert!(top < strip && strip < bar);
        assert!(qml.contains("readonly property int slots: 10"));
        assert!(qml.contains("width: parent.width"));
        assert!(qml.contains("trail_strip_json"));
        assert!(qml.contains("trailStrip.height"));
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
        let mut h = Trail::default();
        h.begin_new_jump(1, at(t, 0), true);
        let _ = h.step_trail_back(2, at(t, 1));
        h.clear();
        assert!(h.trail_back.is_empty());
        assert!(h.trail_forward.is_empty());
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

    #[test]
    fn a_visit_alone_does_not_push_the_stack() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.note_visit(0, at(t, 0));
        h.note_visit(1, at(t, 1));
        assert!(!h.contains_page(0));
        assert!(!h.contains_page(1));
        assert!(h.picker_targets(9, 1).is_empty());
        assert_eq!(h.step_trail_back(1, at(t, 2)), None);
    }

    #[test]
    fn store_page_marks_it_and_keeps_the_forward_tail() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.begin_new_jump(2, at(t, 1), true);
        assert_eq!(h.step_trail_back(4, at(t, 2)), Some(2));
        assert_eq!(h.trail_forward.iter().copied().collect::<Vec<_>>(), vec![4]);
        h.mark_page(2, at(t, 3));
        assert!(h.contains_page(2));
        assert_eq!(h.trail_forward.iter().copied().collect::<Vec<_>>(), vec![4]);
        assert_eq!(h.step_trail_forward(2, at(t, 4)), Some(4));
    }

    #[test]
    fn store_while_still_on_the_page_is_not_a_back_step() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(0, at(t, 0), true);
        h.mark_page(3, at(t, 1));
        assert!(h.contains_page(3));
        assert_eq!(h.step_trail_back(3, at(t, 2)), Some(0));
        assert!(h.contains_page(3));
        assert_eq!(h.trail_forward.iter().copied().collect::<Vec<_>>(), vec![3]);
        assert_eq!(h.step_trail_forward(0, at(t, 3)), Some(3));
    }

    #[test]
    fn stepping_away_after_store_does_not_push_the_new_page() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.mark_page(1, at(t, 0));
        // A sequential leave only notes the visit. It does not call begin_new_jump.
        h.note_visit(2, at(t, 1));
        assert!(h.contains_page(1));
        assert!(!h.contains_page(2));
        let pages: Vec<i32> = h
            .picker_targets(9, 2)
            .iter()
            .map(|item| item.page)
            .collect();
        assert_eq!(pages, vec![1]);
    }

    #[test]
    fn goto_records_the_left_page_without_a_dwell() {
        let t = Instant::now();
        let mut h = Trail::default();
        h.begin_new_jump(4, at(t, 0), true);
        assert!(h.contains_page(4));
        assert!(!h.contains_page(9));
        assert_eq!(h.trail_back.iter().copied().collect::<Vec<_>>(), vec![4]);
    }

    #[test]
    fn stack_tag_matches_the_laser_badge() {
        let qml = include_str!("../qml/Presenter.qml");
        let laser = qml.find("label: \"LASER\"").expect("laser badge");
        let stack = qml.find("label: \"STACK\"").expect("stack badge");
        assert!(
            stack > laser && stack - laser < 400,
            "STACK must sit in the same badge row as LASER"
        );
        assert!(
            !qml.contains("#e6a23c") && !qml.contains("id: stackMark"),
            "the amber on-slide mark is gone"
        );
        assert!(qml.contains("current_on_stack"));
        assert!(
            qml.contains("Mark this slide on the jump stack"),
            "help overlay must document the store key"
        );
    }

    #[test]
    fn help_overlay_scrolls_inside_the_window() {
        let qml = include_str!("../qml/Presenter.qml");
        let start = qml.find("id: helpOverlay").expect("help overlay");
        let end = qml[start..]
            .find("id: historyOverlay")
            .map(|offset| start + offset)
            .expect("history overlay");
        let block = &qml[start..end];
        assert!(
            block.contains("id: helpFlick"),
            "help list needs a flickable"
        );
        assert!(block.contains("ScrollBar.vertical"));
        assert!(
            block.contains("ScrollBar.AlwaysOn"),
            "scrollbar stays visible when the list is taller than the panel"
        );
        assert!(
            block.contains("Math.min(root.height - 32, helpColumn.implicitHeight"),
            "panel height must stay inside the window"
        );
        assert!(block.contains("interactive: contentHeight > height"));
    }
}
