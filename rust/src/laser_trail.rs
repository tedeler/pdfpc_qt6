//! Laser afterglow: one polyline per mouse-drag, faded as a whole stroke.

/// Ignore mouse jitter smaller than this (page coords).
pub const MIN_POINT_GAP: f64 = 1.0e-4;

/// Append a vertex to the active polyline. Near-duplicates are skipped so we
/// only store a clean polygon chain while the button is held.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_builds_single_polyline() {
        let mut stroke = Vec::new();
        assert!(append_vertex(&mut stroke, (0.1, 0.5), MIN_POINT_GAP));
        assert!(append_vertex(&mut stroke, (0.2, 0.5), MIN_POINT_GAP));
        assert!(append_vertex(&mut stroke, (0.35, 0.55), MIN_POINT_GAP));
        assert_eq!(stroke.len(), 3);
        assert_eq!(stroke[0], (0.1, 0.5));
        assert_eq!(stroke[2], (0.35, 0.55));
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
    fn polyline_needs_two_points_to_draw() {
        let mut stroke = Vec::new();
        append_vertex(&mut stroke, (0.1, 0.2), MIN_POINT_GAP);
        assert!(stroke.len() < 2, "single vertex is not a drawable polyline");
        append_vertex(&mut stroke, (0.3, 0.4), MIN_POINT_GAP);
        assert!(stroke.len() >= 2);
    }
}
