use iced::{
    Element, Font,
    widget::{button, checkbox, column, combo_box, row, space, text_input},
};
use iced_aw::widget::{LabeledFrame, NumberInput};
use renameit_lib::{
    NumberOptions, RenameOption,
    number::{NumberFormat, NumberMode},
};

use crate::{Action, OptionBox, input_field};

#[derive(Debug, Clone)]
pub struct NumberView {
    modes: combo_box::State<Modes>,
    mode: Option<Modes>,
    formats: combo_box::State<Formats>,
    format: Option<Formats>,
    pos: usize,
    start: u32,
    step: u32,
    pad: usize,
    char: String,
    sep: String,
    upper: bool,
}

impl Default for NumberView {
    fn default() -> Self {
        Self {
            modes: combo_box::State::new(vec![
                Modes::None,
                Modes::Prefix,
                Modes::Suffix,
                Modes::Insert,
            ]),
            mode: Some(Modes::None),
            formats: combo_box::State::new(vec![
                Formats::Decimal,
                Formats::Binary,
                Formats::Octal,
                Formats::Hex,
                Formats::Ascii,
            ]),
            format: Some(Formats::Decimal),
            pos: Default::default(),
            start: 1,
            step: 1,
            pad: Default::default(),
            char: Default::default(),
            sep: Default::default(),
            upper: Default::default(),
        }
    }
}

impl OptionBox for NumberOptions {
    fn to_options(&self, index: usize) -> RenameOption {
        RenameOption::Number(if let NumberMode::Insert(i) = self.mode {
            Self {
                mode: NumberMode::Insert(index + i),
                ..self.clone()
            }
        } else {
            self.clone()
        })
    }
}

impl NumberView {
    pub fn to_option_box(&self) -> NumberOptions {
        NumberOptions {
            mode: match self.mode.unwrap_or_default() {
                Modes::Prefix | Modes::None => NumberMode::Prefix,
                Modes::Suffix => NumberMode::Suffix,
                Modes::Insert => NumberMode::Insert(self.pos),
            },
            value: self.start + self.step,
            pad: self.pad,
            char: self.char.chars().next().unwrap_or('0'),
            sep: self.sep.clone(),
            format: match self.format.unwrap_or_default() {
                Formats::Decimal => NumberFormat::Decimal,
                Formats::Binary => NumberFormat::Binary,
                Formats::Octal => NumberFormat::Octal,
                Formats::Hex if self.upper => NumberFormat::HexUpper,
                Formats::Hex => NumberFormat::HexLower,
                Formats::Ascii if self.upper => NumberFormat::AsciiUpper,
                Formats::Ascii => NumberFormat::AsciiLower,
            },
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        LabeledFrame::new(
            "Auto-Numbering",
            column![
                row![
                    input_field(
                        "Mode",
                        combo_box(&self.modes, "", self.mode.as_ref(), Message::Mode)
                            .icon(text_input::Icon {
                                font: Font::default(),
                                code_point: '▾',
                                size: None,
                                spacing: 5.0,
                                side: text_input::Side::Right,
                            })
                            .into()
                    ),
                    {
                        let w: Element<'_, Message> = if matches!(self.mode, Some(Modes::Insert)) {
                            input_field(
                                "At",
                                NumberInput::new(&self.pos, 0..=usize::MAX, Message::Pos).into(),
                            )
                            .into()
                        } else {
                            space::horizontal().into()
                        };
                        w
                    }
                ],
                row![
                    input_field(
                        "Start",
                        NumberInput::new(&self.start, 0..=(u32::MAX - self.step), Message::Start)
                            .into()
                    ),
                    input_field(
                        "Step",
                        NumberInput::new(&self.step, 1..=u32::MAX, Message::Step).into()
                    ),
                ],
                row![
                    input_field(
                        "Pad",
                        NumberInput::new(&self.pad, 0..=usize::MAX, Message::Pad).into()
                    ),
                    input_field(
                        "Char.",
                        text_input(&self.sep, &self.sep)
                            .on_input(Message::Char)
                            .into()
                    ),
                ],
                row![
                    input_field(
                        "Format",
                        combo_box(&self.formats, "", self.format.as_ref(), Message::Format)
                            .icon(text_input::Icon {
                                font: Font::default(),
                                code_point: '▾',
                                size: None,
                                spacing: 5.0,
                                side: text_input::Side::Right,
                            })
                            .into()
                    ),
                    {
                        let w: Element<'_, Message> = match self.format {
                            Some(Formats::Hex) | Some(Formats::Ascii) => input_field(
                                "Upper",
                                checkbox(self.upper).on_toggle(Message::Upper).into(),
                            )
                            .into(),
                            _ => space::horizontal().into(),
                        };
                        w
                    }
                ],
                input_field(
                    "Sep",
                    text_input(&self.sep, &self.sep)
                        .on_input(Message::Sep)
                        .into()
                ),
                button("Clear").on_press(Message::Reset)
            ],
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        if !matches!(message, Message::Reset | Message::Mode(_))
            && matches!(self.mode, Some(Modes::None))
        {
            self.mode = Some(Modes::Prefix);
        }
        match message {
            Message::Reset => {
                *self = Self::default();
                return Action::Remove;
            }
            Message::Char(s) => {
                self.char = match s.chars().count() {
                    0 => String::new(),
                    1 => s,
                    _ => s.chars().last().map(|c| c.to_string()).unwrap_or_default(),
                }
            }
            Message::Format(f) => self.format = Some(f),
            Message::Mode(m) => self.mode = Some(m),
            Message::Pad(n) => self.pad = n,
            Message::Pos(n) => self.pos = n,
            Message::Sep(s) => self.sep = s,
            Message::Start(n) => self.start = n,
            Message::Step(n) => self.step = n,
            Message::Upper(b) => self.upper = b,
        }
        Action::Update
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Char(String),
    Format(Formats),
    Mode(Modes),
    Pad(usize),
    Pos(usize),
    Reset,
    Sep(String),
    Start(u32),
    Step(u32),
    Upper(bool),
}

#[derive(Debug, Default, Copy, Clone)]
pub enum Modes {
    #[default]
    None,
    Prefix,
    Suffix,
    Insert,
}

#[derive(Debug, Default, Copy, Clone)]
pub enum Formats {
    #[default]
    Decimal,
    Binary,
    Octal,
    Hex,
    Ascii,
}

impl std::fmt::Display for Modes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Modes::None => "None",
            Modes::Prefix => "Prefix",
            Modes::Suffix => "Suffix",
            Modes::Insert => "Insert",
        })
    }
}

impl std::fmt::Display for Formats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Formats::Decimal => "Decimal",
            Formats::Binary => "Binary",
            Formats::Octal => "Octal",
            Formats::Hex => "Hex",
            Formats::Ascii => "A-Z",
        })
    }
}
