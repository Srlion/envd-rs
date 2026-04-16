use std::{io, path::Path};

pub use envd_macros::{dyn_var, var};
pub use envd_parser::parse;

pub fn load() -> io::Result<()> {
    load_with("./.env", Options::default())
}

pub fn load_with(p: impl AsRef<Path>, opts: Options) -> io::Result<()> {
    let content = std::fs::read_to_string(p)?;
    let env = parse(&content);

    for (key, value) in env {
        if opts.override_existing || std::env::var_os(&key).is_none() {
            unsafe {
                std::env::set_var(key, value);
            }
        }
    }

    Ok(())
}

#[derive(Default)]
pub struct Options {
    pub override_existing: bool,
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn override_existing(mut self) -> Self {
        self.override_existing = true;
        self
    }
}
