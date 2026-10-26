//! Laser afterglow: one polyline (or straight segment) per mouse-drag.

/// Ignore mouse jitter smaller than this (page coords).
pub const MIN_POINT_GAP: f64 = 1.0e-4;

/// Append a vertex to a freehand polyline. Near-duplicates are skipped.
pub fn append_vertex(stroke: &mut Vec<(f64, f64)>, new_pt: (f64, f64), min_gap: f64) -> bool {
    if let Some(&(lx, ly)) = stroke.last() {
        let dx = new_pt.0 - lx;
        let dy = new_pt.1 - ly;
        if dx * dx + dy * dy < min_gap * min_gap {
            return false;
        }
    }
    stroke.push(new_pt);
    true
}

/// Keep only start→current as a straight segment (Shift+drag).
pub fn set_straight_segment(
    stroke: &mut Vec<(f64, f64)>,
    start: (f64, f64),
    current: (f64, f64),
) {
    stroke.clear();
    stroke.push(start);
    stroke.push(current);
}

/// Opacity while drawing is always 1. After release, linear fade over `ttl_secs`.
pub fn stroke_alpha(fade_started: Option<std::time::Instant>, now: std::time::Instant, ttl_secs: f64) -> Option<f64> {
    match fade_started {
        None => Some(1.0),
        Some(t0) => {
            let age = now.duration_since(t0).as_secs_f64();
            if age >= ttl_secs {
                None
            } else {
                Some(1.0 - age / ttl_secs)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn append_builds_single_polyline() {
        let mut stroke = Vec::new();
        assert!(append_vertex(&mut stroke, (0.1, 0.5), MIN_POINT_GAP));
        assert!(append_vertex(&mut stroke, (0.2, 0.5), MIN_POINT_GAP));
        assert!(append_vertex(&mut stroke, (0.35, 0.55), MIN_POINT_GAP));
        assert_eq!(stroke.len(), 3);
    }

    #[test]
    fn append_skips_jitter() {
        let mut stroke = vec![(0.5, 0.5)];
        assert!(!append_vertex(
            &mut stroke,
            (0.5 + MIN_POINT_GAP * 0.1, 0.5),
            MIN_POINT_GAP
        ));
        assert_eq!(stroke.len(), 1);
    }

    #[test]
    fn straight_segment_replaces_polyline() {
        let mut stroke = vec![(0.1, 0.1), (0.2, 0.3), (0.4, 0.2)];
        set_straight_segment(&mut stroke, (0.1, 0.1), (0.9, 0.8));
        assert_eq!(stroke, vec![(0.1, 0.1), (0.9, 0.8)]);
    }

    #[test]
    fn fade_starts_only_after_release() {
        let now = Instant::now();
        assert_eq!(stroke_alpha(None, now, 2.0), Some(1.0));
        let released = now - Duration::from_millis(500);
        let a = stroke_alpha(Some(released), now, 2.0).unwrap();
        assert!((a - 0.75).abs() < 0.01);
        let old = now - Duration::from_secs(3);
        assert_eq!(stroke_alpha(Some(old), now, 2.0), None);
    }
}
