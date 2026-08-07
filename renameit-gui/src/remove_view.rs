use iced::{
    Element,
    widget::{button, checkbox, column, radio, row, text_input},
};
use iced_aw::widget::{LabeledFrame, NumberInput};
use renameit_lib::{RemoveOptions, RenameOption};

use crate::{Action, OptionBox, input_field};

#[derive(Debug, Clone)]
pub struct RemoveView {
    first: usize,
    last: usize,
    start: usize,
    end: usize,
    chars: String,
    words: String,
    location: Option<bool>,
    crop: String,
    digits: bool,
    ascii_high: bool,
    trim: bool,
    double_space: bool,
    english: bool,
    symbols: bool,
    lead_dots: bool,
}

impl Default for RemoveView {
    fn default() -> Self {
        Self {
            first: Default::default(),
            last: Default::default(),
            start: Default::default(),
            end: Default::default(),
            chars: Default::default(),
            words: Default::default(),
            location: Some(true),
            crop: Default::default(),
            digits: Default::default(),
            ascii_high: Default::default(),
            trim: Default::default(),
            double_space: Default::default(),
            english: Default::default(),
            symbols: Default::default(),
            lead_dots: Default::default(),
        }
    }
}

impl OptionBox for RemoveView {
    fn to_options(&self) -> Box<dyn Fn(usize) -> RenameOption + Send + Sync> {
        let opt = RemoveOptions {
            first_n: self.first,
            last_n: self.last,
            range: (self.start, self.end),
            characters: self.chars.clone(),
            words: self.words.clone(),
            crop: (self.location.unwrap_or(true), self.crop.clone()),
            digits: self.digits,
            ascii_high: self.ascii_high,
            trim: self.trim,
            double_space: self.double_space,
            english_letters: self.english,
            symbols: self.symbols,
            lead_dots: self.lead_dots,
        };
        Box::new(move |_| RenameOption::Remove(opt.clone()))
    }
}

impl RemoveView {
    pub fn view(&self) -> Element<'_, Message> {
        LabeledFrame::new(
            "Remove",
            column![
                row![
                    input_field(
                        "First n",
                        NumberInput::new(&self.first, 0..usize::MAX, |n| Message::Num(
                            NumField::First,
                            n
                        ))
                        .into()
                    ),
                    input_field(
                        "Last n",
                        NumberInput::new(&self.last, 0..usize::MAX, |n| Message::Num(
                            NumField::Last,
                            n
                        ))
                        .into()
                    ),
                ],
                row![
                    input_field(
                        "From",
                        NumberInput::new(&self.start, 0..usize::MAX, |n| Message::Num(
                            NumField::Start,
                            n
                        ))
                        .into()
                    ),
                    input_field(
                        "To",
                        NumberInput::new(&self.end, 0..usize::MAX, |n| Message::Num(
                            NumField::End,
                            n
                        ))
                        .into()
                    ),
                ],
                row![
                    input_field(
                        "Chars",
                        text_input(&self.chars, &self.chars)
                            .on_input(|v| Message::Str(StrField::Char, v))
                            .into()
                    ),
                    input_field(
                        "Words",
                        text_input(&self.words, &self.words)
                            .on_input(|v| Message::Str(StrField::Word, v))
                            .into()
                    ),
                ],
                input_field(
                    "Crop",
                    row![
                        text_input(&self.crop, &self.crop)
                            .on_input(|v| Message::Str(StrField::Crop, v)),
                        column![
                            radio("Before", true, self.location, |b| Message::Bool(
                                BoolField::Location,
                                b
                            )),
                            radio("After", false, self.location, |b| Message::Bool(
                                BoolField::Location,
                                b
                            )),
                        ]
                    ]
                    .into()
                ),
                row![
                    input_field(
                        "Digits",
                        checkbox(self.digits)
                            .on_toggle(|b| Message::Bool(BoolField::Digits, b))
                            .into()
                    ),
                    input_field(
                        "Chars",
                        checkbox(self.english)
                            .on_toggle(|b| Message::Bool(BoolField::Letters, b))
                            .into()
                    ),
                    input_field(
                        "Syms",
                        checkbox(self.symbols)
                            .on_toggle(|b| Message::Bool(BoolField::Symbols, b))
                            .into()
                    ),
                ],
                row![
                    input_field(
                        "Trim",
                        checkbox(self.trim)
                            .on_toggle(|b| Message::Bool(BoolField::Trim, b))
                            .into()
                    ),
                    input_field(
                        "Extra Sp",
                        checkbox(self.double_space)
                            .on_toggle(|b| Message::Bool(BoolField::ExtraSpaces, b))
                            .into()
                    ),
                ],
                row![
                    input_field(
                        "Ascii High",
                        checkbox(self.ascii_high)
                            .on_toggle(|b| Message::Bool(BoolField::Ascii, b))
                            .into()
                    ),
                    input_field(
                        "Lead Dots",
                        checkbox(self.lead_dots)
                            .on_toggle(|b| Message::Bool(BoolField::Dots, b))
                            .into()
                    ),
                ],
                button("Clear").on_press(Message::Reset)
            ],
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Reset => {
                *self = Self::default();
                return Action::Remove;
            }
            Message::Str(field, value) => self.update_string(field, value),
            Message::Num(field, value) => self.update_number(field, value),
            Message::Bool(field, value) => self.update_bool(field, value),
        }
        Action::Update
    }

    fn update_string(&mut self, field: StrField, value: String) {
        match field {
            StrField::Char => self.chars = value,
            StrField::Crop => self.crop = value,
            StrField::Word => self.words = value,
        }
    }

    fn update_number(&mut self, field: NumField, value: usize) {
        match field {
            NumField::First => self.first = value,
            NumField::Last => self.last = value,
            NumField::Start => {
                self.start = value;
                if self.start > self.end {
                    self.end = self.start;
                };
            }
            NumField::End => {
                self.end = value;
                if self.end < self.start {
                    self.start = self.end;
                };
            }
        }
    }

    fn update_bool(&mut self, field: BoolField, value: bool) {
        match field {
            BoolField::Location => self.location = Some(value),
            BoolField::Digits => self.digits = value,
            BoolField::Ascii => self.ascii_high = value,
            BoolField::Trim => self.trim = value,
            BoolField::ExtraSpaces => self.double_space = value,
            BoolField::Letters => self.english = value,
            BoolField::Symbols => self.symbols = value,
            BoolField::Dots => self.lead_dots = value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    Str(StrField, String),
    Num(NumField, usize),
    Bool(BoolField, bool),
}

#[derive(Debug, Copy, Clone)]
pub enum StrField {
    Char,
    Crop,
    Word,
}

#[derive(Debug, Copy, Clone)]
pub enum NumField {
    First,
    Last,
    Start,
    End,
}

#[derive(Debug, Copy, Clone)]
pub enum BoolField {
    Location,
    Digits,
    Ascii,
    Trim,
    ExtraSpaces,
    Letters,
    Symbols,
    Dots,
}
