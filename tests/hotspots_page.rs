use gitinsight_rs::analytics::FileHotspot;
use gitinsight_rs::ui::hotspot::{hotspot_row, hotspot_rows};

#[test]
fn hotspot_row_contains_path_and_change_count() {
    let mut hotspot = FileHotspot::new("src/main.rs");
    hotspot.change_count = 12;

    let row = hotspot_row(&hotspot);

    assert!(row.contains("src/main.rs"));
    assert!(row.contains("12 changes"));
}

#[test]
fn hotspot_rows_respect_top_n() {
    let mut first = FileHotspot::new("src/main.rs");
    first.change_count = 3;
    let mut second = FileHotspot::new("src/lib.rs");
    second.change_count = 2;
    let mut third = FileHotspot::new("README.md");
    third.change_count = 1;

    let rows = hotspot_rows(&[first, second, third], 2);

    assert_eq!(rows.len(), 2);
    assert!(rows[0].contains("src/main.rs"));
    assert!(rows[1].contains("src/lib.rs"));
}
