use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct WriteAheadLog {
    file: File,
}

impl WriteAheadLog {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { file })
    }

    pub fn log_put(&mut self, key: &str, value: &str) -> std::io::Result<()> {
        writeln!(self.file, "put:{}:{}", key, value)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn log_delete(&mut self, key: &str) -> std::io::Result<()> {
        writeln!(self.file, "del:{}", key)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn replay<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<(String, Option<String>)>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut ops = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("put:") {
                if let Some((k, v)) = line["put:".len()..].split_once(':') {
                    ops.push((k.to_string(), Some(v.to_string())));
                }
            } else if line.starts_with("del:") {
                let key = line["del:".len()..].to_string();
                ops.push((key, None));
            }
        }

        Ok(ops)
    }

    pub fn reset<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        File::create(path)?; // truncates the file
        Ok(())
    }
}
