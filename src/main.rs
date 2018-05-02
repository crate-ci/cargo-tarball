#![warn(warnings)]

extern crate env_logger;
extern crate exitcode;
extern crate globwalk;
extern crate liquid;
extern crate stager;
extern crate tempfile;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate structopt;

#[cfg(feature = "serde_json")]
extern crate serde_json;
#[cfg(feature = "serde_yaml")]
extern crate serde_yaml;
#[cfg(feature = "toml")]
extern crate toml;

use std::ffi;
use std::fs;
use std::io::Write;
use std::io;
use std::path;
use std::process;

use failure::ResultExt;
use structopt::StructOpt;

use stager::de::Render;
use stager::builder::ActionBuilder;

mod compress;

mod stage {
    use super::*;
    use std::io::Read;

    #[cfg(feature = "serde_yaml")]
    pub fn load_yaml(path: &path::Path) -> Result<stager::de::Staging, failure::Error> {
        let f = fs::File::open(path)?;
        serde_yaml::from_reader(f).map_err(|e| e.into())
    }

    #[cfg(not(feature = "serde_yaml"))]
    pub fn load_yaml(_path: &path::Path) -> Result<stager::de::Staging, failure::Error> {
        bail!("yaml is unsupported");
    }

    #[cfg(feature = "serde_json")]
    pub fn load_json(path: &path::Path) -> Result<stager::de::Staging, failure::Error> {
        let f = fs::File::open(path)?;
        serde_json::from_reader(f).map_err(|e| e.into())
    }

    #[cfg(not(feature = "serde_json"))]
    pub fn load_json(_path: &path::Path) -> Result<stager::de::Staging, failure::Error> {
        bail!("json is unsupported");
    }

    #[cfg(feature = "toml")]
    pub fn load_toml(path: &path::Path) -> Result<stager::de::Staging, failure::Error> {
        let mut f = fs::File::open(path)?;
        let mut text = String::new();
        f.read_to_string(&mut text)?;
        toml::from_str(&text).map_err(|e| e.into())
    }

    #[cfg(not(feature = "toml"))]
    pub fn load_toml(_path: &path::Path) -> Result<stager::de::Staging, failure::Error> {
        bail!("toml is unsupported");
    }
}

fn load_stage(path: &path::Path) -> Result<stager::de::Staging, failure::Error> {
    let extension = path.extension().unwrap_or_default();
    let value = if extension == ffi::OsStr::new("yaml") {
        stage::load_yaml(path)
    } else if extension == ffi::OsStr::new("toml") {
        stage::load_toml(path)
    } else if extension == ffi::OsStr::new("json") {
        stage::load_json(path)
    } else {
        bail!("Unsupported file type");
    }?;

    Ok(value)
}

mod object {
    use super::*;
    use std::io::Read;

    #[cfg(feature = "serde_yaml")]
    pub fn load_yaml(path: &path::Path) -> Result<liquid::Value, failure::Error> {
        let f = fs::File::open(path)?;
        serde_yaml::from_reader(f).map_err(|e| e.into())
    }

    #[cfg(not(feature = "serde_yaml"))]
    pub fn load_yaml(_path: &path::Path) -> Result<liquid::Value, failure::Error> {
        bail!("yaml is unsupported");
    }

    #[cfg(feature = "serde_json")]
    pub fn load_json(path: &path::Path) -> Result<liquid::Value, failure::Error> {
        let f = fs::File::open(path)?;
        serde_json::from_reader(f).map_err(|e| e.into())
    }

    #[cfg(not(feature = "serde_json"))]
    pub fn load_json(_path: &path::Path) -> Result<liquid::Value, failure::Error> {
        bail!("json is unsupported");
    }

    #[cfg(feature = "toml")]
    pub fn load_toml(path: &path::Path) -> Result<liquid::Value, failure::Error> {
        let mut f = fs::File::open(path)?;
        let mut text = String::new();
        f.read_to_string(&mut text)?;
        toml::from_str(&text).map_err(|e| e.into())
    }

    #[cfg(not(feature = "toml"))]
    pub fn load_toml(_path: &path::Path) -> Result<liquid::Value, failure::Error> {
        bail!("toml is unsupported");
    }

    pub fn insert(
        object: &mut liquid::Object,
        path: &[String],
        key: String,
        value: liquid::Value,
    ) -> Result<(), failure::Error> {
        let leaf = path.iter().cloned().fold(Ok(object), |object, key| {
            let cur_object = object?;
            cur_object
                .entry(key)
                .or_insert_with(|| liquid::Value::Object(liquid::Object::new()))
                .as_object_mut()
                .ok_or_else(|| {
                    failure::Context::new(format!(
                        "Aborting: Duplicate in data tree. Would overwrite {:?} ",
                        path
                    ))
                })
        })?;

        match leaf.insert(key, value) {
            None => Ok(()),
            _ => bail!(
                "The data from {:?} can't be loaded: the key already exists",
                path
            ),
        }
    }
}

fn load_data(path: &path::Path) -> Result<liquid::Value, failure::Error> {
    let extension = path.extension().unwrap_or_default();
    let value = if extension == ffi::OsStr::new("yaml") {
        object::load_yaml(path)
    } else if extension == ffi::OsStr::new("toml") {
        object::load_toml(path)
    } else if extension == ffi::OsStr::new("json") {
        object::load_json(path)
    } else {
        bail!("Unsupported file type");
    }?;

    Ok(value)
}

fn load_data_dirs(roots: &[path::PathBuf]) -> Result<liquid::Object, failure::Error> {
    let mut object = liquid::Object::new();
    // TODO(epage): swap out globwalk for something that uses gitignore so we can have
    // exclusion support.
    let patterns: &[&'static str] = &[
        #[cfg(feature = "serde_yaml")]
        "*.yaml",
        #[cfg(feature = "serde_json")]
        "*.json",
        #[cfg(feature = "toml")]
        "*.toml",
    ];
    for root in roots {
        for entry in globwalk::GlobWalker::from_patterns(root, &patterns)? {
            let entry = entry?;
            let data_file = entry.path();
            let data = load_data(data_file)?;
            let rel_source = data_file.strip_prefix(&root)?;
            let path = rel_source.parent().unwrap_or_else(|| path::Path::new(""));
            let path: Option<Vec<_>> = path.components()
                .map(|c| {
                    let c: &ffi::OsStr = c.as_ref();
                    c.to_str().map(String::from)
                })
                .collect();
            let path = match path {
                Some(p) => p,
                None => {
                    warn!("Invalid data file path: {:?}", rel_source);
                    continue;
                }
            };
            let key = match rel_source
                .file_name()
                .expect("file name to exist due to globwalk")
                .to_str()
                .map(String::from)
            {
                Some(p) => p,
                None => {
                    warn!("Invalid data file path: {:?}", rel_source);
                    continue;
                }
            };
            object::insert(&mut object, &path, key, data)?;
        }
    }

    Ok(object)
}

#[derive(StructOpt, Debug)]
#[structopt(name = "staging")]
struct Arguments {
    #[structopt(short = "i", long = "input", name = "STAGE", parse(from_os_str))]
    input_stage: path::PathBuf,
    #[structopt(short = "d", long = "data", name = "DATA_DIR", parse(from_os_str))]
    data_dir: Vec<path::PathBuf>,
    #[structopt(short = "o", long = "output", name = "OUT", parse(from_os_str))]
    output: path::PathBuf,
    #[structopt(short = "n", long = "dry-run")]
    dry_run: bool,
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbosity: u8,
}

fn run() -> Result<exitcode::ExitCode, failure::Error> {
    let mut builder = env_logger::Builder::new();
    let args = Arguments::from_args();
    let level = match args.verbosity {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    builder.filter(None, level);
    if level == log::LevelFilter::Trace {
        builder.default_format_timestamp(false);
    } else {
        builder.format(|f, record| {
            writeln!(
                f,
                "[{}] {}",
                record.level().to_string().to_lowercase(),
                record.args()
            )
        });
    }
    builder.init();

    let data = load_data_dirs(&args.data_dir)?;
    let engine = stager::de::TemplateEngine::new(data)?;

    let staging = load_stage(&args.input_stage)
        .with_context(|_| format!("Failed to load {:?}", args.input_stage))?;

    let staging = staging.format(&engine);
    let staging = match staging {
        Ok(s) => s,
        Err(e) => {
            error!("Failed reading stage file: {}", e);
            return Ok(exitcode::DATAERR);
        }
    };

    // TODO(epage): Support compressing an in-memory stream of `stager::Action`s
    let staging_dir = tempfile::tempdir()?;
    let staging = staging.build(staging_dir.path());
    let staging = match staging {
        Ok(s) => s,
        Err(e) => {
            error!("Failed preparing staging: {}", e);
            return Ok(exitcode::IOERR);
        }
    };

    for action in staging {
        debug!("{}", action);
        if !args.dry_run {
            action
                .perform()
                .with_context(|_| format!("Failed staging files: {}", action))?;
        }
    }

    let format = compress::Format::Tgz;
    compress::compress(staging_dir.path(), &args.output, format)?;

    Ok(exitcode::OK)
}

fn main() {
    let code = match run() {
        Ok(e) => e,
        Err(ref e) => {
            writeln!(&mut io::stderr(), "{}", e).expect("writing to stderr won't fail");
            exitcode::SOFTWARE
        }
    };
    process::exit(code);
}
