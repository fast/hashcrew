use std::env;
use std::fs::File;
use std::io::{self, Read};

use rache::Xxh3;

fn main() -> io::Result<()> {
    let path = env::args_os()
        .nth(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "usage: hash-file <path>"))?;
    let mut file = File::open(&path)?;
    let mut hasher = Xxh3::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    println!("{:016x}  {}", hasher.digest(), path.to_string_lossy());
    Ok(())
}
