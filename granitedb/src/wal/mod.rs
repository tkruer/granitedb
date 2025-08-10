use anyhow::{bail, Result};
use crc32fast::Hasher;
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

pub struct Wal {
    f: File,
}

impl Wal {
    pub fn open(path: &Path) -> Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(Self { f })
    }

    pub fn append(&mut self, seq: u64, key: &[u8], val: &[u8], fsync: bool) -> Result<u64> {
        let mut buf = Vec::with_capacity(8 + 4 + 4 + key.len() + val.len() + 4);
        buf.extend_from_slice(&seq.to_le_bytes());
        buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
        buf.extend_from_slice(&(val.len() as u32).to_le_bytes());
        buf.extend_from_slice(key);
        buf.extend_from_slice(val);

        let mut h = Hasher::new();
        h.update(&buf);
        let crc = h.finalize();
        buf.extend_from_slice(&crc.to_le_bytes());

        self.f.seek(SeekFrom::End(0))?;
        self.f.write_all(&buf)?;
        if fsync {
            self.f.sync_all()?;
        }
        Ok(buf.len() as u64)
    }

    pub fn replay(&mut self, mut visit: impl FnMut(u64, &[u8], &[u8]) -> Result<()>) -> Result<()> {
        self.f.seek(SeekFrom::Start(0))?;
        let mut rdr = io::BufReader::new(&self.f);
        loop {
            let mut header = [0u8; 8 + 4 + 4];
            if rdr.read_exact(&mut header).is_err() {
                break;
            } // EOF
            let seq = u64::from_le_bytes(header[0..8].try_into().unwrap());
            let klen = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
            let vlen = u32::from_le_bytes(header[12..16].try_into().unwrap()) as usize;

            let mut kv = vec![0u8; klen + vlen];
            rdr.read_exact(&mut kv)?;

            let mut crc_buf = [0u8; 4];
            rdr.read_exact(&mut crc_buf)?;
            let want = u32::from_le_bytes(crc_buf);

            let mut h = Hasher::new();
            h.update(&header);
            h.update(&kv);
            let got = h.finalize();
            if got != want {
                bail!("WAL CRC mismatch");
            }

            let (k, v) = kv.split_at(klen);
            visit(seq, k, v)?;
        }
        Ok(())
    }
}
