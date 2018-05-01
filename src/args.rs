use std::path;

#[derive(StructOpt, Debug)]
#[structopt(name = "staging")]
pub struct Arguments {
    #[structopt(short = "i", long = "input", name = "STAGE", parse(from_os_str))]
    pub input_stage: path::PathBuf,
    #[structopt(short = "d", long = "data", name = "DATA_DIR", parse(from_os_str))]
    pub data_dir: Vec<path::PathBuf>,
    #[structopt(short = "o", long = "output", name = "OUT", parse(from_os_str))]
    pub output: path::PathBuf,
    #[structopt(short = "n", long = "dry-run")]
    pub dry_run: bool,
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: u8,
}
