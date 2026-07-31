use super::{Action, OptionBox};
use iced::{
    Element, Font,
    widget::{button, column, combo_box, text_input},
};
use iced_aw::widget::LabeledFrame;
use renameit_lib::{NameOptions, RenameOption};

#[derive(Debug)]
pub struct NameView {
    choices: combo_box::State<Choice>,
    version: Option<Choice>,
    text: String,
}

impl Default for NameView {
    fn default() -> Self {
        Self {
            choices: combo_box::State::new(vec![
                Choice::Keep,
                Choice::Remove,
                Choice::Reverse,
                Choice::Fixed,
            ]),
            version: Some(Choice::Keep),
            text: Default::default(),
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub enum Choice {
    #[default]
    Keep,
    Remove,
    Reverse,
    Fixed,
}

#[derive(Clone)]
pub enum Message {
    ChangeVersion(Choice),
    Update(String),
    Reset,
}

impl OptionBox for NameOptions {
    fn to_options(&self, _index: usize) -> RenameOption {
        RenameOption::Name(self.clone())
    }
}

impl NameView {
    pub fn to_option_box(&self) -> NameOptions {
        match self.version {
            Some(Choice::Keep) | None => NameOptions::Keep,
            Some(Choice::Remove) => NameOptions::Remove,
            Some(Choice::Reverse) => NameOptions::Reverse,
            Some(Choice::Fixed) => NameOptions::Fixed(self.text.clone()),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        LabeledFrame::new(
            "Name",
            column![
                combo_box(
                    &self.choices,
                    "",
                    self.version.as_ref(),
                    Message::ChangeVersion
                )
                .icon(text_input::Icon {
                    font: Font::default(),
                    code_point: '▾',
                    size: None,
                    spacing: 5.0,
                    side: text_input::Side::Right,
                }),
                text_input("", &self.text).on_input(Message::Update),
                button("Reset").on_press(Message::Reset)
            ],
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangeVersion(choice) => self.version = Some(choice),
            Message::Update(text) => self.text = text,
            Message::Reset => {
                self.version = Some(Choice::Keep);
                self.text = String::new();
                return Action::Remove;
            }
        }
        Action::Update
    }
}

impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Choice::Keep => "Keep",
            Choice::Remove => "Remove",
            Choice::Reverse => "Reverse",
            Choice::Fixed => "Fixed",
        })
    }
}
