//! 时间线页面测试：验证提交行文本和原始顺序保留。

use chrono::DateTime;
use gitinsight_rs::models::TimelineEntry;
use gitinsight_rs::ui::timeline::{timeline_row, timeline_rows};

#[test]
fn timeline_row_contains_commit_id_author_and_message() {
    let entry = TimelineEntry::new(
        "abcdef12",
        "Alice",
        "alice@example.com",
        "add timeline page",
        DateTime::from_timestamp(0, 0).expect("unix epoch should be valid"),
    );

    let row = timeline_row(&entry);

    assert!(row.contains("abcdef12"));
    assert!(row.contains("Alice"));
    assert!(row.contains("add timeline page"));
}

#[test]
fn timeline_rows_preserve_given_order() {
    let first = TimelineEntry::new(
        "11111111",
        "Tom",
        "tom@example.com",
        "newer",
        DateTime::from_timestamp(60, 0).expect("timestamp should be valid"),
    );
    let second = TimelineEntry::new(
        "22222222",
        "Bob",
        "bob@example.com",
        "older",
        DateTime::from_timestamp(0, 0).expect("unix epoch should be valid"),
    );

    let rows = timeline_rows(&[first, second]);

    assert!(rows[0].contains("newer"));
    assert!(rows[1].contains("older"));
}
