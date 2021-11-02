mod matcher;
mod opts;
mod printer;

use crate::matcher::find_data;
use crate::opts::Opts;
use crate::printer::print_output;
use clap::Parser;
use memmap::MmapOptions;
use std::fs::File;
use std::io::Result;
use std::sync::mpsc::sync_channel;
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
