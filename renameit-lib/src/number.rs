use super::{Process, Renamer};
use std::fmt::Write;

/// Add sequential numbers to the file.
/// - `Mode` - Choose between prefix, suffix, both, or insert at a location (0 indexed).
/// - `Value` - Specify a value number for the numbering.
/// - `Step` - The number to be added to the previous.
/// - `Pad` - The minimum number of digits occupied by the numeric element.
/// - `Char` - The character to use for padding. By default, numeric bases will be padded with leading zeros; the a-z and A-Z options will be padded with "a" or "A" as appropriate.
/// - `Sep`. - A character or characters that you wish to be inserted between the old filename and the number. If you enter the special character ":" (colon) in the Sep. box then this will be replaced with the auto-number. So a separator value of ABC:DEF: would result in ABC1DEF1, ABC2ABC2 etc.
/// - `Format` - You can choose to append the auto-number in any various bases (binary, decimal, hex (upper and lower), octal), ASCII letters A-Z.
#[derive(Default, Debug, Clone)]
pub struct NumberOptions {
    pub mode: NumberMode,
    pub value: u32,
    pub pad: usize,
    pub char: char,
    pub sep: String,
    pub format: NumberFormat,
}

impl Process for NumberOptions {
    fn process(&self, file: &mut Renamer) {
        let val = self.number_value();
        match self.mode {
            NumberMode::Prefix => file.stem.insert_str(0, &format!("{}{}", val, self.sep)),
            NumberMode::Suffix => write!(file.stem, "{}{}", self.sep, val)
                .expect("Unexpected error appending string."),
            NumberMode::Insert(idx) => {
                let mut chars = file.stem.chars().collect::<Vec<_>>();
                let idx = idx.min(chars.len());
                chars.splice(idx..idx, format!("{}{}{}", self.sep, val, self.sep).chars());
                file.stem = chars.iter().collect();
            }
        };
    }
}

impl NumberOptions {
    fn number_value(&self) -> String {
        let replace = match &self.format {
            NumberFormat::Decimal => format!("{}", self.value),
            NumberFormat::Binary => format!("{:b}", self.value),
            NumberFormat::Octal => format!("{:o}", self.value),
            NumberFormat::HexUpper => format!("{:X}", self.value),
            NumberFormat::HexLower => format!("{:x}", self.value),
            f => {
                // ASCII Upper or lower
                let offset = match f {
                    NumberFormat::AsciiLower => 96_u8,
                    _ => 64_u8,
                };
                let mut res: Vec<char> = Vec::new();
                let mut val = self.value;
                while val > 0 {
                    val -= 1;
                    res.push(char::from((val % 26) as u8 + offset));
                    val /= 26;
                }
                res.reverse();
                res.into_iter().collect::<String>()
            }
        };
        if self.pad > replace.len() {
            let mut val =
                std::iter::repeat_n(self.char, self.pad - replace.len()).collect::<String>();
            val.push_str(&replace);
            val
        } else {
            replace
        }
    }
}

/// Select from
/// `NumberMode::Prefix`,
/// `NumberMode::Suffix`, or
/// `NumberMode::Insert(usize)`.
#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub enum NumberMode {
    #[default]
    Prefix,
    Suffix,
    Insert(usize),
}

/// Select from
/// `NumberFormat:Binary`,
/// `NumberFormat:Decimal`,
/// `NumberFormat:HexUpper`,
/// `NumberFormat:HexLower`,
/// `NumberFormat:Octal`,
/// `NumberFormat:AsciiUpper`, or
/// `NumberFormat:AsciiLower`
#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub enum NumberFormat {
    Binary,
    #[default]
    Decimal,
    HexUpper,
    HexLower,
    Octal,
    AsciiUpper,
    AsciiLower,
}

/*
#[derive(Default)]
pub struct NumberView {
    mode: NumberMode,
    position: ValText<usize>,
    start: ValText<u32>,
    increment: ValText<u32>,
    pad: ValText<usize>,
    padding_char: ValText<char>,
    sep: String,
    reset_pos: ValText<usize>,
    format: NumberFormat,
    width: f32,
}

impl NumberView {
    pub fn new(width: f32) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }
}

impl OptionBuilder for NumberView {
    type Processor = NumberOptions;

    fn build(&self) -> NumberOptions {
        NumberOptions {
            mode: self.mode,
            value: self.start.get_val().unwrap_or(0),
            pad: self.pad.get_val().unwrap_or(0),
            char: self.padding_char.get_val().unwrap_or(match self.format {
                NumberFormat::Binary
                | NumberFormat::Decimal
                | NumberFormat::HexUpper
                | NumberFormat::HexLower
                | NumberFormat::Octal => '0',
                NumberFormat::AsciiUpper => 'A',
                NumberFormat::AsciiLower => 'a',
            }),
            sep: self
                .sep
                .replace(':', &format!("{}", self.start.get_val().unwrap_or(0))),
            format: self.format,
        }
    }
}

impl Incrementer for &mut NumberView {
    fn increment(&mut self, field: &str) {
        match field {
            "start" => self.start.set_val(match self.start.get_val() {
                Some(v) => v + 1,
                None => 1,
            }),
            "increment" => self.increment.set_val(match self.increment.get_val() {
                Some(v) => v + 1,
                None => 1,
            }),
            "pad" => self.pad.set_val(match self.pad.get_val() {
                Some(v) => v + 1,
                None => 1,
            }),
            "reset_pos" => self.reset_pos.set_val(match self.reset_pos.get_val() {
                Some(v) => v + 1,
                None => 1,
            }),
            _ => {}
        }
    }

    fn decrement(&mut self, field: &str) {
        match field {
            "start" => self.start.set_val(match self.start.get_val() {
                Some(v) => v.saturating_sub(1),
                None => 0,
            }),
            "increment" => self.increment.set_val(match self.increment.get_val() {
                Some(v) => v.saturating_sub(1),
                None => 0,
            }),
            "pad" => self.pad.set_val(match self.pad.get_val() {
                Some(v) => v.saturating_sub(1),
                None => 0,
            }),
            "reset_pos" => self.reset_pos.set_val(match self.reset_pos.get_val() {
                Some(v) => v.saturating_sub(1),
                None => 0,
            }),
            _ => {}
        }
    }
}

impl Widget for &mut NumberView {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.set_width(self.width);
            ui.label("Numbering");
            ui.horizontal(|ui| {
                ui.set_width(self.width);
                ui.label("Mode");
                let response = ComboBox::from_id_source("Number Mode")
                    .selected_text(match self.mode {
                        NumberMode::Prefix => "Prefix",
                        NumberMode::Suffix => "Suffix",
                        NumberMode::Insert(_) => "Insert",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.mode, NumberMode::Prefix, "Prefix");
                        ui.selectable_value(&mut self.mode, NumberMode::Suffix, "Suffix");
                        ui.selectable_value(&mut self.mode, NumberMode::Insert(0), "Insert");
                    })
                    .response;
                if response.changed() {
                    match &self.mode {
                        NumberMode::Insert(_) => {
                            self.mode = NumberMode::Insert(self.position.get_val().unwrap_or(0));
                        }
                        _ => self.position.set_val(0),
                    }
                };
                ui.label("at:");
                if ui.text_edit_singleline(&mut self.position).changed() {
                    if self.position.is_empty() {
                        self.mode = NumberMode::Prefix;
                    } else if !self.position.is_valid() {
                        self.position.revert()
                    } else {
                        self.mode = NumberMode::Insert(self.position.get_val().unwrap());
                    };
                }
            });
            ui.horizontal(|ui| {
                ui.set_width(self.width);
                ui.label("Start");
                if ui
                    .add(TextEdit::singleline(&mut self.start).desired_width(NUM_WIDTH * 3.0))
                    .changed()
                    && !self.start.is_valid()
                    && !self.start.is_empty()
                {
                    self.start.revert();
                }
                ui.add(Arrows::new("Number Start", &mut self, "start"));
                ui.label("Incr.");
                if ui
                    .add(TextEdit::singleline(&mut self.increment).desired_width(NUM_WIDTH * 3.0))
                    .changed()
                    && !self.increment.is_valid()
                    && !self.increment.is_empty()
                {
                    self.increment.revert();
                }
                ui.add(Arrows::new("Number Increment", &mut self, "increment"));
            });
            ui.horizontal(|ui| {
                ui.set_width(self.width);
                ui.label("Pad");
                if ui
                    .add(TextEdit::singleline(&mut self.pad).desired_width(NUM_WIDTH * 3.0))
                    .changed()
                    && !self.pad.is_valid()
                    && !self.pad.is_empty()
                {
                    self.pad.revert();
                }
                ui.add(Arrows::new("Number Pad", &mut self, "pad"));
                ui.label("Char");
                if ui
                    .add(
                        TextEdit::singleline(&mut self.padding_char).desired_width(NUM_WIDTH * 3.0),
                    )
                    .changed()
                    && !self.padding_char.is_valid()
                    && !self.padding_char.is_empty()
                {
                    if let Some(c) = self.padding_char.get_text().chars().last() {
                        self.padding_char.set_val(c)
                    };
                };
            });
            ui.horizontal(|ui| {
                ui.set_width(self.width);
                ui.label("Reset Val.");
                if ui.text_edit_singleline(&mut self.reset_pos).changed()
                    && !self.reset_pos.is_valid()
                    && !self.reset_pos.is_empty()
                {
                    self.reset_pos.revert();
                }
                ui.add(Arrows::new("Number Reset", &mut self, "reset_pos"));
            });
            ui.horizontal(|ui| {
                ui.set_width(self.width);
                ui.label("Format");
                ComboBox::from_id_source("Number Format")
                    .selected_text(match self.format {
                        NumberFormat::Binary => "Binary",
                        NumberFormat::Decimal => "Decimal",
                        NumberFormat::HexUpper => "Hex Upper",
                        NumberFormat::HexLower => "Hex Lower",
                        NumberFormat::Octal => "Octal",
                        NumberFormat::AsciiUpper => "A-Z",
                        NumberFormat::AsciiLower => "a-z",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.format, NumberFormat::Binary, "Binary");
                        ui.selectable_value(&mut self.format, NumberFormat::Octal, "Octal");
                        ui.selectable_value(&mut self.format, NumberFormat::Decimal, "Decimal");
                        ui.selectable_value(&mut self.format, NumberFormat::HexUpper, "Hex Upper");
                        ui.selectable_value(&mut self.format, NumberFormat::HexLower, "Hex Lower");
                        ui.selectable_value(&mut self.format, NumberFormat::AsciiUpper, "A-Z");
                        ui.selectable_value(&mut self.format, NumberFormat::AsciiLower, "a-z");
                    });
            });
        })
        .response
    }
}
*/

#[cfg(test)]
mod numbering_test {
    use super::*;
    use std::path::Path;

    fn vec_compare(va: &[String], vb: &[String]) -> bool {
        (va.len() == vb.len()) &&  // zip stops at the shortest
     va.iter()
       .zip(vb)
       .all(|(a,b)| a == b)
    }

    #[test]
    fn prefix_decimal_with_padding() {
        let mut files = (0..10)
            .map(|_| Renamer::new(Path::new("TestFile.txt")).unwrap())
            .collect::<Vec<Renamer>>();
        let pad = 2;
        let char = '0';
        let sep = "--";
        for (value, file) in files.iter_mut().enumerate() {
            let format = NumberFormat::Decimal;
            let mode = NumberMode::Prefix;
            let opt = NumberOptions {
                mode,
                value: (&value + 1) as u32,
                pad,
                char,
                sep: String::from(sep),
                format,
            };
            opt.process(file);
        }
        let expected = (1..=10)
            .map(|i| format!("{i:02}--TestFile"))
            .collect::<Vec<String>>();
        let result = files
            .iter()
            .map(|f| f.stem.clone())
            .collect::<Vec<String>>();
        assert!(vec_compare(&result, &expected));
    }

    #[test]
    fn suffix_binary_no_padding() {
        let mut file = Renamer::new(Path::new("TestFile.txt")).unwrap();
        let format = NumberFormat::Binary;
        let value = 5;
        let pad = 0;
        let char = '0';
        let sep = ".".into();
        let mode = NumberMode::Suffix;
        let opt = NumberOptions {
            mode,
            value,
            pad,
            char,
            sep,
            format,
        };
        opt.process(&mut file);
        assert_eq!(file.stem, "TestFile.101");
    }

    #[test]
    fn insert_asciiupper() {
        let mut file = Renamer::new(Path::new("TestFile.txt")).unwrap();
        let format = NumberFormat::AsciiUpper;
        let value = 50;
        let pad = 0;
        let char = '0';
        let sep = "_".into();
        let mode = NumberMode::Insert(4);
        let opt = NumberOptions {
            mode,
            value,
            pad,
            char,
            sep,
            format,
        };
        opt.process(&mut file);
        assert_eq!(file.stem, "Test_AX_File");
    }
}
