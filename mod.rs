use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

pub const TOMBSTONE: &str = "__DELETED__";

pub struct BinarySSTable;

impl BinarySSTable {
    /// Writes key-value pairs to disk in a custom binary format.
    pub fn write_to_file<P: AsRef<Path>>(
        path: P,
        entries: &[(String, String)],
    ) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write record count
        writer.write_all(&(entries.len() as u32).to_le_bytes())?;

        for (key, value) in entries {
            let key_bytes = key.as_bytes();
            let value_bytes = value.as_bytes();

            writer.write_all(&(key_bytes.len() as u32).to_le_bytes())?;
            writer.write_all(&(value_bytes.len() as u32).to_le_bytes())?;

            writer.write_all(key_bytes)?;
            writer.write_all(value_bytes)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Reads a specific key from a binary SSTable file.
    pub fn get_from_file<P: AsRef<Path>>(
        path: P,
        key: &str,
    ) -> std::io::Result<Option<Option<String>>> {
        let file = OpenOptions::new().read(true).open(path)?;
        let mut reader = BufReader::new(file);

        // Read record count
        let mut count_buf = [0u8; 4];
        reader.read_exact(&mut count_buf)?;
        let record_count = u32::from_le_bytes(count_buf);

        for _ in 0..record_count {
            let key_len = read_u32(&mut reader)? as usize;
            let val_len = read_u32(&mut reader)? as usize;

            let mut key_buf = vec![0u8; key_len];
            reader.read_exact(&mut key_buf)?;

            let mut val_buf = vec![0u8; val_len];
            reader.read_exact(&mut val_buf)?;

            let k = String::from_utf8_lossy(&key_buf);
            let v = String::from_utf8_lossy(&val_buf);

            if k == key {
                if v == TOMBSTONE {
                    return Ok(Some(None)); // tombstone
                } else {
                    return Ok(Some(Some(v.to_string())));
                }
            }
        }

        Ok(None) // key not found
    }
}

/// Reads a little-endian u32 from a stream
fn read_u32<R: Read>(reader: &mut R) -> std::io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}
