#[macro_use]
extern crate clap;
#[macro_use]
extern crate structopt;

use std::env;
use std::fs;
use std::io::Write;
use std::path;
use std::process;

use structopt::StructOpt;
use structopt::clap::Shell;

#[allow(dead_code)]
#[path = "src/format.rs"]
mod format;
#[allow(dead_code)]
#[path = "src/args.rs"]
mod args;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/format.rs");
    println!("cargo:rerun-if-changed=src/args.rs");

    // OUT_DIR is set by Cargo and it's where any additional build artifacts
    // are written.
    let outdir = match env::var_os("OUT_DIR") {
        Some(outdir) => outdir,
        None => {
            eprintln!(
                "OUT_DIR environment variable not defined. \
                 Please file a bug: \
                 https://github.com/crate-ci/cargo-tarball/issues/new"
            );
            process::exit(1);
        }
    };

    // Use clap to build completion files.
    let completions_dir = path::Path::new(&outdir).join("completions");
    fs::create_dir_all(&completions_dir).unwrap();
    let bin = env!("CARGO_PKG_NAME");
    let mut clap = args::Arguments::clap();
    clap.gen_completions(bin, Shell::Bash, &completions_dir);
    clap.gen_completions(bin, Shell::Fish, &completions_dir);
    clap.gen_completions(bin, Shell::PowerShell, &completions_dir);
    clap.gen_completions(bin, Shell::Zsh, &completions_dir);

    // env::ARCH doesn't include full triple, and AFAIK there isn't a nicer way of getting the full triple
    // (see lib.rs for the rest of this hack)
    let out = path::PathBuf::from(env::var_os("OUT_DIR").expect("run within cargo"))
        .join("default_target.txt");
    let default_target = env::var("TARGET").expect("run as cargo build script");
    let mut file = fs::File::create(out).unwrap();
    file.write_all(default_target.as_bytes()).unwrap();
}
