use super::Zoom;

#[test]
fn larger_and_smaller_step_by_a_tenth() {
    let zoom = Zoom::default();

    assert!((zoom.factor() - 1.0).abs() < f32::EPSILON);
    assert!((zoom.larger().factor() - 1.1).abs() < 1e-6);
    assert!((zoom.smaller().factor() - 0.9).abs() < 1e-6);
}

#[test]
fn the_zoom_stops_at_its_bounds() {
    let mut largest = Zoom::default();
    let mut smallest = Zoom::default();
    for _ in 0_u8..20 {
        largest = largest.larger();
        smallest = smallest.smaller();
    }

    assert!((largest.factor() - 1.5).abs() < 1e-6);
    assert!((smallest.factor() - 0.8).abs() < 1e-6);
}

#[test]
fn a_stored_factor_comes_back_rounded_and_within_the_bounds() {
    assert_eq!(Zoom::from(1.3), Zoom::default().larger().larger().larger());
    assert_eq!(Zoom::from(1.29), Zoom::from(1.3));
    assert_eq!(Zoom::from(5.0), Zoom::from(1.5));
    assert_eq!(Zoom::from(0.2), Zoom::from(0.8));
    assert_eq!(Zoom::from(f32::NAN), Zoom::default());
}
