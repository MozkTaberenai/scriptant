pub use std::ffi::{OsStr, OsString};
pub use std::io::{prelude::*, BufReader, BufWriter};
pub use std::path::{Path, PathBuf};
pub type AnyError = Box<dyn std::error::Error>;
pub type Result<T, E = AnyError> = std::result::Result<T, E>;

mod cmd;
pub use cmd::*;

pub mod fs;

mod echo;
pub use echo::{echo, Echo};

mod style;
pub use style::{style, Style};

#[macro_export]
macro_rules! cmd {
    ($program:expr) => {
        $crate::Command::new($program)
    };
    ($program:expr, $($arg:expr),* $(,)?) => {
        $crate::Command::new($program)$(.arg($arg))*
    };
}

#[macro_export]
macro_rules! echo {
    ($($arg:expr),* $(,)?) => {
        $crate::Echo::new()
            $(.put($arg))*
            .end();
    };
    () => {
        println!();
    };
}
