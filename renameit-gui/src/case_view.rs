use iced::{
    Element, Font,
    widget::{button, checkbox, column, combo_box, row, text_input},
};
use iced_aw::widget::LabeledFrame;
use renameit_lib::{Case, CaseOptions, RenameOption};

use crate::{Action, OptionBox, input_field};

#[derive(Debug, Clone)]
pub struct CaseView {
    case: combo_box::State<Case>,
    selected: Option<Case>,
    snake: bool,
    exceptions: String,
}

impl Default for CaseView {
    fn default() -> Self {
        Self {
            case: combo_box::State::new(vec![
                Case::Keep,
                Case::Lower,
                Case::Upper,
                Case::Title,
                Case::Sentence,
            ]),
            selected: Some(Case::Keep),
            snake: Default::default(),
            exceptions: Default::default(),
        }
    }
}

impl OptionBox for CaseOptions {
    fn to_options(&self, _index: usize) -> RenameOption {
        RenameOption::Case(self.clone())
    }
}

impl CaseView {
    pub fn to_option_box(&self) -> CaseOptions {
        CaseOptions {
            case: self.selected.unwrap_or(Case::Keep),
            snake: self.snake,
            exceptions: self.exceptions.clone(),
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        LabeledFrame::new(
            "Case",
            column![
                row![
                    combo_box(
                        &self.case,
                        "",
                        self.selected.as_ref(),
                        Message::ChangeSelected
                    )
                    .icon(text_input::Icon {
                        font: Font::default(),
                        code_point: '▾',
                        size: None,
                        spacing: 5.0,
                        side: text_input::Side::Right,
                    }),
                    input_field(
                        "Snake\nCase",
                        checkbox(self.snake).on_toggle(Message::Toggle).into()
                    )
                ],
                input_field(
                    "Except",
                    text_input("", &self.exceptions)
                        .on_input(Message::Update)
                        .into()
                ),
                button("Clear").on_press(Message::Reset)
            ]
            .width(200),
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangeSelected(case) => self.selected = Some(case),
            Message::Reset => {
                self.selected = Some(Case::Keep);
                self.exceptions = String::new();
                return Action::Remove;
            }
            Message::Toggle(b) => self.snake = b,
            Message::Update(text) => self.exceptions = text,
        }
        Action::Update
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeSelected(Case),
    Reset,
    Toggle(bool),
    Update(String),
}
