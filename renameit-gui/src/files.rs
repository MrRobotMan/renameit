use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use iced::{
    Element,
    widget::{column, text, text_input},
};
use renameit_lib::{Renamer, helpers::get_directory};

#[derive(Debug, PartialEq, Eq, Clone)]
enum Status {
    Selected(Renamer),
    NotSelected(Renamer),
}

#[derive(Default)]
pub struct Files {
    files: Vec<Status>, // file and if it's slated to be changed
    path: Option<PathBuf>,
    path_text: String,
}

#[derive(Clone)]
pub enum Message {
    NewDir(String),
    Submitted,
}

impl std::ops::Not for Status {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Status::Selected(renamer) => Status::NotSelected(renamer),
            Status::NotSelected(renamer) => Status::Selected(renamer),
        }
    }
}

impl Files {
    pub fn new<P: AsRef<Path>>(path: Option<P>) -> Self {
        let mut files = Self::default();
        let path = if let Some(p) = path {
            get_directory(p).map_or_else(|_| home::home_dir(), Some)
        } else {
            home::home_dir()
        };
        files.path = path;
        files.path_text = files
            .path
            .clone()
            .map_or_else(String::new, |p| p.display().to_string());
        files.populate();
        files
    }

    fn new_dir<S: AsRef<str>>(&mut self, path: S) -> Option<PathBuf> {
        if let Ok(p) = get_directory(path.as_ref()) {
            self.path = Some(p);
        }
        self.path.clone()
    }

    fn populate(&mut self) {
        self.files.clear();
        let Some(path) = &self.path else { return };
        // Only try to get files from the path if the path is valid and accessable.
        if let Ok(dir) = read_dir(path) {
            for pa in dir {
                if let Ok(p) = pa
                    && let Ok(file) = p.path().try_into()
                {
                    self.files.push(Status::NotSelected(file));
                }
            }
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::NewDir(text) => self.path_text = text,
            Message::Submitted => {
                let p = &self.path_text.clone();
                if PathBuf::from(p).is_dir() {
                    self.new_dir(p);
                    if self.path.is_some() {
                        self.populate();
                    };
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            text_input(&self.path_text, &self.path_text)
                .on_submit(Message::Submitted)
                .on_input(Message::NewDir)
        ]
        .extend(self.files.iter().map(|s| {
            let f = match s {
                Status::Selected(f) => f.info(),
                Status::NotSelected(f) => f.info(),
            };
            text(if let Some(e) = f.2 {
                format!("{}.{}", f.0, e)
            } else {
                f.0.to_string()
            })
            .into()
        }))
        .into()
    }

    // pub fn process(&mut self) {
    //     for (file, _) in self
    //         .files
    //         .iter_mut()
    //         .filter(|(_, s)| *s == Status::Selected)
    //     {
    //         if file.rename().is_err() {
    //             file.revert().expect("Unknown error.")
    //         }
    //     }
    // }

    // pub fn preview(&mut self) {
    //     for (file, _) in self
    //         .files
    //         .iter_mut()
    //         .filter(|(_, s)| *s == Status::Selected)
    //     {
    //         file.preview();
    //     }
    // }
}
