use std::{cell::Cell, path::Path, sync::Arc};

use iced::{
    self, Element, Subscription, Task, keyboard,
    widget::{Row, column, row, text},
};

mod add_view;
mod files;
use renameit_lib::{DirectoryError, RenameOption};

pub fn run<P: AsRef<Path>>(initial_dir: Option<P>) -> Result<(), GuiError> {
    let app = Cell::new(App::new(initial_dir));
    iced::application(move || app.take(), App::update, App::view)
        .title("Renameit!")
        .subscription(App::subscription)
        .run()?;
    Ok(())
}

#[derive(Default)]
struct App {
    files: files::Files,
    add: add_view::AddView,
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
            Message::Add(message) => match self.add.update(message) {
                Action::Update => {
                    return self.apply_option(self.add.clone());
                }
                Action::Remove => {
                    return self.remove_option(self.add.to_options(0));
                }
            },
            Message::Files(message) => match self.files.update(message) {
                files::Action::None => {}
                files::Action::Run(task) => return task.map(Message::Files),
                files::Action::Reapply => return self.reapply(),
            },
            Message::None => {}
        }
        Task::none()
    }

    fn apply_option(&mut self, option: impl OptionBox + Send + Sync + 'static) -> Task<Message> {
        let opt = Arc::new(move |idx| option.to_options(idx));
        match self.files.update(files::Message::SetOption(opt)) {
            files::Action::None | files::Action::Reapply => Task::none(),
            files::Action::Run(task) => task.map(Message::Files),
        }
    }

    fn remove_option(&mut self, option: RenameOption) -> Task<Message> {
        match self.files.update(files::Message::ClearOption(option)) {
            files::Action::None | files::Action::Reapply => Task::none(),
            files::Action::Run(task) => task.map(Message::Files),
        }
    }

    fn reapply(&mut self) -> Task<Message> {
        Task::batch([self.apply_option(self.add.clone())])
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            self.files.view().map(Message::Files),
            row![self.add.view().map(Message::Add)]
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

fn input_field<'a, M: 'a>(label: &'a str, widget: Element<'a, M>) -> Row<'a, M> {
    row![text(label), widget]
}

#[derive(Clone)]
enum Message {
    Add(add_view::Message),
    Files(files::Message),
    None,
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
