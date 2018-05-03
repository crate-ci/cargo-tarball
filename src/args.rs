use std::path;

use format::Format;
use structopt::StructOpt;

#[cfg(windows)]
const DEFAULT_FORMAT: &str = "Zip";
#[cfg(not(windows))]
const DEFAULT_FORMAT: &str = "Tar";

#[derive(StructOpt, Debug)]
#[structopt(name = "staging")]
pub struct Arguments {
    #[structopt(short = "i", long = "input", name = "STAGE", parse(from_os_str))]
    pub input_stage: path::PathBuf,
    #[structopt(short = "d", long = "data", name = "DATA_DIR", parse(from_os_str))]
    pub data_dir: Vec<path::PathBuf>,
    #[structopt(flatten)]
    pub output: Output,
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: u8,
}

#[derive(StructOpt, Debug)]
pub struct Output {
    #[structopt(short = "o", long = "output", name = "OUT", parse(from_os_str))]
    pub dir: path::PathBuf,
    #[structopt(long = "format",
                raw(possible_values = "&Format::variants()", case_insensitive = "true"),
                raw(default_value = "DEFAULT_FORMAT"))]
    pub format: Format,
    #[structopt(short = "n", long = "dry-run")]
    pub dry_run: bool,
}
