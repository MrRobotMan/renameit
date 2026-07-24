use std::{cell::Cell, path::Path};

use iced::{self, Element, Task};

mod files;
use files::Files;
use renameit_lib::DirectoryError;

pub fn run<P: AsRef<Path>>(initial_dir: Option<P>) -> Result<(), GuiError> {
    let app = Cell::new(App::new(initial_dir));
    iced::application(move || app.take(), App::update, App::view)
        .title("Renameit!")
        .run()?;
    Ok(())
}

#[derive(Default)]
struct App {
    files: files::Files,
}

impl App {
    fn new<P: AsRef<Path>>(initial_dir: Option<P>) -> Self {
        let files = Files::new(initial_dir.as_ref());
        Self { files }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Files(message) => match self.files.update(message) {
                files::Action::None => {}
                files::Action::Run(task) => return task.map(Message::Files),
            },
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        self.files.view().map(Message::Files)
    }
}
#[derive(Clone)]
enum Message {
    Files(files::Message),
}

#[derive(Debug, thiserror::Error)]
pub enum GuiError {
    #[error(transparent)]
    Iced(#[from] iced::Error),
    #[error(transparent)]
    Directory(#[from] DirectoryError),
}
