use crate::wal::WriteAheadLog;
use crate::{bincoding::BinarySSTable, bincoding::TOMBSTONE};
use std::collections::BTreeMap;
use std::fs;

const SSTABLE_DIR: &str = ".granitedb/";
const MEMTABLE_THRESHOLD: usize = 100;

pub struct LSMTree {
    pub memtable: BTreeMap<String, String>,
    pub sstable_counter: usize,
    pub wal: WriteAheadLog,
    pub threshold: usize,
}

impl LSMTree {
    pub fn new() -> Self {
        fs::create_dir_all(SSTABLE_DIR).unwrap();
        let wal_path = format!("{SSTABLE_DIR}wal.log");

        let mut memtable = BTreeMap::new();
        let sstable_counter = 0;

        // Replay WAL to rebuild memtable
        if let Ok(ops) = WriteAheadLog::replay(&wal_path) {
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

        let wal = WriteAheadLog::new(&wal_path).unwrap();
        let threshold = 3;

        Self {
            memtable,
            sstable_counter,
            wal,
            threshold,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.memtable.get(key).map(|v| v.clone())
    }

    pub fn put(&mut self, key: String, value: String) {
        self.wal.log_put(&key, &value).unwrap();
        self.memtable.insert(key, value);
        if self.memtable.len() >= MEMTABLE_THRESHOLD {
            self.flush_to_disk();
        }
    }

    pub fn delete(&mut self, key: String) {
        self.wal.log_delete(&key).unwrap();
        self.memtable.insert(key, TOMBSTONE.to_string());
        if self.memtable.len() >= MEMTABLE_THRESHOLD {
            self.flush_to_disk();
        }
    }

    fn flush_to_disk(&mut self) {
        let filename = format!("{SSTABLE_DIR}sstable_{}.bin", self.sstable_counter);
        let entries: Vec<_> = self
            .memtable
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        BinarySSTable::write_to_file(&filename, &entries).unwrap();
        WriteAheadLog::reset(format!("{SSTABLE_DIR}wal.log")).unwrap();

        self.memtable.clear();
        self.sstable_counter += 1;
    }
}
