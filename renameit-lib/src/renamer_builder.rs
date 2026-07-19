use std::collections::HashSet;

use super::date;
use super::*;

#[derive(Default, Debug)]
pub struct RenamerBuilder {
    renamer: Renamer,
}

impl RenamerBuilder {
    pub fn new_unchecked(path: &Path) -> Self {
        Self {
            renamer: Renamer::new(path).unwrap(),
        }
    }

    pub fn new(path: &Path) -> Result<Self, FileError> {
        Ok(Self {
            renamer: Renamer::new(path)?,
        })
    }

    pub fn build(self) -> Renamer {
        self.renamer
    }

    pub fn with_add(
        mut self,
        prefix: Option<String>,
        insert: Option<(i32, String)>,
        suffix: Option<String>,
        word_space: bool,
    ) -> Self {
        self.renamer.add = Some(AddOptions {
            prefix,
            insert,
            suffix,
            word_space,
        });
        self
    }

    pub fn with_case(mut self, case: Case, snake: bool, exceptions: String) -> Self {
        self.renamer.case = Some(CaseOptions {
            case,
            snake,
            exceptions,
        });
        self
    }

    pub fn with_date(
        mut self,
        date_mode: date::DateMode,
        date_type: date::DateType,
        fmt: date::DateFormat,
        sep: String,
        seg: String,
        full_year: bool,
    ) -> Self {
        self.renamer.date = Some(DateOptions {
            date_mode,
            date_type,
            fmt,
            sep,
            seg,
            full_year,
        });
        self
    }

    pub fn with_extension(mut self, ext: ExtensionOptions) -> Self {
        self.renamer.ext = Some(ext);
        self
    }

    pub fn with_folder(mut self, mode: folder::FolderMode, sep: String, levels: i32) -> Self {
        self.renamer.folder = Some(FolderOptions { mode, sep, levels });
        self
    }

    pub fn with_name(mut self, name: NameOptions) -> Self {
        self.renamer.name = Some(name);
        self
    }

    pub fn with_number(
        mut self,
        mode: number::NumberMode,
        value: u32,
        pad: usize,
        char: char,
        sep: String,
        format: number::NumberFormat,
    ) -> Self {
        self.renamer.number = Some(NumberOptions {
            mode,
            value,
            pad,
            char,
            sep,
            format,
        });
        self
    }

    pub fn with_reg(mut self, exp: String, rep: String, extension: bool) -> Self {
        self.renamer.regex = Some(RegexOptions {
            exp,
            rep,
            extension,
        });
        self
    }

    /// ranges: [first_n, last_n, range_start, range_end]
    /// toggles: [digits, ascii_high, trim, double_space, chars, symbols, lead_dots]
    /// crop: Before (true) or after (false) string
    pub fn with_remove(
        mut self,
        ranges: [usize; 4],
        characters: String,
        words: String,
        crop: (bool, String),
        toggles: HashSet<Toggle>,
    ) -> Self {
        let first_n = ranges[0];
        let last_n = ranges[1];
        let range = (ranges[2], ranges[3]);
        self.renamer.remove = Some(RemoveOptions {
            first_n,
            last_n,
            range,
            characters,
            words,
            crop,
            digits: toggles.contains(&Toggle::Digits),
            ascii_high: toggles.contains(&Toggle::AsciiHigh),
            trim: toggles.contains(&Toggle::Trim),
            double_space: toggles.contains(&Toggle::DoubleSpace),
            english_letters: toggles.contains(&Toggle::EnglishLetters),
            symbols: toggles.contains(&Toggle::Symbols),
            lead_dots: toggles.contains(&Toggle::LeadDots),
        });
        self
    }

    pub fn with_replace(mut self, replace: String, with: String, case_sensative: bool) -> Self {
        self.renamer.replace = Some(ReplaceOptions {
            replace,
            with,
            case_sensative,
        });
        self
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum Toggle {
    AsciiHigh,
    Digits,
    DoubleSpace,
    EnglishLetters,
    LeadDots,
    Symbols,
    Trim,
}
