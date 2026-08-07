use iced::{
    Element, Font,
    widget::{button, checkbox, column, combo_box, row, space, text_input},
};
use iced_aw::widget::LabeledFrame;
use renameit_lib::{
    DateOptions, RenameOption,
    date::{DateFormat, DateMode, DateType},
};

use crate::{Action, OptionBox, input_field};

#[derive(Debug, Clone)]
pub struct DateView {
    date_modes: combo_box::State<DateMode>,
    date_mode: Option<DateMode>,
    date_types: combo_box::State<DateType>,
    date_type: Option<DateType>,
    date_formats: combo_box::State<DateFormat>,
    date_format: Option<DateFormat>,
    custom: String,
    full_year: bool,
    sep: String,
    seg: String,
}

impl Default for DateView {
    fn default() -> Self {
        Self {
            date_modes: combo_box::State::new(vec![
                DateMode::Prefix,
                DateMode::Suffix,
                DateMode::None,
            ]),
            date_mode: Some(DateMode::None),
            date_types: combo_box::State::new(vec![
                DateType::Created,
                DateType::Modified,
                DateType::Current,
            ]),
            date_type: Some(DateType::Created),
            date_formats: combo_box::State::new(DateFormat::iter().collect()),
            date_format: Some(DateFormat::default()),
            custom: String::new(),
            full_year: Default::default(),
            sep: Default::default(),
            seg: Default::default(),
        }
    }
}

impl OptionBox for DateView {
    fn to_options(&self) -> Box<dyn Fn(usize) -> RenameOption + Send + Sync> {
        let date_mode = self.date_mode.clone().unwrap_or_default();
        let date_type = self.date_type.clone().unwrap_or_default();
        let fmt = self.date_format.clone().unwrap_or_default();
        let sep = self.sep.clone();
        let seg = self.seg.clone();
        let full_year = self.full_year;
        Box::new(move |_| {
            RenameOption::Date(DateOptions {
                date_mode: date_mode.clone(),
                date_type: date_type.clone(),
                fmt: fmt.clone(),
                sep: sep.clone(),
                seg: seg.clone(),
                full_year,
            })
        })
    }
}

impl DateView {
    pub fn view(&self) -> Element<'_, Message> {
        LabeledFrame::new(
            "Date",
            column![
                input_field(
                    "Mode",
                    combo_box(
                        &self.date_modes,
                        "",
                        self.date_mode.as_ref(),
                        Message::ChangeMode,
                    )
                    .icon(text_input::Icon {
                        font: Font::default(),
                        code_point: '▾',
                        size: None,
                        spacing: 5.0,
                        side: text_input::Side::Right,
                    })
                    .into(),
                ),
                input_field(
                    "Type",
                    combo_box(
                        &self.date_types,
                        "",
                        self.date_type.as_ref(),
                        Message::ChangeType,
                    )
                    .icon(text_input::Icon {
                        font: Font::default(),
                        code_point: '▾',
                        size: None,
                        spacing: 5.0,
                        side: text_input::Side::Right,
                    })
                    .into(),
                ),
                input_field(
                    "Format",
                    combo_box(
                        &self.date_formats,
                        "",
                        self.date_format.as_ref(),
                        Message::ChangeFormat,
                    )
                    .icon(text_input::Icon {
                        font: Font::default(),
                        code_point: '▾',
                        size: None,
                        spacing: 5.0,
                        side: text_input::Side::Right,
                    })
                    .into(),
                ),
                {
                    let wid: Element<'_, Message> =
                        if matches!(self.date_format, Some(DateFormat::Custom(_))) {
                            text_input("", &self.custom)
                                .on_input(|item| Message::Update(Field::Cust, item))
                                .into()
                        } else {
                            space::horizontal().height(32).into()
                        };
                    wid
                },
                row![
                    input_field(
                        "Sep",
                        text_input("", &self.sep)
                            .on_input(|item| Message::Update(Field::Sep, item))
                            .into()
                    ),
                    input_field(
                        "Seg",
                        text_input("", &self.seg)
                            .on_input(|item| Message::Update(Field::Seg, item))
                            .into()
                    ),
                ],
                row![
                    input_field(
                        "YYYY",
                        checkbox(self.full_year).on_toggle(Message::Toggle).into()
                    ),
                    button("Clear").on_press(Message::Reset)
                ],
            ]
            .width(200),
        )
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Reset => {
                self.date_mode = Some(DateMode::default());
                self.date_type = Some(DateType::default());
                self.date_format = Some(DateFormat::default());
                self.sep = String::new();
                return Action::Remove;
            }
            Message::Toggle(b) => self.full_year = b,
            Message::ChangeMode(mode) => self.date_mode = Some(mode),
            Message::ChangeType(typ) => self.date_type = Some(typ),
            Message::ChangeFormat(fmt) => self.date_format = Some(fmt),
            Message::Update(Field::Cust, text) => self.custom = text,
            Message::Update(Field::Seg, text) => self.seg = text,
            Message::Update(Field::Sep, text) => self.sep = text,
        }
        Action::Update
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Field {
    Cust,
    Sep,
    Seg,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeMode(DateMode),
    ChangeType(DateType),
    ChangeFormat(DateFormat),
    Reset,
    Toggle(bool),
    Update(Field, String),
}
