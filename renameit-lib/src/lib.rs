use std::{
    cmp::Ordering,
    ffi::OsStr,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

pub mod add;
pub mod case;
pub mod date;
pub mod error;
pub mod extension;
pub mod folder;
pub mod name;
pub mod number;
pub mod reg;
pub mod remove;
pub mod renamer_builder;
pub mod replace;

#[cfg(test)]
mod tester;

use add::AddOptions;
pub use case::{Case, CaseOptions};
use chrono::{DateTime, Local};
pub use date::DateOptions;
pub use error::*;
pub use extension::ExtensionOptions;
pub use folder::FolderOptions;
pub use name::NameOptions;
pub use number::NumberOptions;
pub use reg::RegexOptions;
pub use remove::RemoveOptions;
pub use replace::ReplaceOptions;

pub trait Process {
    fn process(&self, file: &mut Renamer);
}

#[allow(dead_code)]
trait OptionBuilder {
    type Processor: Process;

    fn build(&self) -> Self::Processor;
}

/// Tool to rename a single file.
/// Takes the `&path` and various options (processed in order) to return a `PathBuf`
/// used to rename the file.
/// Options are
///    -  1 RegEx
///    -  2 Name
///    -  3 Replace
///    -  4 Case
///    -  5 Remove
///    -  6 Add
///    -  7 Auto Date
///    -  8 Append Folder Name
///    -  9 Numbering
///    - 10 Extension
///
/// # Example
///
/// ```
/// # use std::path::{Path, PathBuf};
/// # use renameit_lib::{NameOptions, Case, CaseOptions, Renamer, Process, Options};
/// let file = Path::new("file.txt");
/// let name = NameOptions::Fixed("new_name".into());
/// let case = CaseOptions{case: Case::Upper, snake: false, exceptions: "n".into()};
/// let mut rename = Renamer::new(file).unwrap().with_option(Options::Name(name)).with_option(Options::Case(case));
/// let new_name = rename.preview();
/// assert_eq!(new_name, PathBuf::from("nEW_nAME.txt"));
/// ```
#[derive(Debug, Default)]
pub struct Renamer {
    stem: String,
    pub(crate) renamed: String,
    pub(crate) original: PathBuf,
    valid_original: bool,
    pub(crate) extension: Option<String>,
    add: Option<AddOptions>,
    case: Option<CaseOptions>,
    date: Option<DateOptions>,
    ext: Option<ExtensionOptions>,
    folder: Option<FolderOptions>,
    name: Option<NameOptions>,
    number: Option<NumberOptions>,
    regex: Option<RegexOptions>,
    remove: Option<RemoveOptions>,
    replace: Option<ReplaceOptions>,
    pub(crate) _is_dir: bool,
}

impl Renamer {
    /// Create a new File object from a Path.
    /// No checking is performed to validate that the Path exists or is a file.
    /// To perform this check use [Renamer::try_from<&Path>], [Renamer::try_from<&PathBuf>], or `[Renamer::try_from<PathBuf>]`
    pub fn new(path: &Path) -> Result<Self, FileError> {
        let extension = {
            generate_path_as_string(path.extension()).map(|e| match e {
                PathString::Valid(s) => s,
                PathString::Invalid(s) => s,
            })
        };
        match generate_path_as_string(path.file_stem()) {
            Some(stem) => {
                let (stem, valid_original) = match stem {
                    PathString::Valid(s) => (s, true),
                    PathString::Invalid(s) => (s, false),
                };
                Ok(Self {
                    stem,
                    valid_original,
                    extension,
                    original: path.to_owned(),
                    ..Default::default()
                })
            }
            None => Err(FileError::BadStem),
        }
    }

    pub fn preview(&mut self) -> PathBuf {
        let mut opts: Vec<Box<dyn Process>> = vec![];
        if let Some(opt) = &self.regex {
            opts.push(Box::new(opt.clone()));
        };
        if let Some(opt) = &self.name {
            opts.push(Box::new(opt.clone()));
        };
        if let Some(opt) = &self.replace {
            opts.push(Box::new(opt.clone()));
        };
        if let Some(opt) = &self.case {
            opts.push(Box::new(opt.clone()));
        };
        if let Some(opt) = &self.remove {
            opts.push(Box::new(opt.clone()));
        };
        if let Some(opt) = &self.add {
            opts.push(Box::new(opt.clone()));
        };
        if let Some(opt) = &self.date {
            opts.push(Box::new(opt.clone()));
        };
        if let Some(opt) = &self.folder {
            opts.push(Box::new(opt.clone()));
        };
        if let Some(opt) = &self.number {
            opts.push(Box::new(opt.clone()));
        };
        if let Some(opt) = &self.ext {
            opts.push(Box::new(opt.clone()));
        };
        for opt in opts {
            opt.process(self);
        }
        let mut new_name = match self.original.parent() {
            None => PathBuf::from("/"),
            Some(p) => PathBuf::from(p),
        };
        // Note: If the renamed stem looks absolute ('/', 'C:\\', etc) it will
        // full replace the path ignoring all parents. This will be kept because it
        // will be obvious in the GUI automatic preview and ultimately it is then
        // the user's responsibility to make a better name. TODO: add a check for
        // illegal characters on Linux (/), Windows (< > : " / \ | ? *), and
        // MacOS (; /) as well as reserved names. See https://stackoverflow.com/questions/1976007/what-characters-are-forbidden-in-windows-and-linux-directory-names
        // for references.
        new_name.push(Path::new(&self.stem));
        new_name = match &self.extension {
            None => new_name,
            Some(e) => new_name.with_extension(e),
        };
        self.renamed = new_name.to_str().unwrap_or("NON-UTF NAME").to_string();
        new_name
    }

    /// Rename the file. Can not be undone.
    pub fn rename(mut self) -> Result<(), FileError> {
        let new_name = &self.preview();
        fs::rename(&self.original, new_name)?;
        Ok(())
    }

    /// Revert the previewed changes to a file.
    pub fn revert(&mut self) -> Result<(), FileError> {
        let temp: &Renamer = &self.original.clone().try_into()?;
        self.stem = temp.stem.clone();
        self.extension = temp.extension.clone();
        Ok(())
    }

    pub fn with_option(mut self, option: Options) -> Self {
        use Options::*;
        match option {
            Regex(opt) => self.regex = Some(opt),
            Name(opt) => self.name = Some(opt),
            Case(opt) => self.case = Some(opt),
            Remove(opt) => self.remove = Some(opt),
            Add(opt) => self.add = Some(opt),
            Date(opt) => self.date = Some(opt),
            Folder(opt) => self.folder = Some(opt),
            Number(opt) => self.number = Some(opt),
            Extension(opt) => self.ext = Some(opt),
        }
        self
    }

    // Return the information on a file.
    pub fn info(
        &self,
    ) -> (
        Filename<'_>,
        Filename<'_>,
        Extension<'_>,
        Size,
        DateModified,
        DateCreated,
    ) {
        let mut size = None;
        let mut modified = None;
        let mut created = None;
        if let Ok(data) = self.original.metadata() {
            if self.original.is_file() {
                size = Some(data.len())
            };
            if let Ok(dt) = data.modified() {
                modified = Some(dt.into())
            };
            if let Ok(dt) = data.created() {
                created = Some(dt.into())
            };
        };
        (
            &self.stem,
            &self.renamed,
            self.extension.as_deref(),
            size,
            modified,
            created,
        )
    }

    // Check if the original file was valid UTF-8
    pub fn is_valid(&self) -> bool {
        self.valid_original
    }
}

impl TryFrom<&Path> for Renamer {
    type Error = FileError;

    fn try_from(path: &Path) -> Result<Self, FileError> {
        if !path.exists() {
            return Err(FileError::NotFound);
        }
        let extension = {
            generate_path_as_string(path.extension()).map(|e| match e {
                PathString::Valid(s) => s,
                PathString::Invalid(s) => s,
            })
        };
        match generate_path_as_string(path.file_stem()) {
            Some(stem) => {
                let (stem, valid_original) = match stem {
                    PathString::Valid(s) => (s, true),
                    PathString::Invalid(s) => (s, false),
                };
                Ok(Self {
                    stem,
                    valid_original,
                    extension,
                    original: path.to_owned(),
                    _is_dir: path.is_dir(),
                    ..Default::default()
                })
            }
            None => Err(FileError::BadStem),
        }
    }
}

impl TryFrom<PathBuf> for Renamer {
    type Error = FileError;

    fn try_from(value: PathBuf) -> Result<Self, FileError> {
        value.as_path().try_into()
    }
}

impl TryFrom<&PathBuf> for Renamer {
    type Error = FileError;

    fn try_from(value: &PathBuf) -> Result<Self, FileError> {
        value.as_path().try_into()
    }
}

/*
impl From<&Renamer> for WidgetText {
    fn from(value: &Renamer) -> Self {
        Self::RichText(RichText::new(match &value.extension {
            None => value.stem.clone(),
            Some(ext) => format!("{}.{}", value.stem, ext),
        }))
    }
}
*/

#[derive(Debug)]
pub enum PathString {
    Valid(String),
    Invalid(String),
}

/// Convert a Path to a mutable string
pub fn generate_path_as_string(part: Option<&OsStr>) -> Option<PathString> {
    part.map(|path| match path.to_str() {
        Some(s) => PathString::Valid(s.into()),
        None => PathString::Invalid(path.to_string_lossy().into_owned()),
    })
}

pub type Filename<'a> = &'a str;
pub type Extension<'a> = Option<&'a str>;
pub type Size = Option<u64>;
pub type DateCreated = Option<DateTime<Local>>;
pub type DateModified = Option<DateTime<Local>>;

#[derive(Debug)]
pub enum Options {
    Regex(RegexOptions),
    Name(NameOptions),
    Case(CaseOptions),
    Remove(RemoveOptions),
    Add(AddOptions),
    Date(DateOptions),
    Folder(FolderOptions),
    Number(NumberOptions),
    Extension(ExtensionOptions),
}

impl Ord for Renamer {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.original.is_dir(), other.original.is_dir()) {
            (true, true) => self.stem.cmp(&other.stem),
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => match (&self.extension, &other.extension) {
                (None, None) => self.stem.cmp(&other.stem),
                (None, Some(ext)) => {
                    let mut rhs = other.stem.clone();
                    rhs.push_str(ext);
                    self.stem.cmp(&rhs)
                }
                (Some(ext), None) => {
                    let mut lhs = self.stem.clone();
                    lhs.push_str(ext);
                    lhs.cmp(&other.stem)
                }
                (Some(self_ext), Some(other_ext)) => {
                    let mut lhs = self.stem.clone();
                    lhs.push_str(self_ext);
                    let mut rhs = other.stem.clone();
                    rhs.push_str(other_ext);
                    lhs.cmp(&rhs)
                }
            },
        }
    }
}

impl PartialEq for Renamer {
    fn eq(&self, other: &Self) -> bool {
        self.original == other.original
    }
}

impl Eq for Renamer {}

impl PartialOrd for Renamer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod file_tests {
    use renamer_builder::RenamerBuilder;

    use super::*;

    #[test]
    fn test_regex() {
        let file = Path::new("Testfile123.txt");
        let expected = PathBuf::from("TestfileABC.txt");
        let opt = RegexOptions {
            exp: "123".into(),
            rep: "ABC".into(),
            extension: false,
        };
        let mut rename = Renamer::new(file).unwrap().with_option(Options::Regex(opt));
        let result = rename.preview();
        assert_eq!(result, expected)
    }

    #[test]
    fn test_name() {
        let file = Path::new("file.txt");
        let expected = PathBuf::from("new_name.txt");
        let name = NameOptions::Fixed("new_name".into());
        let mut rename = Renamer::new(file).unwrap().with_option(Options::Name(name));
        let new_name = rename.preview();
        assert_eq!(new_name, expected)
    }

    #[test]
    fn test_renamed_midway_through() {
        let file = Path::new("file_with_a_name.txt");
        let file2 = Path::new("file_with_a_name2.txt");
        let _ = fs::File::create(file);
        let _ = fs::rename(file, file2);
        let mut renamer = RenamerBuilder::new_unchecked(file)
            .with_replace("_".into(), "-".into(), false)
            .build();
        assert_eq!(Path::new("file-with-a-name.txt"), renamer.preview());
        assert!(matches!(renamer.rename(), Err(FileError::Io(_))));
        let _ = fs::remove_file(file2);
        let _ = fs::remove_file("file-with-a-name.txt");
    }
}
