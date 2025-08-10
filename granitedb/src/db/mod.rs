use crate::{bincoding::TOMBSTONE, lsmtree::LSMTree, wal::WriteAheadLog};
use std::{collections::BTreeMap, fs, io, path::PathBuf};

pub struct GraniteDB {
    pub sstable_dir: PathBuf,
    pub threshold: usize,
    pub wal_path: PathBuf,
}

impl GraniteDB {
    pub fn new() -> Self {
        let dir = PathBuf::from(".granitedb");
        Self {
            sstable_dir: dir.clone(),
            threshold: 3,
            wal_path: dir.join("wal.log"),
        }
    }

    pub fn with_sstable_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        let d = dir.into();
        self.wal_path = d.join("wal.log");
        self.sstable_dir = d;
        self
    }

    pub fn with_threshold(mut self, threshold: usize) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn build(self) -> io::Result<LSMTree> {
        fs::create_dir_all(&self.sstable_dir)?;

        // replay into memtable
        let mut memtable = BTreeMap::new();
        if let Ok(ops) = WriteAheadLog::replay(&self.wal_path) {
            for (key, value_opt) in ops {
                match value_opt {
                    Some(val) => {
                        memtable.insert(key, val);
                    }
                    None => {
                        memtable.insert(key, TOMBSTONE.to_string());
                    }
                }
            }
        }

        Ok(LSMTree {
            memtable,
            sstable_counter: 0, // TODO: initialize from manifest later
            wal: WriteAheadLog::new(&self.wal_path)?,
            threshold: self.threshold,
        })
    }
}
