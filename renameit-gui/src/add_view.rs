use crate::{Action, OptionBox, input_field};
use iced::{
    Element,
    widget::{button, checkbox, column, text_input},
};
use iced_aw::widget::{LabeledFrame, NumberInput};
use renameit_lib::{RenameOption, add::AddOptions};

#[derive(Default, Clone)]
pub struct AddView {
    prefix: String,
    insert: String,
    position: i32,
    suffix: String,
    word_space: bool,
}

#[derive(Copy, Clone)]
pub enum Field {
    Prefix,
    Insert,
    Suffix,
}

#[derive(Clone)]
pub enum Message {
    ChangedText((Field, String)),
    NumberChanged(i32),
    BoxToggle(bool),
    Reset,
}

impl OptionBox for AddView {
    fn to_options(&self, _index: usize) -> RenameOption {
        RenameOption::Add(AddOptions {
            prefix: (!self.prefix.is_empty()).then(|| self.prefix.clone()),
            insert: (!self.insert.is_empty()).then(|| (self.position, self.insert.clone())),
            suffix: (!self.suffix.is_empty()).then(|| self.suffix.clone()),
            word_space: self.word_space,
        })
    }
}

impl AddView {
    pub fn view(&self) -> Element<'_, Message> {
        LabeledFrame::new(
            "Add",
            column![
                input_field(
                    "Prefix",
                    text_input("", &self.prefix)
                        .on_input(|s| Message::ChangedText((Field::Prefix, s)))
                        .into()
                ),
                input_field(
                    "Insert",
                    text_input("", &self.insert)
                        .on_input(|s| Message::ChangedText((Field::Insert, s)))
                        .into()
                ),
                input_field(
                    "At",
                    NumberInput::new(&self.position, i32::MIN..=i32::MAX, Message::NumberChanged)
                        .into()
                ),
                input_field(
                    "Suffix",
                    text_input("", &self.suffix)
                        .on_input(|s| Message::ChangedText((Field::Suffix, s)))
                        .into()
                ),
                input_field(
                    "Word Space",
                    checkbox(self.word_space)
                        .on_toggle(Message::BoxToggle)
                        .into()
                ),
                button("Reset").on_press(Message::Reset)
            ],
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangedText((field, text)) => match field {
                Field::Prefix => self.prefix = text,
                Field::Insert => self.insert = text,
                Field::Suffix => self.suffix = text,
            },
            Message::NumberChanged(n) => self.position = n,
            Message::BoxToggle(b) => self.word_space = b,
            Message::Reset => {
                *self = Self::default();
                return Action::Remove;
            }
        }
        Action::Update
    }
}
