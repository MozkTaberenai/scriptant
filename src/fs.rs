use super::*;

use once_cell::sync::Lazy;
static ECHO_PREFIX: Lazy<String> = Lazy::new(|| echo::prefix(" fs"));

pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "rename".bold().cyan(),
        from.to_string_lossy().bold().underline(),
        "->",
        to.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::rename(from, to)?)
}

pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "copy".bold().cyan(),
        from.to_string_lossy().bold().underline(),
        "->",
        to.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::copy(from, to)?)
}

pub fn hard_link(original: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    let original = original.as_ref();
    let link = link.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "hard_link".bold().cyan(),
        original.to_string_lossy().bold().underline(),
        "->",
        link.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::hard_link(original, link)?)
}

pub fn create_dir(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "create_dir".bold().cyan(),
        path.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::create_dir(path)?)
}

pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "create_dir_all".bold().cyan(),
        path.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::create_dir_all(path)?)
}

pub fn metadata(path: impl AsRef<Path>) -> Result<std::fs::Metadata> {
    let path = path.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "metadata".bold().cyan(),
        path.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::metadata(path)?)
}

pub fn read_dir(path: impl AsRef<Path>) -> Result<std::fs::ReadDir> {
    let path = path.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "read_dir".bold().cyan(),
        path.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::read_dir(path)?)
}

pub fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let path = path.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "read".bold().cyan(),
        path.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::read(path)?)
}

pub fn read_to_string(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "read_to_string".bold().cyan(),
        path.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::read_to_string(path)?)
}

pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    let contents = contents.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "write".bold().cyan(),
        format!("{} bytes", contents.len()),
        "->",
        path.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::write(path, contents)?)
}

pub fn remove_dir(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "remove_dir".bold().cyan(),
        path.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::remove_dir(path)?)
}

pub fn remove_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "remove_dir_all".bold().cyan(),
        path.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::remove_dir_all(path)?)
}

pub fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    echo!(
        &*ECHO_PREFIX,
        "remove_file".bold().cyan(),
        path.to_string_lossy().bold().underline(),
    );
    Ok(std::fs::remove_file(path)?)
}
