mod opts;

use crate::opts::Opts;
use clap::Parser;
use memmap::{Mmap, MmapOptions};
use std::cmp::{max, min};
use std::fs::File;
use std::io::{Result, Write};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::sync::Barrier;
use std::thread;

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    //2 threads for input/output and then 1 to wait for both to complete before exiting
    let barrier = Arc::new(Barrier::new(3));

    match opts.file {
        None => {}
        Some(file) => unsafe {
            let (tx, rx) = sync_channel(64);

            let file = File::open(file).unwrap();
            let mmap = MmapOptions::new().map(&file).unwrap();
            let mmap = Arc::new(mmap);

            {
                let barrier = barrier.clone();
                let mmap = mmap.clone();
                thread::spawn(move || {
                    find_data(&mmap, tx);
                    barrier.wait();
                });
            }

            {
                let mmap = mmap.clone();
                let barrier = barrier.clone();
                thread::spawn(move || {
                    print_output(&mmap, rx, opts.context).unwrap();
                    barrier.wait();
                });
            }
            barrier.wait();
        },
    }
    Ok(())
}

fn print_output(mmap: &Mmap, rx: Receiver<Match>, context: usize) -> Result<()> {
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
struct Match {
    index: usize,
    line: usize,
    column: usize,
    byte: u8,
}

unsafe fn find_data(mmap: &Mmap, tx: SyncSender<Match>) {
    let mut line = 0;
    let mut column = 0;
    for (index, byte) in mmap.iter().enumerate() {
        let byte = *byte;

        if !is_printable_character(byte) {
            tx.send(Match {
                index,
                line,
                column,
                byte,
            })
            .unwrap();
        }
        if byte == b'\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
}

fn is_printable_character(byte: u8) -> bool {
    !byte.is_ascii_control() || byte == b'\r' || byte == b'\n' || byte == b'\t'
}
#[cfg(test)]
mod test {
    use crate::is_printable_character;

    #[test]
    fn should_identify_non_printable_characters() {
        assert_eq!(is_printable_character(b'\r'), true);
        assert_eq!(is_printable_character(b'\n'), true);
        assert_eq!(is_printable_character(b'\t'), true);
        assert_eq!(is_printable_character(b' '), true);
        for c in 'a'..='z' {
            assert_eq!(is_printable_character(c as u8), true);
        }

        for c in 'A'..='Z' {
            assert_eq!(is_printable_character(c as u8), true);
        }

        for c in '0'..='9' {
            assert_eq!(is_printable_character(c as u8), true);
        }
    }
}
