use super::size;

#[test]
fn small_sizes_stay_in_bytes() {
    assert_eq!(size(0), "0 B");
    assert_eq!(size(1023), "1023 B");
}

#[test]
fn larger_sizes_take_one_decimal_in_the_largest_unit() {
    assert_eq!(size(1024), "1.0 KiB");
    assert_eq!(size(4 * 1024 * 1024 + 300 * 1024), "4.3 MiB");
    assert_eq!(size(200 * 1024 * 1024 * 1024), "200.0 GiB");
}
