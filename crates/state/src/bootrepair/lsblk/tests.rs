use super::{LSBLK_ARGUMENTS, parse_lsblk};

const GIB: u64 = 1024 * 1024 * 1024;

fn listing(size: &str) -> String {
    format!(
        r#"{{"blockdevices":[
            {{"path":"/dev/sda","type":"disk","size":{size},"model":"Disk","children":[
                {{"path":"/dev/sda1","type":"part","size":{size},"fstype":"btrfs","label":"Garuda"}}
            ]}}
        ]}}"#
    )
}

#[test]
fn lsblk_is_asked_for_sizes_in_bytes() {
    assert!(LSBLK_ARGUMENTS.contains(&"--bytes"), "{LSBLK_ARGUMENTS:?}");
}

#[test]
fn a_size_in_bytes_is_kept_as_a_number_for_the_page_to_word() {
    let size = 200 * GIB;
    let inventory = parse_lsblk(&listing(&size.to_string())).unwrap();

    assert_eq!(inventory.disks[0].size, size);
    assert_eq!(inventory.systems[0].size, size);
}

#[test]
fn an_lsblk_that_quotes_its_bytes_is_read_too() {
    let inventory = parse_lsblk(&listing(&format!("\"{}\"", 200 * GIB))).unwrap();

    assert_eq!(inventory.systems[0].size, 200 * GIB);
}

#[test]
fn a_size_in_another_shape_is_no_size_and_no_error() {
    let inventory = parse_lsblk(&listing("\"200G\"")).unwrap();

    assert_eq!(inventory.systems[0].size, 0);
    assert_eq!(inventory.disks.len(), 1);
}

#[test]
fn a_card_reader_without_a_card_is_no_disk() {
    let inventory = parse_lsblk(&listing("0")).unwrap();

    assert!(inventory.disks.is_empty(), "{:?}", inventory.disks);
}
