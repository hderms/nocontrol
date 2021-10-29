use clap::Parser;

#[derive(Parser)]
#[clap(version = "1.0", author = "Dermot H. <dermot.thomas.haughey@gmail.com")]
pub struct Opts {
    ///File to open, uses mmap if given; otherwise default to STDIN
    #[clap(short, long)]
    pub file: Option<String>,
    /// number of context characters to show around the control character
    #[clap(short, long, default_value = "0")]
    pub context: usize,
}
