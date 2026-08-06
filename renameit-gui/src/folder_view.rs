use iced::{
    Element, Font,
    widget::{button, column, combo_box, text_input},
};
use iced_aw::widget::{LabeledFrame, NumberInput};
use renameit_lib::{FolderOptions, RenameOption, folder::FolderMode};

use crate::{Action, OptionBox, input_field};

#[derive(Debug, Clone)]
pub struct FolderView {
    options: combo_box::State<FolderMode>,
    selected: Option<FolderMode>,
    sep: String,
    levels: i32,
}

impl Default for FolderView {
    fn default() -> Self {
        Self {
            options: combo_box::State::new(vec![
                FolderMode::None,
                FolderMode::Prefix,
                FolderMode::Suffix,
            ]),
            selected: Some(FolderMode::default()),
            sep: Default::default(),
            levels: 1,
        }
    }
}

impl OptionBox for FolderOptions {
    fn to_options(&self, _index: usize) -> RenameOption {
        RenameOption::Folder(self.clone())
    }
}

impl FolderView {
    pub fn to_option_box(&self) -> FolderOptions {
        FolderOptions {
            mode: self.selected.unwrap_or_default(),
            sep: self.sep.clone(),
            levels: self.levels,
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        LabeledFrame::new(
            "Parent Folder Name",
            column![
                input_field(
                    "Mode",
                    combo_box(
                        &self.options,
                        "",
                        self.selected.as_ref(),
                        Message::UpdateSelected
                    )
                    .icon(text_input::Icon {
                        font: Font::default(),
                        code_point: '▾',
                        size: None,
                        spacing: 5.0,
                        side: text_input::Side::Right,
                    })
                    .into()
                ),
                input_field(
                    "Sep.",
                    text_input("", &self.sep)
                        .on_input(Message::UpdateText)
                        .into()
                ),
                input_field(
                    "Levels",
                    NumberInput::new(&self.levels, i32::MIN..=i32::MAX, Message::UpdateNumber)
                        .into()
                ),
                button("Clear").on_press(Message::Reset)
            ],
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        if !matches!(message, Message::Reset | Message::UpdateSelected(_))
            && matches!(self.selected, Some(FolderMode::None))
        {
            self.selected = Some(FolderMode::Prefix);
        }
        match message {
            Message::Reset => {
                self.selected = Some(FolderMode::default());
                self.sep = String::new();
                self.levels = 1;
                return Action::Remove;
            }
            Message::UpdateNumber(n) => self.levels = n,
            Message::UpdateSelected(m) => self.selected = Some(m),
            Message::UpdateText(t) => self.sep = t,
        }
        Action::Update
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    UpdateNumber(i32),
    UpdateSelected(FolderMode),
    UpdateText(String),
}
