use crate::options::Options;
use crate::wal::Wal;
use anyhow::Result;
use std::{collections::BTreeMap, fs, path::PathBuf};

pub struct Db {
    opts: Options,
    wal: Wal,
    mem: BTreeMap<Vec<u8>, Vec<u8>>,
    seq: u64,
}

impl Db {
    pub fn open(opts: Options) -> Result<Self> {
        fs::create_dir_all(&opts.path)?;
        let wal_path = opts.path.join("wal.log");
        let mut wal = Wal::open(&wal_path)?;
        let mut mem = BTreeMap::new();
        let mut seq = 0u64;

        wal.replay(|s, k, v| {
            seq = seq.max(s);
            mem.insert(k.to_vec(), v.to_vec());
            Ok(())
        })?;

        Ok(Self {
            opts,
            wal,
            mem,
            seq,
        })
    }

    pub fn put(&mut self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
        self.seq += 1;
        let (k, v) = (key.as_ref(), val.as_ref());
        self.wal.append(self.seq, k, v, self.opts.wal_fsync)?;
        self.mem.insert(k.to_vec(), v.to_vec());
        Ok(())
    }

    pub fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        Ok(self.mem.get(key.as_ref()).cloned())
    }
}
