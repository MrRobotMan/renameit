use iced::{
    self, Element,
    widget::{Column, button, row, text},
};

mod directory;
pub use directory::{Columns, Directory, get_initial_directory};
use renameit_lib::{Renamer, RenamerError};

pub fn run(initial_dir: Option<String>) -> Result<(), GuiError> {
    let mut files = Selected::default();
    if let Ok(initial) = get_initial_directory(initial_dir) {
        files.add(initial)?;
    };
    iced::run(update, view)?;
    Ok(())
}

fn update(states: &mut States, msg: Message) {
    match msg {
        Message::Inc(idx) => {
            let state = &mut states.states[idx];
            println!("{:?}", state);
            state.age += 1;
        }
        Message::Add => {
            states.add();
        }
    }
}

fn view(states: &States) -> Element<'_, Message> {
    let mut col = Column::with_capacity(states.states.len() + 1);
    col = col.extend(states.states.iter().map(|r| {
        button(row![text(r.id), text(r.age)].spacing(20))
            .on_press(Message::Inc(r.id))
            .into()
    }));
    col.push(button("New").on_press(Message::Add)).into()
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Inc(usize),
    Add,
}

#[derive(Debug, Default)]
struct States {
    states: Vec<State>,
}

impl States {
    fn add(&mut self) {
        self.states.push(State {
            id: self.states.len(),
            age: 0,
        })
    }
}

#[derive(Debug, Default)]
struct State {
    id: usize,
    age: i64,
}

use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Selected {
    selected: Vec<Renamer>,
}

impl Selected {
    fn clear(&mut self) {
        self.selected.clear()
    }

    fn add(&mut self, file: PathBuf) -> Result<(), RenamerError> {
        self.selected.push(Renamer::try_from(file.as_path())?);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GuiError {
    #[error(transparent)]
    Iced(#[from] iced::Error),
    #[error(transparent)]
    Renamer(#[from] RenamerError),
}
