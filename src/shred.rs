use rand::Rng;
use std::fs;
use std::io::{self, Seek, Write};
use std::path::Path;

pub fn shred_path(path: &Path, passes: u32) -> io::Result<()> {
    if path.is_dir() {
        shred_dir(path, passes)
    } else {
        shred_file(path, passes)
    }
}

fn shred_file(path: &Path, passes: u32) -> io::Result<()> {
    let size = fs::metadata(path)?.len() as usize;
    if size == 0 {
        return fs::remove_file(path);
    }

    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    let mut rng = rand::rng();
    let mut buf = vec![0u8; size.min(65536)];

    for _ in 0..passes {
        file.seek(io::SeekFrom::Start(0))?;
        let mut remaining = size;
        while remaining > 0 {
            let chunk = remaining.min(buf.len());
            rng.fill(&mut buf[..chunk]);
            file.write_all(&buf[..chunk])?;
            remaining -= chunk;
        }
        file.flush()?;
        file.sync_all()?;
    }

    // 最后一轮零填充
    file.seek(io::SeekFrom::Start(0))?;
    buf.fill(0);
    let mut remaining = size;
    while remaining > 0 {
        let chunk = remaining.min(buf.len());
        file.write_all(&buf[..chunk])?;
        remaining -= chunk;
    }
    file.flush()?;
    file.sync_all()?;

    drop(file);
    fs::remove_file(path)
}

fn shred_dir(path: &Path, passes: u32) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            shred_dir(&p, passes)?;
        } else {
            shred_file(&p, passes)?;
        }
    }
    fs::remove_dir(path)
}
