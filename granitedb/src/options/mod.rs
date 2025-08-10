use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Options {
    pub path: PathBuf,
    pub wal_fsync: bool,
    pub memtable_bytes: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            path: "./data".into(),
            wal_fsync: true,
            memtable_bytes: 32 * 1024 * 1024,
        }
    }
}
