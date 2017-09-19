use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};


/// Calls `File::open(path)` and if the result is `Err`, replace the error with new one which
/// message contains `path`.
pub fn open_file_remembering_path(path: &Path) -> io::Result<File> {
    match File::open(path) {
        Ok(file) => Ok(file),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("No such file: {:?}", path),
        )),
        Err(ref e) => Err(io::Error::new(
            e.kind(),
            format!("IO error occured while opening {:?}: {}", path, e),
        )),
    }
}


/// Calls `fs::create_dir_all` and `File::create`.
pub fn create_file_and_dirs(path: &Path) -> io::Result<File> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    match File::create(path) {
        Ok(file) => Ok(file),
        Err(ref e) => Err(io::Error::new(
            e.kind(),
            format!(
                "IO error occured while opening/creating {:?}: {}",
                path,
                e
            ),
        )),
    }
}


/// Returns a `String` read from given source.
pub fn string_from_read<R: Read>(read: R) -> io::Result<String> {
    let mut read = read;
    let mut buf = String::new();
    read.read_to_string(&mut buf)?;
    Ok(buf)
}


/// Equals to `string_from_read(open_file_remembering_path(path)?)`.
pub fn string_from_file_path(path: &Path) -> io::Result<String> {
    string_from_read(open_file_remembering_path(path)?)
}


/// Prints the given string ignoring the last newline if it exists.
pub fn eprintln_trimming_last_newline(s: &str) {
    if s.chars().last() == Some('\n') {
        eprint_and_flush!("{}", s);
    } else {
        eprintln!("{}", s);
    }
}


/// Returns the path the current user's home directory as `io::Result`.
pub fn home_dir_as_io_result() -> io::Result<PathBuf> {
    env::home_dir().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Home directory not found",
    ))
}


pub trait OkAsRefOr {
    type Item;
    /// Get the value `&x` if `Some(ref x) = self`.
    ///
    /// # Panics
    ///
    /// Panics if `self` is `None`.
    fn ok_as_ref_or<E>(&self, e: E) -> Result<&Self::Item, E>;
}

impl<T> OkAsRefOr for Option<T> {
    type Item = T;
    fn ok_as_ref_or<E>(&self, e: E) -> Result<&T, E> {
        match *self {
            Some(ref x) => Ok(x),
            None => Err(e),
        }
    }
}


pub trait UnwrapAsRefMut {
    type Item;
    /// Gets the value `&mut x` if `Some(ref mut x) = self`.
    ///
    /// # Panics
    ///
    /// Panics if `self` is `None`.
    fn unwrap_as_ref_mut(&mut self) -> &mut Self::Item;
}

impl<T> UnwrapAsRefMut for Option<T> {
    type Item = T;

    fn unwrap_as_ref_mut(&mut self) -> &mut T {
        match *self {
            Some(ref mut x) => x,
            None => {
                panic!(
                    "called `<Option as UnwrapAsRefMut>::unwrap_as_ref_mut` \
                     on a `None` value"
                )
            }
        }
    }
}


pub trait CapitalizeFirst {
    /// Capitalizes the first letter.
    fn capitalize_first(&self) -> String;
}

impl CapitalizeFirst for str {
    fn capitalize_first(&self) -> String {
        let mut chars = self.chars();
        chars
            .next()
            .map(|c| format!("{}{}", c.to_uppercase(), chars.as_str()))
            .unwrap_or_default()
    }
}
