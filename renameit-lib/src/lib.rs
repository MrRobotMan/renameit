use std::{
    cmp::Ordering,
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
pub mod helpers;
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
use helpers::{PathString, generate_path_as_string};
pub use name::NameOptions;
pub use number::NumberOptions;
pub use reg::RegexOptions;
pub use remove::RemoveOptions;
pub use replace::ReplaceOptions;

pub trait Process {
    fn process(&self, file: &mut Renamer);
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
/// # use renameit_lib::{NameOptions, Case, CaseOptions, Renamer, Process, RenameOption};
/// let file = Path::new("file.txt");
/// let name = NameOptions::Fixed("new_name".into());
/// let case = CaseOptions{case: Case::Upper, snake: false, exceptions: "n".into()};
/// let mut rename = Renamer::new(file).unwrap().with_option(RenameOption::Name(name)).with_option(RenameOption::Case(case));
/// let new_name = rename.preview();
/// assert_eq!(new_name, PathBuf::from("nEW_nAME.txt"));
/// ```
#[derive(Debug, Default, Clone)]
pub struct Renamer {
    stem: String,
    renamed: String,
    original: PathBuf,
    valid_original: bool,
    extension: Option<String>,
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
    is_dir: bool,
    reverted: (String, Option<String>),
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
                let is_dir = path.is_dir();
                Ok(Self {
                    stem: stem.clone(),
                    valid_original,
                    extension: extension.clone(),
                    original: path.to_owned(),
                    reverted: (stem, extension),
                    is_dir,
                    ..Default::default()
                })
            }
            None => Err(FileError::BadStem),
        }
    }

    pub fn is_dir(&self) -> bool {
        self.is_dir
    }

    pub fn preview(&mut self) -> PathBuf {
        self.revert();
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
        // Note: If the renamed stem looks absolute ('/', 'C:\\', etc) it will
        // full replace the path ignoring all parents. This will be kept because it
        // will be obvious in the GUI automatic preview and ultimately it is then
        // the user's responsibility to make a better name. TODO: add a check for
        // illegal characters on Linux (/), Windows (< > : " / \ | ? *), and
        // MacOS (; /) as well as reserved names. See https://stackoverflow.com/questions/1976007/what-characters-are-forbidden-in-windows-and-linux-directory-names
        // for references.
        let mut new_name = PathBuf::from(&self.stem);
        new_name = match &self.extension {
            None => new_name,
            Some(e) => new_name.with_extension(e),
        };
        self.renamed = new_name.to_str().unwrap_or("NON-UTF NAME").to_string();
        new_name
    }

    /// Rename the file. Can not be undone.
    pub fn rename(mut self) -> Result<Self, (PathBuf, FileError)> {
        let new_name = self.preview();
        if let Err(e) = fs::rename(&self.original, &new_name) {
            return Err((self.original, e.into()));
        };
        Self::new(&new_name).map_err(|e| (new_name, e))
    }

    /// Revert the previewed changes to a file.
    pub fn revert(&mut self) {
        (self.stem, self.extension) = self.reverted.clone();
        self.renamed = String::new();
    }

    pub fn with_option(mut self, option: RenameOption) -> Self {
        use RenameOption::*;
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

    pub fn add_option(&mut self, option: RenameOption) {
        match option {
            RenameOption::Regex(opt) => self.regex = Some(opt),
            RenameOption::Name(opt) => self.name = Some(opt),
            RenameOption::Case(opt) => self.case = Some(opt),
            RenameOption::Remove(opt) => self.remove = Some(opt),
            RenameOption::Add(opt) => self.add = Some(opt),
            RenameOption::Date(opt) => self.date = Some(opt),
            RenameOption::Folder(opt) => self.folder = Some(opt),
            RenameOption::Number(opt) => self.number = Some(opt),
            RenameOption::Extension(opt) => self.ext = Some(opt),
        }
    }

    pub fn remove_option(&mut self, option: RenameOption) {
        match option {
            RenameOption::Regex(_) => self.regex = None,
            RenameOption::Name(_) => self.name = None,
            RenameOption::Case(_) => self.case = None,
            RenameOption::Remove(_) => self.remove = None,
            RenameOption::Add(_) => self.add = None,
            RenameOption::Date(_) => self.date = None,
            RenameOption::Folder(_) => self.folder = None,
            RenameOption::Number(_) => self.number = None,
            RenameOption::Extension(_) => self.ext = None,
        }
    }

    pub fn clear_options(&mut self) {
        self.regex = None;
        self.name = None;
        self.case = None;
        self.remove = None;
        self.add = None;
        self.date = None;
        self.folder = None;
        self.number = None;
        self.ext = None;
    }

    /// Return the information on a file.
    /// Returns (stem, renamed, extension, size, date modified, date created)
    pub fn info(
        &mut self,
    ) -> (
        Filename<'_>,
        Filename<'_>,
        Extension<'_>,
        Size,
        DateModified,
        DateCreated,
    ) {
        self.preview();
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
                let reverted = (stem.clone(), extension.clone());
                Ok(Self {
                    stem,
                    valid_original,
                    extension,
                    reverted,
                    original: path.to_owned(),
                    is_dir: path.is_dir(),
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

pub type Filename<'a> = &'a str;
pub type Extension<'a> = Option<&'a str>;
pub type Size = Option<u64>;
pub type DateCreated = Option<DateTime<Local>>;
pub type DateModified = Option<DateTime<Local>>;

#[derive(Debug, Clone)]
pub enum RenameOption {
    Add(AddOptions),
    Case(CaseOptions),
    Date(DateOptions),
    Extension(ExtensionOptions),
    Folder(FolderOptions),
    Name(NameOptions),
    Number(NumberOptions),
    Regex(RegexOptions),
    Remove(RemoveOptions),
}

impl Ord for Renamer {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.is_dir, other.is_dir) {
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
    use std::io;

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
        let mut rename = Renamer::new(file)
            .unwrap()
            .with_option(RenameOption::Regex(opt));
        let result = rename.preview();
        assert_eq!(result, expected)
    }

    #[test]
    fn test_name() {
        let file = Path::new("file.txt");
        let expected = PathBuf::from("new_name.txt");
        let name = NameOptions::Fixed("new_name".into());
        let mut rename = Renamer::new(file)
            .unwrap()
            .with_option(RenameOption::Name(name));
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
        assert!(matches!(
            renamer.rename(),
            Err((p, FileError::Io(e))) if p == *"file_with_a_name.txt" && e.kind() == io::ErrorKind::NotFound
        ));
        let _ = fs::remove_file(file2);
        let _ = fs::remove_file("file-with-a-name.txt");
    }

    #[test]
    fn test_preview_does_not_reapply_options() {
        let expected = PathBuf::from("aFile.txt");
        let mut renamer = RenamerBuilder::new_unchecked(Path::new("file.txt"))
            .with_case(Case::Title, false, String::new())
            .with_add(Some("a".into()), None, None, false)
            .build();
        let _ = renamer.preview();
        let actual = renamer.preview();
        assert_eq!(expected, actual);
    }
}
