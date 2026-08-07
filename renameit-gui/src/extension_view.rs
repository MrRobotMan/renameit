use iced::{
    Element, Font,
    widget::{Space, button, column, combo_box, text_input},
};
use iced_aw::widget::LabeledFrame;
use renameit_lib::{ExtensionOptions, RenameOption};

use crate::{Action, OptionBox};

#[derive(Debug, Clone)]
pub struct ExtensionView {
    options: combo_box::State<Opt>,
    selected: Option<Opt>,
    text: String,
}

impl Default for ExtensionView {
    fn default() -> Self {
        Self {
            options: combo_box::State::new(vec![
                Opt::Keep,
                Opt::Lower,
                Opt::Upper,
                Opt::Title,
                Opt::New,
                Opt::Extra,
                Opt::Remove,
            ]),
            selected: Some(Opt::Keep),
            text: String::new(),
        }
    }
}

impl OptionBox for ExtensionView {
    fn to_options(&self) -> Box<dyn Fn(usize) -> RenameOption + Send + Sync> {
        let opt = match self.selected {
            Some(Opt::Keep) | None => ExtensionOptions::Keep,
            Some(Opt::Lower) => ExtensionOptions::Lower,
            Some(Opt::Upper) => ExtensionOptions::Upper,
            Some(Opt::Title) => ExtensionOptions::Title,
            Some(Opt::New) => ExtensionOptions::New(self.text.clone()),
            Some(Opt::Extra) => ExtensionOptions::Extra(self.text.clone()),
            Some(Opt::Remove) => ExtensionOptions::Remove,
        };
        Box::new(move |_| RenameOption::Extension(opt.clone()))
    }
}

impl ExtensionView {
    pub fn view(&self) -> Element<'_, Message> {
        LabeledFrame::new(
            "Extension",
            column![
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
                }),
                {
                    let w: Element<'_, Message> = match self.selected {
                        Some(opt) if [Opt::New, Opt::Extra].contains(&opt) => {
                            text_input("", &self.text)
                                .on_input(Message::UpdateText)
                                .into()
                        }
                        _ => Space::new().height(30).into(),
                    };
                    w
                },
                button("Clear").on_press(Message::Reset)
            ],
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Reset => {
                self.selected = Some(Opt::Keep);
                self.text = String::new();
                return Action::Remove;
            }
            Message::UpdateText(text) => self.text = text,
            Message::UpdateSelected(opt) => self.selected = Some(opt),
        }
        Action::Update
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Opt {
    Keep,
    Lower,
    Upper,
    Title,
    New,
    Extra,
    Remove,
}

impl std::fmt::Display for Opt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Opt::Keep => "Keep",
            Opt::Lower => "Lower",
            Opt::Upper => "Upper",
            Opt::Title => "Title",
            Opt::New => "New",
            Opt::Extra => "Extra",
            Opt::Remove => "Remove",
        })
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    UpdateText(String),
    UpdateSelected(Opt),
}
