use iced::{
    Element,
    widget::{button, checkbox, column, row, text_input},
};
use iced_aw::widget::LabeledFrame;
use renameit_lib::{RenameOption, ReplaceOptions};

use crate::{Action, OptionBox, input_field};

#[derive(Debug, Clone, Default)]
pub struct ReplaceView {
    replace: String,
    with: String,
    case_sensative: bool,
}

impl OptionBox for ReplaceView {
    fn to_options(&self) -> Box<dyn Fn(usize) -> RenameOption + Send + Sync> {
        let opt = ReplaceOptions {
            replace: self.replace.clone(),
            with: self.with.clone(),
            case_sensative: self.case_sensative,
        };
        Box::new(move |_| RenameOption::Replace(opt.clone()))
    }
}

impl ReplaceView {
    pub fn view(&self) -> Element<'_, Message> {
        LabeledFrame::new(
            "Replace",
            column![
                input_field(
                    "Replace",
                    text_input("", &self.replace)
                        .on_input(|s| Message::Update(Field::Replace, s))
                        .into()
                ),
                input_field(
                    "With",
                    text_input("", &self.with)
                        .on_input(|s| Message::Update(Field::With, s))
                        .into()
                ),
                row![
                    input_field(
                        "Case\nSensative",
                        checkbox(self.case_sensative)
                            .on_toggle(Message::Toggle)
                            .into()
                    ),
                    button("Clear").on_press(Message::Reset)
                ]
            ],
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Update(Field::Replace, text) => self.replace = text,
            Message::Update(Field::With, text) => self.with = text,
            Message::Toggle(b) => self.case_sensative = b,
            Message::Reset => {
                self.replace = String::new();
                self.with = String::new();
                self.case_sensative = false;
                return Action::Remove;
            }
        }
        Action::Update
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Field {
    Replace,
    With,
}

#[derive(Debug, Clone)]
pub enum Message {
    Update(Field, String),
    Toggle(bool),
    Reset,
}
