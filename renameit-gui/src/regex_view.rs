use crate::{Action, OptionBox, input_field};
use iced::{
    Element,
    widget::{button, checkbox, column, text_input},
};
use iced_aw::widget::LabeledFrame;
use renameit_lib::{RenameOption, reg::RegexOptions};

#[derive(Default, Clone)]
pub struct RegexView {
    expression: String,
    replacement: String,
    include_extension: bool,
}

impl OptionBox for RegexView {
    fn to_options(&self) -> Box<dyn Fn(usize) -> RenameOption + Send + Sync> {
        let opt = RegexOptions {
            exp: self.expression.clone(),
            rep: self.replacement.clone(),
            extension: self.include_extension,
        };
        Box::new(move |_| RenameOption::Regex(opt.clone()))
    }
}

impl RegexView {
    pub fn view(&self) -> Element<'_, Message> {
        LabeledFrame::new(
            "Regex",
            column![
                input_field(
                    "Expression",
                    text_input("", &self.expression)
                        .on_input(|s| Message::ChangedText((Field::Expression, s)))
                        .into()
                ),
                input_field(
                    "Replacement",
                    text_input("", &self.replacement)
                        .on_input(|s| Message::ChangedText((Field::Replacement, s)))
                        .into()
                ),
                input_field(
                    "Include Extension",
                    checkbox(self.include_extension)
                        .on_toggle(Message::BoxToggle)
                        .into()
                ),
                button("Clear").on_press(Message::Reset)
            ],
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangedText((field, text)) => match field {
                Field::Expression => self.expression = text,
                Field::Replacement => self.replacement = text,
            },
            Message::BoxToggle(b) => self.include_extension = b,
            Message::Reset => {
                *self = Self::default();
                return Action::Remove;
            }
        }
        Action::Update
    }
}

#[derive(Copy, Clone)]
pub enum Field {
    Expression,
    Replacement,
}

#[derive(Clone)]
pub enum Message {
    ChangedText((Field, String)),
    BoxToggle(bool),
    Reset,
}
