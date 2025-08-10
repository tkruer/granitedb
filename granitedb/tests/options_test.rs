use granitedb::Options;
use std::path::PathBuf;

#[test]
fn default_has_expected_values() {
    let o = Options::default();
    assert_eq!(o.path, PathBuf::from("./data"));
    assert!(o.wal_fsync);
    assert_eq!(o.memtable_bytes, 32 * 1024 * 1024);
}
