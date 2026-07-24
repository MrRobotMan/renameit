use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use iced::{
    Element, Renderer, Task, Theme,
    widget::{self, Id, column, row, scrollable, text, text_input},
};
use iced_table2::table::{Column, table};
use renameit_lib::{Renamer, helpers::get_directory};

const HEADERS: [&str; 6] = [
    "Name",
    "Extension",
    "New Name",
    "Size",
    "Date Modified",
    "Date Created",
];

pub struct Files {
    files: Vec<Renamer>, // file and if it's slated to be changed
    rows: Vec<FileData>,
    selected: Vec<bool>,
    columns: [DisplayColumn; 6],
    header_id: Id,
    body_id: Id,
    sort_col: Option<usize>,
    sort_ascending: bool,
    path: Option<PathBuf>,
    path_text: String,
}

impl Default for Files {
    fn default() -> Self {
        // (stem, renamed, extension, size, date modified, date created)
        Self {
            files: Vec::new(),
            rows: Vec::new(),
            selected: Vec::new(),
            columns: [
                DisplayColumn {
                    width: 180.0,
                    ..Default::default()
                },
                DisplayColumn {
                    width: 80.0,
                    ..Default::default()
                },
                DisplayColumn {
                    width: 180.0,
                    ..Default::default()
                },
                DisplayColumn {
                    width: 90.0,
                    ..Default::default()
                },
                DisplayColumn {
                    width: 150.0,
                    ..Default::default()
                },
                DisplayColumn {
                    width: 150.0,
                    ..Default::default()
                },
            ],
            header_id: Id::unique(),
            body_id: Id::unique(),
            sort_col: None,
            sort_ascending: true,
            path: None,
            path_text: String::new(),
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
            self.populate();
        }
        self.path.clone()
    }

    fn populate(&mut self) {
        self.files.clear();
        self.rows.clear();
        for v in self.selected.iter_mut() {
            *v = false;
        }
        let Some(path) = &self.path else { return };
        // Only try to get files from the path if the path is valid and accessable.
        if let Ok(dir) = read_dir(path) {
            for pa in dir {
                if let Ok(p) = pa
                    && let Ok(file) = p.path().try_into()
                {
                    self.rows.push(FileData::from(&file));
                    self.files.push(file);
                    self.selected.push(false);
                }
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
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
            Message::SyncHeader(offset) => {
                return Action::Run(widget::operation::scroll_to(self.header_id.clone(), offset));
            }
            Message::ColumnDragged(index, offset) => {
                if let Some(col) = self.columns.get_mut(index) {
                    col.resize_offset = Some(offset);
                }
            }
            Message::ColumnReleased => {
                for col in &mut self.columns {
                    if let Some(offset) = col.resize_offset.take() {
                        col.width += offset;
                    }
                }
            }
            Message::HeaderPressed(index) => {
                if index != 2 {
                    if self.sort_col == Some(index) {
                        self.sort_ascending = !self.sort_ascending;
                    } else {
                        self.sort_col = Some(index);
                        self.sort_ascending = true;
                    }
                    for (idx, col) in self.columns.iter_mut().enumerate() {
                        col.sort = if idx == index {
                            Some(self.sort_ascending)
                        } else {
                            None
                        };
                    }
                    let mut zipped = self
                        .rows
                        .drain(..)
                        .zip(self.files.drain(..))
                        .collect::<Vec<_>>();
                    zipped.sort_by(|(a, _), (b, _)| {
                        let ord = a[index].cmp(&b[index]);
                        if self.sort_ascending {
                            ord
                        } else {
                            ord.reverse()
                        }
                    });
                    let mut unzipped = (self.rows.clone(), self.files.clone());
                    unzipped.extend(zipped);
                    (self.rows, self.files) = unzipped;
                    for v in self.selected.iter_mut() {
                        *v = false;
                    }
                }
            }
        }
        Action::None
    }

    pub fn view(&self) -> Element<'_, Message> {
        let tbl: Element<'_, Message> = table(
            self.header_id.clone(),
            self.body_id.clone(),
            &self.columns,
            &self.rows,
            Message::SyncHeader,
        )
        .on_column_resize(Message::ColumnDragged, Message::ColumnReleased)
        .on_header_press(Message::HeaderPressed)
        .cell_padding(6)
        .min_width(400.0)
        .into();

        column![
            text_input(&self.path_text, &self.path_text)
                .on_submit(Message::Submitted)
                .on_input(Message::NewDir),
            tbl
        ]
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

#[derive(Default, Copy, Clone)]
struct DisplayColumn {
    width: f32,
    resize_offset: Option<f32>,
    sort: Option<bool>,
}

impl<'a> Column<'a, Message, Theme, Renderer> for DisplayColumn {
    type Row = FileData;

    fn header(&'a self, col_index: usize) -> Element<'a, Message, Theme, Renderer> {
        let label = text(HEADERS[col_index]);
        match self.sort {
            Some(true) => row![label, text(" ▴")].into(),
            Some(false) => row![label, text(" ▾")].into(),
            None => label.into(),
        }
    }

    fn cell(
        &'a self,
        col_index: usize,
        _row_index: usize,
        row: &'a Self::Row,
    ) -> Element<'a, Message, Theme, Renderer> {
        text(&row[col_index]).into()
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn resize_offset(&self) -> Option<f32> {
        self.resize_offset
    }
}

pub enum Action {
    None,
    Run(Task<Message>),
}

#[derive(Clone)]
pub enum Message {
    NewDir(String),
    Submitted,
    SyncHeader(scrollable::AbsoluteOffset),
    ColumnDragged(usize, f32),
    ColumnReleased,
    HeaderPressed(usize),
}

#[derive(Clone)]
struct FileData {
    original: String,
    renamed: String,
    extension: String,
    size: String,
    modified: String,
    created: String,
}

impl std::ops::Index<usize> for FileData {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.original,
            1 => &self.extension,
            2 => &self.renamed,
            3 => &self.size,
            4 => &self.modified,
            _ => &self.created,
        }
    }
}

impl From<&Renamer> for FileData {
    fn from(file: &Renamer) -> Self {
        let data = file.info();
        Self {
            original: data.0.to_string(),
            extension: data.2.map_or_else(String::new, str::to_string),
            renamed: if let Some(e) = data.2 {
                format!("{}.{e}", data.1)
            } else {
                data.1.to_string()
            },
            size: data.3.map_or_else(String::new, |s| format!("{s} B")),
            modified: data
                .4
                .map_or_else(String::new, |dt| dt.format("%Y-%m-%d %H:%M").to_string()),
            created: data
                .5
                .map_or_else(String::new, |dt| dt.format("%Y-%m-%d %H:%M").to_string()),
        }
    }
}
