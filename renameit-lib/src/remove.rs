use crate::{Process, Renamer};

/// Options for removing parts of the filename.
/// Remove specific parts of a filename but not file extensions.
///
/// - `First n` - Remove the first n characters from the name.
/// - `Last n` - Remove the last n characters from the name.
/// - `Range` - Remove a string of text, e.g. from the 6th to the 9th characters (0 indexed).
/// - `Characters` - Remove occurrences of the listed characters from the name (no separator needed).
/// - `Words` - Remove occurrences of listed words (separated by spaces).
/// - `Crop` - Remove any text which occurs before (or after) a specific character or word.
/// - `Digits` - Remove all occurrences of the digits 0-9 from the filename.
/// - `High` - Remove ASCII characters (chars from 128 to 255).
/// - `Trim` - Remove leading and trailing spaces.
/// - `D/S` - Remove occurrences of double spaces, and replace them with single spaces.
/// - `English Letters` - Remove all characters (matching regex a-zA-Z).
/// - `Sym` - Remove all symbols (~`!@#$%^&*()_-+={}[]|\/?"':;.,<>).
/// - `Lead Dots` - Remove "." from the front of filenames.
///
/// Note: When you use the `words` option, you have the ability of specifying a special
/// value using the wildcard (*). This will remove the specified string, and any
/// characters occupied by the wildcard. So for example, specifying \[*\] would convert
/// "Hello\[ABC\] Joe" to just "Hello Joe", as it has removed the two square brackets and
/// everything between. The wildcard can not be at the start or end of the word.
/// For that case use crop.
#[derive(Debug, Clone)]
pub struct RemoveOptions {
    pub first_n: usize,
    pub last_n: usize,
    pub range: (usize, usize),
    pub characters: String,
    pub words: String,
    pub crop: (bool, String),
    pub digits: bool,
    pub ascii_high: bool,
    pub trim: bool,
    pub double_space: bool,
    pub english_letters: bool,
    pub symbols: bool,
    pub lead_dots: bool,
}

impl Default for RemoveOptions {
    fn default() -> Self {
        Self {
            first_n: Default::default(),
            last_n: Default::default(),
            range: Default::default(),
            characters: Default::default(),
            words: Default::default(),
            crop: (true, String::new()),
            digits: Default::default(),
            ascii_high: Default::default(),
            trim: Default::default(),
            double_space: Default::default(),
            english_letters: Default::default(),
            symbols: Default::default(),
            lead_dots: Default::default(),
        }
    }
}

impl Process for RemoveOptions {
    fn process(&self, file: &mut Renamer) {
        let file = &mut file.stem;
        if self.first_n + self.last_n > 0 {
            self.first_last(file)
        }
        if self.range.0 < file.len() && self.range.1 > self.range.0 {
            self.start_end(file)
        }

        if !self.characters.is_empty() {
            for chr in self.characters.chars() {
                self.remove_char(file, chr);
            }
        }

        if !self.words.is_empty() {
            for word in self.words.split(' ') {
                self.remove_word(file, word);
            }
        }

        if !self.crop.1.is_empty() {
            let (before, position) = &self.crop;
            let pos = file.find(position);
            match (before, pos) {
                (true, Some(p)) => *file = file[p..].to_owned(),
                (false, Some(p)) => *file = file[..(p + position.len())].to_owned(),
                _ => (),
            }
        }

        if self.digits {
            for chr in "0123456789".chars() {
                self.remove_char(file, chr);
            }
        }

        if self.ascii_high {
            let chars = (128..=255).map(char::from);
            for chr in chars {
                self.remove_char(file, chr);
            }
        }

        if self.trim {
            *file = file.trim().to_owned();
        }

        if self.english_letters {
            let chars = (65..=90).map(char::from).chain((97..=122).map(char::from));
            for chr in chars {
                self.remove_char(file, chr)
            }
        }

        if self.symbols {
            for chr in r#"~`!@#$%^&*()_-+={}[]|\/?"':;.,<>"#.chars() {
                self.remove_char(file, chr)
            }
        }

        if self.lead_dots && file.starts_with('.') {
            file.remove(0);
        }

        if self.double_space {
            self.remove_double_spaces(file)
        }
    }
}

impl RemoveOptions {
    fn first_last(&self, file: &mut String) {
        let chars = file.chars().collect::<Vec<_>>();
        if self.first_n + self.last_n > chars.len() {
            file.clear()
        } else {
            let mut end = chars.len() - self.last_n;
            if end < self.first_n {
                end = self.first_n;
            }
            *file = chars[self.first_n..end].iter().collect();
        }
    }

    fn start_end(&self, file: &mut String) {
        use std::cmp::min;
        // Change from 1 indexed to 0 indexed.
        let mut chars = file.chars().collect::<Vec<_>>();
        let start = self.range.0;
        let end = min(chars.len(), self.range.1);
        if start < end {
            chars.drain(start..end);
        }
        *file = chars.iter().collect();
    }

    fn remove_char(&self, file: &mut String, chr: char) {
        *file = file.replace(chr, "");
    }

    fn remove_word(&self, file: &mut String, word: &str) {
        if word.contains('*') {
            let w = word.split('*').collect::<Vec<&str>>();
            let (start, end) = (w[0], w[1]);
            let start_idx = file.find(start);
            let end_idx = file.find(end);
            if let (Some(start_idx), Some(end_idx)) = (start_idx, end_idx) {
                let word = &file[start_idx..(end_idx + end.len())];
                *file = file.replace(word, "");
            }
        } else {
            *file = file.replace(word, "")
        }
    }

    fn remove_double_spaces(&self, file: &mut String) {
        while file.contains("  ") {
            *file = file.replace("  ", " ");
        }
    }
}

#[cfg(test)]
mod remove_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn combined_removals() {
        let first_n = 2;
        let last_n = 2;
        let range = (0, 2);
        let characters = "ft".into();
        let words = "ile w*h".into();
        let crop = (true, "".into());
        let digits = true;
        let ascii_high = true;
        let trim = true;
        let double_space = true;
        let chars = false;
        let symbols = true;
        let lead_dots = false;
        let mut file = Renamer::new(Path::new("some test file  1234withÃ!  testing")).unwrap();
        let opt = RemoveOptions {
            first_n,
            last_n,
            range,
            characters,
            words,
            crop,
            digits,
            ascii_high,
            trim,
            double_space,
            english_letters: chars,
            symbols,
            lead_dots,
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("es esi"))
    }

    #[test]
    fn test_too_many_removed_from_end() {
        let first_n = 6;
        let last_n = 4;
        let range = (0, 0);
        let characters = "".into();
        let words = "".into();
        let crop = (true, "".into());
        let digits = false;
        let ascii_high = false;
        let trim = false;
        let double_space = false;
        let chars = false;
        let symbols = false;
        let lead_dots = false;
        let mut file = Renamer::new(Path::new("test_file")).unwrap();
        let opt = RemoveOptions {
            first_n,
            last_n,
            range,
            characters,
            words,
            crop,
            digits,
            ascii_high,
            trim,
            double_space,
            english_letters: chars,
            symbols,
            lead_dots,
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from(""))
    }

    #[test]
    fn test_too_many_removed_total() {
        let first_n = 60;
        let last_n = 4;
        let range = (0, 0);
        let characters = "".into();
        let words = "".into();
        let crop = (true, "".into());
        let digits = false;
        let ascii_high = false;
        let trim = false;
        let double_space = false;
        let chars = false;
        let symbols = false;
        let lead_dots = false;
        let mut file = Renamer::new(Path::new("test_file")).unwrap();
        let opt = RemoveOptions {
            first_n,
            last_n,
            range,
            characters,
            words,
            crop,
            digits,
            ascii_high,
            trim,
            double_space,
            english_letters: chars,
            symbols,
            lead_dots,
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from(""))
    }

    #[test]
    fn crop_before() {
        let first_n = 0;
        let last_n = 0;
        let range = (0, 0);
        let characters = "".into();
        let words = "".into();
        let crop = (true, "to".into());
        let digits = false;
        let ascii_high = false;
        let trim = false;
        let double_space = false;
        let chars = false;
        let symbols = false;
        let lead_dots = true;
        let mut file = Renamer::new(Path::new("file to test")).unwrap();
        let opt = RemoveOptions {
            first_n,
            last_n,
            range,
            characters,
            words,
            crop,
            digits,
            ascii_high,
            trim,
            double_space,
            english_letters: chars,
            symbols,
            lead_dots,
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("to test"));
    }

    #[test]
    fn remove_chars_lead_dot() {
        let first_n = 0;
        let last_n = 0;
        let range = (0, 0);
        let characters = "".into();
        let words = "".into();
        let crop = (true, "".into());
        let digits = false;
        let ascii_high = false;
        let trim = false;
        let double_space = false;
        let chars = true;
        let symbols = false;
        let lead_dots = true;
        let mut file = Renamer::new(Path::new("./.file123")).unwrap();
        let opt = RemoveOptions {
            first_n,
            last_n,
            range,
            characters,
            words,
            crop,
            digits,
            ascii_high,
            trim,
            double_space,
            english_letters: chars,
            symbols,
            lead_dots,
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("123"));
    }

    #[test]
    fn crop_after_found() {
        let first_n = 0;
        let last_n = 0;
        let range = (0, 0);
        let characters = "".into();
        let words = "".into();
        let crop = (false, "file".into());
        let digits = false;
        let ascii_high = false;
        let trim = false;
        let double_space = false;
        let chars = false;
        let symbols = false;
        let lead_dots = true;
        let mut file = Renamer::new(Path::new(".file123")).unwrap();
        let opt = RemoveOptions {
            first_n,
            last_n,
            range,
            characters,
            words,
            crop,
            digits,
            ascii_high,
            trim,
            double_space,
            english_letters: chars,
            symbols,
            lead_dots,
        };
        opt.process(&mut file);
        assert_eq!(file.stem, String::from("file"));
    }
}
