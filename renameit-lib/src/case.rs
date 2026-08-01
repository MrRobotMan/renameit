use crate::{Process, Renamer};
use inflector::Inflector;

/// Change the case of the file.
/// - `Keep` - Do change the capitalization (default).
/// - `Lower` - change all selected files to lowercase.
/// - `Upper` - CHANGE ALL SELECTED FILES TO UPPERCASE.
/// - `Title` - Change All Selected Files To Title Case.
/// - `Sentence` - Change all selected files to sentence case.
/// - `Snake` - Flag_to_change_all_selected_files_to_snake_case.
///
/// Exceptions: You can also enter a list of "exceptions", separated by semicolons.
/// So for example if you entered PDF;doc then any occurrence of pdf (or PDF, Pdf,
/// etc) would be converted to upper-case, and every occurrence of DOC (or DoC)
/// would become doc.
#[derive(Default, Debug, Clone)]
pub struct CaseOptions {
    pub case: Case,
    pub snake: bool,
    pub exceptions: String,
}

/// Select from
/// `Case::Keep` to not change case (default),
/// `Case::Lower` to convert to lowercase,
/// `Case::Upper` to convert to uppercase,
/// `Case::Title` to convert to titlecase, or
/// `Case::Sentence` to convert to sentence case.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Case {
    #[default]
    Keep,
    Lower,
    Upper,
    Title,
    Sentence,
}

impl Case {
    pub fn iterator() -> impl Iterator<Item = Case> {
        [
            Self::Keep,
            Self::Lower,
            Self::Upper,
            Self::Title,
            Self::Sentence,
        ]
        .iter()
        .copied()
    }
}

impl std::fmt::Display for Case {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Case::Keep => "Keep",
            Case::Lower => "Lower",
            Case::Upper => "Upper",
            Case::Title => "Title",
            Case::Sentence => "Sentence",
        })
    }
}

impl Process for CaseOptions {
    fn process(&self, file: &mut Renamer) {
        match self.case {
            Case::Keep => (),
            Case::Lower => {
                file.stem = file.stem.to_lowercase();
            }
            Case::Upper => {
                file.stem = file.stem.to_uppercase();
            }
            Case::Title => {
                file.stem = file.stem.to_title_case();
            }
            Case::Sentence => {
                file.stem = file.stem.to_sentence_case();
            }
        };
        if !&self.exceptions.is_empty() {
            for exception in self.exceptions.split(';') {
                let mod_exception = match self.case {
                    Case::Keep => exception.to_owned(),
                    Case::Lower => exception.to_lowercase(),
                    Case::Upper => exception.to_uppercase(),
                    Case::Title => exception.to_title_case(),
                    Case::Sentence => exception.to_sentence_case(),
                };
                file.stem = file.stem.replace(&mod_exception, exception);
            }
        }
        if self.snake {
            file.stem = file.stem.replace(' ', "_")
        };
    }
}

#[cfg(test)]
mod case_tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn test_keep_case() {
        let mut file = Renamer::new(Path::new("test file")).unwrap();
        let opt = CaseOptions {
            case: Case::Keep,
            snake: false,
            exceptions: String::new(),
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("test file"));
    }

    #[test]
    fn test_keep_case_snake() {
        let mut file = Renamer::new(Path::new("test file")).unwrap();
        let opt = CaseOptions {
            case: Case::Keep,
            snake: true,
            exceptions: String::new(),
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("test_file"));
    }

    #[test]
    fn test_lower_case() {
        let mut file = Renamer::new(Path::new("TEST FILE")).unwrap();
        let opt = CaseOptions {
            case: Case::Lower,
            snake: false,
            exceptions: String::new(),
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("test file"));
    }

    #[test]
    fn test_lower_case_snake() {
        let mut file = Renamer::new(Path::new("TEST FILE")).unwrap();
        let opt = CaseOptions {
            case: Case::Lower,
            snake: true,
            exceptions: String::new(),
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("test_file"));
    }

    #[test]
    fn test_upper_case() {
        let mut file = Renamer::new(Path::new("test file")).unwrap();
        let opt = CaseOptions {
            case: Case::Upper,
            snake: false,
            exceptions: String::new(),
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("TEST FILE"));
    }

    #[test]
    fn test_upper_case_snake() {
        let mut file = Renamer::new(Path::new("test file")).unwrap();
        let opt = CaseOptions {
            case: Case::Upper,
            snake: true,
            exceptions: String::new(),
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("TEST_FILE"));
    }

    #[test]
    fn test_title_case() {
        let mut file = Renamer::new(Path::new("test file")).unwrap();
        let opt = CaseOptions {
            case: Case::Title,
            snake: false,
            exceptions: String::new(),
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("Test File"));
    }

    #[test]
    fn test_title_case_snake() {
        let mut file = Renamer::new(Path::new("test file")).unwrap();
        let opt = CaseOptions {
            case: Case::Title,
            snake: true,
            exceptions: String::new(),
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("Test_File"));
    }

    #[test]
    fn test_sentence_case() {
        let mut file = Renamer::new(Path::new("test file")).unwrap();
        let opt = CaseOptions {
            case: Case::Sentence,
            snake: false,
            exceptions: String::new(),
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("Test file"));
    }

    #[test]
    fn test_sentence_case_snake() {
        let mut file = Renamer::new(Path::new("test file")).unwrap();
        let opt = CaseOptions {
            case: Case::Sentence,
            snake: true,
            exceptions: String::new(),
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("Test_file"));
    }

    #[test]
    fn test_exceptions_with_upper() {
        let mut files = (
            Renamer::new(Path::new("test file.doc.bak")).unwrap(),
            Renamer::new(Path::new("test file.pdf.bak")).unwrap(),
        );
        let opt = CaseOptions {
            case: Case::Upper,
            snake: false,
            exceptions: "doc;PDF".into(),
        };
        opt.process(&mut files.0);
        opt.process(&mut files.1);
        let expected = (String::from("TEST FILE.doc"), String::from("TEST FILE.PDF"));
        assert_eq!((files.0.stem, files.1.stem), expected);
    }
}
