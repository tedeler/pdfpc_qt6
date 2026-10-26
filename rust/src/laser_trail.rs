//! Laser afterglow trail geometry helpers (page coords 0..1).

/// Default max gap between successive samples in page space.
/// ~0.4% of the page keeps the white core continuous at typical slide sizes.
pub const DEFAULT_MAX_GAP: f64 = 0.004;

/// Insert linearly interpolated points so no consecutive pair exceeds `max_gap`.
#[allow(dead_code)] // used by unit tests + available for callers
pub fn densify_points(points: &[(f64, f64)], max_gap: f64) -> Vec<(f64, f64)> {
    if points.is_empty() {
        return Vec::new();
    }
    let max_gap = max_gap.max(1.0e-9);
    let mut out = Vec::with_capacity(points.len());
    out.push(points[0]);
    for window in points.windows(2) {
        let (ax, ay) = window[0];
        let (bx, by) = window[1];
        let dx = bx - ax;
        let dy = by - ay;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist <= max_gap {
            out.push((bx, by));
            continue;
        }
        let steps = (dist / max_gap).ceil() as usize;
        for s in 1..=steps {
            let t = s as f64 / steps as f64;
            out.push((ax + dx * t, ay + dy * t));
        }
    }
    out
}

/// Append `new_pt`, inserting interpolated samples if the jump from the last
/// point exceeds `max_gap`. Near-duplicates (below `min_gap`) are ignored.
pub fn append_densified(
    stroke: &mut Vec<(f64, f64)>,
    new_pt: (f64, f64),
    max_gap: f64,
    min_gap: f64,
) {
    let Some(&(lx, ly)) = stroke.last() else {
        stroke.push(new_pt);
        return;
    };
    let dx = new_pt.0 - lx;
    let dy = new_pt.1 - ly;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < min_gap {
        return;
    }
    if dist <= max_gap {
        stroke.push(new_pt);
        return;
    }
    let steps = (dist / max_gap).ceil() as usize;
    for s in 1..=steps {
        let t = s as f64 / steps as f64;
        stroke.push((lx + dx * t, ly + dy * t));
    }
}

#[allow(dead_code)] // used by unit tests
pub fn max_segment_length(points: &[(f64, f64)]) -> f64 {
    points
        .windows(2)
        .map(|w| {
            let dx = w[1].0 - w[0].0;
            let dy = w[1].1 - w[0].1;
            (dx * dx + dy * dy).sqrt()
        })
        .fold(0.0_f64, f64::max)
}

/// RoundCap segment drawing leaves a visible gap in a core of width `core_w`
/// (same units as the points) when consecutive points are farther apart than
/// the core diameter.
#[allow(dead_code)] // used by unit tests
pub fn white_core_has_gaps(points: &[(f64, f64)], core_width: f64) -> bool {
    if points.len() < 2 {
        return false;
    }
    max_segment_length(points) > core_width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn densify_fills_sparse_horizontal_stroke() {
        let sparse = vec![(0.1, 0.5), (0.5, 0.5), (0.9, 0.5)];
        let dense = densify_points(&sparse, DEFAULT_MAX_GAP);
        assert!(dense.len() > sparse.len());
        assert!(max_segment_length(&dense) <= DEFAULT_MAX_GAP * 1.01);
        assert!(!white_core_has_gaps(&dense, DEFAULT_MAX_GAP * 2.0));
    }

    #[test]
    fn densify_preserves_already_dense_stroke() {
        let pts = densify_points(&[(0.0, 0.0), (0.002, 0.0), (0.004, 0.0)], DEFAULT_MAX_GAP);
        assert_eq!(pts.len(), 3);
        assert!(max_segment_length(&pts) <= DEFAULT_MAX_GAP);
    }

    #[test]
    fn append_densified_inserts_midpoints_on_large_jump() {
        let mut stroke = vec![(0.0, 0.5)];
        append_densified(&mut stroke, (0.2, 0.5), DEFAULT_MAX_GAP, 1.0e-6);
        assert!(stroke.len() > 2);
        assert!((stroke.last().unwrap().0 - 0.2).abs() < 1.0e-9);
        assert!(max_segment_length(&stroke) <= DEFAULT_MAX_GAP * 1.01);
    }

    #[test]
    fn append_densified_skips_tiny_jitter() {
        let mut stroke = vec![(0.5, 0.5)];
        append_densified(&mut stroke, (0.5 + 1.0e-7, 0.5), DEFAULT_MAX_GAP, 1.0e-6);
        assert_eq!(stroke.len(), 1);
    }

    #[test]
    fn sparse_stroke_would_gap_white_core() {
        // Mimics the screenshot failure mode: samples farther than the thin core.
        let sparse = vec![(0.0, 0.5), (0.05, 0.5), (0.10, 0.5)];
        let core_width = 0.01; // thin white core in page units
        assert!(white_core_has_gaps(&sparse, core_width));
        let dense = densify_points(&sparse, core_width * 0.4);
        assert!(!white_core_has_gaps(&dense, core_width));
    }

    #[test]
    fn continuous_band_predicate_holds_after_densify() {
        // After densify, every step is short enough that RoundCap capsules overlap
        // for both a thin core and a wider glow band.
        let sparse = vec![(0.05, 0.4), (0.35, 0.42), (0.7, 0.39), (0.95, 0.41)];
        let core = 0.008;
        let glow = core * 5.5;
        let dense = densify_points(&sparse, core * 0.35);
        assert!(
            !white_core_has_gaps(&dense, core),
            "white core must be continuous"
        );
        assert!(
            !white_core_has_gaps(&dense, glow),
            "red glow band must be continuous (no bead gaps)"
        );
        assert!(max_segment_length(&dense) <= core * 0.35 * 1.01);
    }
}
