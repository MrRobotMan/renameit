use std::ffi::OsStr;

#[derive(Debug)]
pub(crate) enum PathString {
    Valid(String),
    Invalid(String),
}
/// Convert a Path to a mutable string
pub(crate) fn generate_path_as_string(part: Option<&OsStr>) -> Option<PathString> {
    part.map(|path| match path.to_str() {
        Some(s) => PathString::Valid(s.into()),
        None => PathString::Invalid(path.to_string_lossy().into_owned()),
    })
}
