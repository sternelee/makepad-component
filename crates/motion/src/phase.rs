//! Loader math — the pure phase functions behind the loading indicators.
//!
//! These are the curves and constants the widgets animate with, kept pure so
//! any surface animates the *same* loaders rather than inventing its own
//! spinner. A loading indicator is a brand surface; two of them that disagree
//! read as two products.
//!
//! Everything is a pure function of a phase in `0..1`, so a caller can drive it
//! from a frame delta or from wall-clock elapsed time and get identical output.

/// Pulse loader period.
pub const PULSE_MS: u64 = 2_400;
/// Gradient matrix spinner wave period.
pub const GRADIENT_SPIN_MS: u64 = 750;

/// Cells in the pulse wave loader.
pub const PULSE_CELLS: usize = 5;
/// Side length of the gradient spinner matrix.
pub const MATRIX_SIDE: usize = 3;

/// Pulse loader cells rest at this opacity between pulses.
pub const PULSE_MIN_OPACITY: f32 = 0.08;
/// …and at this scale.
pub const PULSE_MIN_SCALE: f32 = 0.9;
/// Per-cell stagger, as a fraction of the pulse period (0.15s of 2.4s).
pub const PULSE_STAGGER: f32 = 0.15 / 2.4;

/// Per-row tints of the gradient matrix spinner — a "sunrise" gradient sampled
/// at each row: cool blue at the top, through amber, to pink.
pub const GSPIN_ROW_TINTS: [u32; MATRIX_SIDE] = [0xB6D3EF, 0xEDB185, 0xF888A0];
/// Opacity a gradient-spinner cell rests at between pulses.
pub const GSPIN_DIM: f32 = 0.1;

/// Clockwise ring position of each `(row, col)` cell of the 2×3 mini spinner,
/// top-left first. Every cell of a 2×3 grid is on the ring, so the brightness
/// chases around it.
pub const MINI_RING: [[usize; 2]; 3] = [[0, 1], [5, 2], [4, 3]];
/// Cells in the mini spinner's ring.
pub const MINI_RING_LEN: f32 = 6.0;

/// Orb period. Slower than either spinner: these breathe rather than tick, and
/// a tick at this size reads as impatience.
pub const ORB_MS: u64 = 2_000;

/// Orbs in the [`ORB_SEATS`] cluster.
pub const ORBS: usize = 3;
/// Where each cluster orb sits, as `(x, y)` fractions of the box — a triangle,
/// so the group reads as one object and still fills a square slot.
pub const ORB_SEATS: [(f32, f32); ORBS] = [(0.36, 0.36), (0.64, 0.32), (0.48, 0.66)];
/// A cluster orb's diameter at its smallest and largest, as fractions of the
/// box. The swing is the whole point: at the bottom an orb is nearly gone, so
/// the *count* you perceive changes as they trade places. A fixed size would
/// leave the silhouette constant and the thing would read as three dots dimming.
pub const ORB_MIN_SIZE: f32 = 0.14;
pub const ORB_MAX_SIZE: f32 = 0.62;
/// How far a cluster orb wanders from its seat, as a fraction of the box.
pub const ORB_DRIFT: f32 = 0.13;
/// An orb never goes out entirely — the cluster dims, it does not blink.
pub const ORB_MIN_OPACITY: f32 = 0.35;
/// Glow radius at rest and at full breath, as fractions of the box.
pub const ORB_GLOW_MIN: f32 = 0.10;
pub const ORB_GLOW_MAX: f32 = 0.40;

/// Dots in the [`orb_ring_seat`] ring.
pub const ORB_RING_DOTS: usize = 8;
/// The ring's radius and its dot diameter, as fractions of the box.
pub const ORB_RING_RADIUS: f32 = 0.34;
pub const ORB_RING_DOT: f32 = 0.16;

/// Rings in flight at once in the bloom.
pub const ORB_BLOOM_RINGS: usize = 3;
/// Where a bloom ring starts and ends, as fractions of the box.
pub const ORB_BLOOM_MIN: f32 = 0.16;
pub const ORB_BLOOM_MAX: f32 = 1.0;

/// How many cells the mini spinner has.
pub const MINI_CELLS: usize = 6;

// The swing has to be visible or the cluster is three dots dimming: an orb at
// the trough is at most a quarter the diameter of one at the crest. A compile
// error rather than a test, since both sides are constants.
const _: () = assert!(ORB_MAX_SIZE > ORB_MIN_SIZE * 4.0);
// The ring's dot must fit inside its own radius, or the ring is a blob.
const _: () = assert!(ORB_RING_DOT <= ORB_RING_RADIUS);
// The mini spinner's ring table must cover every cell exactly once.
const _: () = assert!(MINI_RING.len() * MINI_RING[0].len() == MINI_CELLS);

/// Linear interpolation.
pub fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

/// Cosine pulse: 0 at phase 0, 1 at phase 0.5, back to 0 at phase 1.
pub fn pulse_wave(phase: f32) -> f32 {
    0.5 - 0.5 * (phase * std::f32::consts::TAU).cos()
}

/// A cell's phase, given the loader's raw phase and the cell's index.
pub fn staggered_phase(raw_delta: f32, index: usize, stagger: f32) -> f32 {
    (raw_delta - index as f32 * stagger).rem_euclid(1.0)
}

/// Pulse loader cell opacity for a phase: 0.08 → 1 → 0.08.
pub fn pulse_opacity(phase: f32) -> f32 {
    PULSE_MIN_OPACITY + (1.0 - PULSE_MIN_OPACITY) * pulse_wave(phase)
}

/// Pulse loader cell scale for a phase: 0.9 → 1 → 0.9.
pub fn pulse_scale(phase: f32) -> f32 {
    PULSE_MIN_SCALE + (1.0 - PULSE_MIN_SCALE) * pulse_wave(phase)
}

/// Gradient-spin cell opacity for a local phase `t` (0..1 of the period),
/// ported from the `gradient-spin-pulse` keyframes: full at the cycle start,
/// easing down to `dim` by 45%, resting at `dim` until 92%, then rising back to
/// full — the per-cell phase offset sweeps this pulse across the grid.
pub fn gspin_opacity(t: f32, dim: f32) -> f32 {
    let t = t.rem_euclid(1.0);
    if t < 0.45 {
        lerp(1.0, dim, t / 0.45)
    } else if t < 0.92 {
        dim
    } else {
        lerp(dim, 1.0, (t - 0.92) / 0.08)
    }
}

/// The phase offset of a `(row, col)` cell in the 3×3 gradient spinner: the
/// pulse enters at the bottom edge and converges toward the top-centre cell, so
/// the wave reads as travelling upward.
pub fn gspin_cell_phase(row: usize, col: usize) -> f32 {
    let centre = (MATRIX_SIDE as f32 - 1.0) / 2.0;
    let max = MATRIX_SIDE as f32 - 1.0 + centre;
    let d = MATRIX_SIDE as f32 - 1.0 - row as f32 + (col as f32 - centre).abs();
    if max == 0.0 {
        0.0
    } else {
        d / (max + 1.0)
    }
}

/// One cluster orb's opacity as it breathes: [`ORB_MIN_OPACITY`] → 1 → back.
pub fn orb_opacity(phase: f32) -> f32 {
    lerp(ORB_MIN_OPACITY, 1.0, pulse_wave(phase))
}

/// One cluster orb's diameter, as a fraction of the box.
pub fn orb_size(phase: f32) -> f32 {
    lerp(ORB_MIN_SIZE, ORB_MAX_SIZE, pulse_wave(phase))
}

/// One cluster orb's glow radius, as a fraction of the box. In step with the
/// opacity, because a glow that peaks off-beat reads as two lights rather than
/// one breathing.
pub fn orb_glow(phase: f32) -> f32 {
    lerp(ORB_GLOW_MIN, ORB_GLOW_MAX, pulse_wave(phase))
}

/// How far a cluster orb has drifted from its seat, as `(dx, dy)` fractions of
/// the box — a small circle walked once per period.
///
/// Being a circle, the drift returns exactly to zero every period, so nothing
/// accumulates however long it runs.
pub fn orb_drift(phase: f32) -> (f32, f32) {
    let angle = phase * std::f32::consts::TAU;
    (ORB_DRIFT * angle.cos(), ORB_DRIFT * angle.sin())
}

/// Where ring dot `index` sits on a circle of `radius`, as `(x, y)` fractions
/// of the box. Twelve o'clock first, going clockwise.
///
/// The radius is an argument because two shapes want the same circle: the ring
/// holds it still and the converge pulses it. Two functions here would be one
/// function and a number.
pub fn orb_ring_seat(index: usize, radius: f32) -> (f32, f32) {
    let angle = index as f32 / ORB_RING_DOTS as f32 * std::f32::consts::TAU
        - std::f32::consts::FRAC_PI_2;
    (0.5 + radius * angle.cos(), 0.5 + radius * angle.sin())
}

/// The converge's radius at `phase`: in to nothing, back out to the ring.
///
/// Every dot shares this one radius, so they arrive at the centre together and
/// stack into a single point — the frame that makes this read as a gathering
/// rather than as a ring that shrank.
pub fn orb_converge_radius(phase: f32) -> f32 {
    lerp(0.0, ORB_RING_RADIUS, pulse_wave(phase))
}

/// A bloom ring's radius, as a fraction of the box: out from
/// [`ORB_BLOOM_MIN`] to [`ORB_BLOOM_MAX`] once per period.
pub fn orb_bloom_radius(phase: f32) -> f32 {
    lerp(ORB_BLOOM_MIN, ORB_BLOOM_MAX, phase.rem_euclid(1.0))
}

/// A bloom ring's opacity: full as it leaves the centre, gone by the edge —
/// squared, so it holds its brightness through the middle of the travel and
/// then goes quickly, which is what keeps the ring from looking like a
/// dissolving circle.
pub fn orb_bloom_opacity(phase: f32) -> f32 {
    let t = 1.0 - phase.rem_euclid(1.0);
    t * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pulse_wave_spans_zero_to_one_and_back() {
        assert!(pulse_wave(0.0).abs() < 1e-6);
        assert!((pulse_wave(0.5) - 1.0).abs() < 1e-6);
        assert!(pulse_wave(1.0).abs() < 1e-6);
        // Monotone up the first half, monotone down the second.
        for i in 0..50 {
            let a = pulse_wave(i as f32 / 100.0);
            let b = pulse_wave((i + 1) as f32 / 100.0);
            assert!(b >= a - 1e-6);
        }
    }

    #[test]
    fn test_pulse_wave_is_periodic() {
        for p in [0.0f32, 0.13, 0.4, 0.77] {
            assert!((pulse_wave(p) - pulse_wave(p + 4.0)).abs() < 1e-4, "{p}");
        }
    }

    #[test]
    fn test_loader_ranges_are_inside_the_unit_interval() {
        for i in 0..=100 {
            let p = i as f32 / 100.0;
            for v in [
                pulse_opacity(p),
                pulse_scale(p),
                orb_opacity(p),
                orb_size(p),
                orb_glow(p),
                orb_converge_radius(p),
                orb_bloom_opacity(p),
            ] {
                assert!((0.0..=1.0).contains(&v), "phase {p}: {v}");
            }
        }
    }

    #[test]
    fn test_pulse_extremes_hit_their_measured_rest_values() {
        assert!((pulse_opacity(0.0) - PULSE_MIN_OPACITY).abs() < 1e-6);
        assert!((pulse_opacity(0.5) - 1.0).abs() < 1e-6);
        assert!((pulse_scale(0.0) - PULSE_MIN_SCALE).abs() < 1e-6);
        assert!((pulse_scale(0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_stagger_offsets_cells_by_the_stagger_fraction() {
        let a = staggered_phase(0.5, 0, PULSE_STAGGER);
        let b = staggered_phase(0.5, 1, PULSE_STAGGER);
        assert!((a - 0.5).abs() < 1e-6);
        assert!((b - (0.5 - PULSE_STAGGER)).abs() < 1e-6);
    }

    #[test]
    fn test_stagger_wraps_rather_than_going_negative() {
        let p = staggered_phase(0.0, 4, PULSE_STAGGER);
        assert!((0.0..1.0).contains(&p), "{p}");
    }

    #[test]
    fn test_every_pulse_cell_gets_a_distinct_phase() {
        // Five cells at the same phase would be one dot, not a wave.
        let phases: Vec<f32> = (0..PULSE_CELLS)
            .map(|i| staggered_phase(0.0, i, PULSE_STAGGER))
            .collect();
        for (i, a) in phases.iter().enumerate() {
            for b in phases.iter().skip(i + 1) {
                assert!((a - b).abs() > 1e-3, "{phases:?}");
            }
        }
    }

    #[test]
    fn test_gspin_opacity_is_full_at_the_cycle_start() {
        assert!((gspin_opacity(0.0, GSPIN_DIM) - 1.0).abs() < 1e-6);
        assert!((gspin_opacity(1.0, GSPIN_DIM) - 1.0).abs() < 1e-6);
        assert!((gspin_opacity(0.7, GSPIN_DIM) - GSPIN_DIM).abs() < 1e-6);
    }

    #[test]
    fn test_gspin_opacity_never_dips_below_its_dim_floor() {
        for i in 0..=100 {
            let v = gspin_opacity(i as f32 / 100.0, GSPIN_DIM);
            assert!(v >= GSPIN_DIM - 1e-6 && v <= 1.0 + 1e-6, "{v}");
        }
    }

    #[test]
    fn test_gspin_wave_travels_upward() {
        // The bottom row must lead the top row, or the wave reads as falling.
        let bottom = gspin_cell_phase(2, 1);
        let top = gspin_cell_phase(0, 1);
        assert!(bottom < top, "bottom {bottom} top {top}");
    }

    #[test]
    fn test_gspin_phases_stay_in_the_unit_interval() {
        for row in 0..MATRIX_SIDE {
            for col in 0..MATRIX_SIDE {
                let p = gspin_cell_phase(row, col);
                assert!((0.0..1.0).contains(&p), "{row},{col}: {p}");
            }
        }
    }

    #[test]
    fn test_mini_ring_covers_every_cell_exactly_once() {
        let mut seen: Vec<usize> = MINI_RING.iter().flat_map(|r| r.iter().copied()).collect();
        seen.sort_unstable();
        assert_eq!(seen, (0..MINI_CELLS).collect::<Vec<_>>());
    }

    #[test]
    fn test_orb_drift_returns_to_its_seat_every_period() {
        // Nothing may accumulate however long the loader runs.
        for seat in [(0.36f32, 0.36f32), (0.64, 0.32)] {
            let (x0, y0) = orb_drift(0.0);
            let (x1, y1) = orb_drift(1.0);
            assert!((x0 - x1).abs() < 1e-4 && (y0 - y1).abs() < 1e-4);
            assert!(x0.is_finite() && y0.is_finite());
            let _ = seat;
        }
    }

    #[test]
    fn test_orb_drift_stays_inside_the_box() {
        for i in 0..=100 {
            let (dx, dy) = orb_drift(i as f32 / 100.0);
            assert!(dx.abs() <= ORB_DRIFT + 1e-6, "{dx}");
            assert!(dy.abs() <= ORB_DRIFT + 1e-6, "{dy}");
        }
    }

    #[test]
    fn test_orb_seats_are_inside_the_box_and_distinct() {
        for (x, y) in ORB_SEATS {
            assert!((0.0..1.0).contains(&x) && (0.0..1.0).contains(&y));
        }
        for (i, a) in ORB_SEATS.iter().enumerate() {
            for b in ORB_SEATS.iter().skip(i + 1) {
                assert!(a != b, "{ORB_SEATS:?}");
            }
        }
    }

    #[test]
    fn test_ring_seat_puts_the_first_dot_at_twelve_o_clock() {
        let (x, y) = orb_ring_seat(0, ORB_RING_RADIUS);
        assert!((x - 0.5).abs() < 1e-5, "{x}");
        assert!(y < 0.5, "twelve o'clock is above centre: {y}");
    }

    #[test]
    fn test_every_ring_dot_lands_on_the_same_circle() {
        for i in 0..ORB_RING_DOTS {
            let (x, y) = orb_ring_seat(i, ORB_RING_RADIUS);
            let r = ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt();
            assert!((r - ORB_RING_RADIUS).abs() < 1e-5, "dot {i}: {r}");
        }
    }

    #[test]
    fn test_converge_meets_at_the_centre_and_returns_to_the_ring() {
        assert!(orb_converge_radius(0.0).abs() < 1e-6);
        assert!(orb_converge_radius(1.0).abs() < 1e-6);
        assert!((orb_converge_radius(0.5) - ORB_RING_RADIUS).abs() < 1e-6);
    }

    #[test]
    fn test_bloom_rings_are_ordered_outward_and_dim_edge_first() {
        // Three rings in flight at once are offset by a third of the period.
        let radii: Vec<f32> = (0..ORB_BLOOM_RINGS)
            .map(|i| orb_bloom_radius(i as f32 / ORB_BLOOM_RINGS as f32))
            .collect();
        for w in radii.windows(2) {
            assert!(w[0] < w[1], "{radii:?}");
        }
        // The one nearest the edge is the faintest.
        let opacities: Vec<f32> = (0..ORB_BLOOM_RINGS)
            .map(|i| orb_bloom_opacity(i as f32 / ORB_BLOOM_RINGS as f32))
            .collect();
        assert!(opacities[0] > opacities[1] && opacities[1] > opacities[2]);
    }

    #[test]
    fn test_bloom_opacity_is_full_at_the_centre_and_gone_at_the_edge() {
        assert!((orb_bloom_opacity(0.0) - 1.0).abs() < 1e-6);
        // Just short of the edge the ring has nearly faded out. The value at
        // phase 1.0 is the *next* ring being born at the centre, not this one
        // dying at the rim — the function is periodic, not ranged.
        assert!(orb_bloom_opacity(0.99) < 1e-3, "{}", orb_bloom_opacity(0.99));
        assert!((orb_bloom_opacity(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_endpoints_and_midpoint() {
        assert_eq!(lerp(2.0, 6.0, 0.0), 2.0);
        assert_eq!(lerp(2.0, 6.0, 1.0), 6.0);
        assert_eq!(lerp(2.0, 6.0, 0.5), 4.0);
    }

    #[test]
    fn test_loaders_are_finite_for_negative_and_large_phases() {
        // A phase can arrive as elapsed-seconds divided by a period, and a
        // clock that steps backwards would make it negative.
        for p in [-3.7f32, -0.1, 17.3, 1000.0] {
            for v in [
                pulse_wave(p),
                pulse_opacity(p),
                orb_opacity(p),
                orb_size(p),
                orb_converge_radius(p),
                orb_bloom_radius(p),
                orb_bloom_opacity(p),
                gspin_opacity(p, GSPIN_DIM),
            ] {
                assert!(v.is_finite(), "phase {p}");
                assert!((0.0..=1.0).contains(&v), "phase {p}: {v}");
            }
        }
    }
}
