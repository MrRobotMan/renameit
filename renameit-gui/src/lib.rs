use std::{
    cell::Cell,
    path::{Path, PathBuf},
    sync::Arc,
};

use iced::{
    self, Element, Subscription, Task, keyboard,
    widget::{Row, button, column, row, text},
};

mod add_view;
mod case_view;
mod date_view;
mod extension_view;
mod files;
mod folder_view;
mod name_view;
mod number_view;
mod regex_view;
mod remove_view;
mod replace_view;
use renameit_lib::{DirectoryError, FileError, RenameOption};

pub fn run<P: AsRef<Path>>(initial_dir: Option<P>) -> Result<(), GuiError> {
    let app = Cell::new(App::new(initial_dir));
    iced::application(move || app.take(), App::update, App::view)
        .title("Renameit!")
        .subscription(App::subscription)
        .font(iced_aw::ICED_AW_FONT_BYTES)
        .run()?;
    Ok(())
}

#[derive(Default)]
struct App {
    errors: Vec<(PathBuf, FileError)>,
    files: files::Files,
    add: add_view::AddView,
    case: case_view::CaseView,
    date: date_view::DateView,
    extension: extension_view::ExtensionView,
    folder: folder_view::FolderView,
    name: name_view::NameView,
    number: number_view::NumberView,
    regex: regex_view::RegexView,
    remove: remove_view::RemoveView,
    replace: replace_view::ReplaceView,
}

macro_rules! match_option {
    ($self:ident, $field:ident, $variant:ident, $msg:ident) => {{
        match $self.$field.update($msg) {
            Action::Update => {
                return $self.apply_option($self.$field.clone());
            }
            Action::Remove => {
                return $self.remove_option($self.$field.to_options(0));
            }
        }
    }};
    ($self:ident, $field:ident, $variant:ident, $msg:ident, $func:ident) => {{
        match $self.$field.update($msg) {
            Action::Update => {
                return $self.apply_option($self.$field.$func());
            }
            Action::Remove => {
                return $self.remove_option($self.$field.$func().to_options(0));
            }
        }
    }};
}

impl App {
    fn new<P: AsRef<Path>>(initial_dir: Option<P>) -> Self {
        let files = files::Files::new(initial_dir.as_ref());
        Self {
            files,
            ..Default::default()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Files(message) => match self.files.update(message) {
                files::Action::None | files::Action::Errors(_) => {}
                files::Action::Run(task) => return task.map(Message::Files),
                files::Action::Reapply => return self.reapply(),
            },
            Message::Confirmed => {
                if let files::Action::Errors(items) = self.files.update(files::Message::Process) {
                    self.errors = items
                }
            }
            Message::Rename => {
                let dialog = rfd::AsyncMessageDialog::new()
                    .set_title("Confirm Renaming")
                    .set_description("Are you sure you want to rename the selection? This action can not be undone.")
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .set_level(rfd::MessageLevel::Warning);
                return Task::perform(dialog.show(), |answer| {
                    if answer == rfd::MessageDialogResult::Yes {
                        Message::Confirmed
                    } else {
                        Message::None
                    }
                });
            }
            Message::None => {}
            Message::Add(message) => match_option!(self, add, Add, message),
            Message::Case(message) => match_option!(self, case, Case, message, to_option_box),
            Message::Date(message) => match_option!(self, date, Date, message, to_option_box),
            Message::Name(message) => match_option!(self, name, Name, message, to_option_box),
            Message::Regex(message) => match_option!(self, regex, Regex, message),
            Message::Replace(message) => match_option!(self, replace, Replace, message),
            Message::Extension(message) => {
                match_option!(self, extension, Extension, message, to_option_box)
            }
            Message::Folder(message) => match_option!(self, folder, Folder, message, to_option_box),
            Message::Number(message) => match_option!(self, number, Number, message, to_option_box),
            Message::Remove(message) => match_option!(self, remove, Remove, message),
        }
        Task::none()
    }

    fn apply_option(&mut self, option: impl OptionBox + Send + Sync + 'static) -> Task<Message> {
        let opt = Arc::new(move |idx| option.to_options(idx));
        if let files::Action::Run(task) = self.files.update(files::Message::SetOption(opt)) {
            task.map(Message::Files)
        } else {
            Task::none()
        }
    }

    fn remove_option(&mut self, option: RenameOption) -> Task<Message> {
        if let files::Action::Run(task) = self.files.update(files::Message::ClearOption(option)) {
            task.map(Message::Files)
        } else {
            Task::none()
        }
    }

    fn reapply(&mut self) -> Task<Message> {
        Task::batch([self.apply_option(self.add.clone())])
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            self.files.view().map(Message::Files),
            //    -  1 RegEx
            //    -  2 Name
            //    -  3 Replace
            //    -  4 Case
            //    -  5 Remove
            //    -  6 Add
            //    -  7 Auto Date
            //    -  8 Append Folder Name
            //    -  9 Numbering
            //    - 10 Extension
            row![
                column![
                    self.regex.view().map(Message::Regex),
                    self.name.view().map(Message::Name),
                ],
                column![
                    self.replace.view().map(Message::Replace),
                    self.case.view().map(Message::Case),
                ],
                self.remove.view().map(Message::Remove),
                self.add.view().map(Message::Add),
                self.date.view().map(Message::Date),
                self.folder.view().map(Message::Folder),
                self.number.view().map(Message::Number),
                self.extension.view().map(Message::Extension),
                button("Rename").on_press(Message::Rename)
            ]
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().map(|event| match event {
            keyboard::Event::ModifiersChanged(m) => Message::Files(files::Message::Modifier(m)),
            _ => Message::None,
        })
    }
}

trait OptionBox {
    fn to_options(&self, index: usize) -> RenameOption;
}

fn input_field<'a, M: 'a, S: Into<String> + 'a>(label: S, widget: Element<'a, M>) -> Row<'a, M> {
    row![text(label.into()), widget]
}

#[derive(Clone)]
enum Message {
    Add(add_view::Message),
    Case(case_view::Message),
    Confirmed,
    Date(date_view::Message),
    Extension(extension_view::Message),
    Files(files::Message),
    Folder(folder_view::Message),
    Name(name_view::Message),
    None,
    Number(number_view::Message),
    Regex(regex_view::Message),
    Remove(remove_view::Message),
    Rename,
    Replace(replace_view::Message),
}

enum Action {
    Remove,
    Update,
}

#[derive(Debug, thiserror::Error)]
pub enum GuiError {
    #[error(transparent)]
    Iced(#[from] iced::Error),
    #[error(transparent)]
    Directory(#[from] DirectoryError),
}
