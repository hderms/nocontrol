use crate::matcher::ControlCharacterMatch;
use memmap::Mmap;
use std::cmp::{max, min};
use std::io::{Result, Write};
use std::sync::mpsc::Receiver;

pub fn print_output(
    mmap: &Mmap,
    rx: Receiver<ControlCharacterMatch>,
    context: usize,
) -> Result<()> {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let mmap_len = mmap.len();

    loop {
        match rx.recv() {
            Ok(m) => {
                writeln!(
                    lock,
                    "{}:{} control character ascii value {} at byte: {}",
                    m.line, m.column, m.byte, m.index
                )
                .unwrap();
                if context != 0 {
                    let minimum = max(0, m.index.saturating_sub(context));
                    let maximum = min(mmap_len, m.index.saturating_add(context));
                    let before = &mmap[minimum..m.index];
                    let after = &mmap[m.index + 1..maximum];
                    lock.write_all(before)?;
                    lock.write_all(b"X")?;
                    lock.write_all(after)?;
                    lock.write_all(b"\n").unwrap()
                }
            }
            Err(_) => return Ok(()),
        }
    }
}
