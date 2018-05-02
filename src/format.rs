arg_enum!{
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Format {
        Tar,
        Tgz,
        Zip,
    }
}

impl Format {
    pub fn ext(self) -> &'static str {
        match self {
            Format::Tar => ".tar",
            Format::Tgz => ".tar.gz",
            Format::Zip => ".zip",
        }
    }
}
