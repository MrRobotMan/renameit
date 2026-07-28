use std::{
    collections::HashSet,
    fs::read_dir,
    mem,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{DateTime, Local};
use iced::{
    Color, Element, Length, Renderer, Task, Theme,
    keyboard::Modifiers,
    widget::{self, Id, button, column, container, row, scrollable, text, text_input},
};
use iced_table2::table::{Column, table};
use renameit_lib::{
    FileError, RenameOption, Renamer,
    helpers::{get_directory, get_start_dir},
};

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
    selected: HashSet<usize>,
    columns: [DisplayColumn; 6],
    header_id: Id,
    body_id: Id,
    sort_col: Option<usize>,
    sort_ascending: bool,
    path: Option<PathBuf>,
    path_text: String,
    modifiers: Modifiers,
    last_row_selected: Option<usize>,
}

impl Default for Files {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            rows: Vec::new(),
            selected: HashSet::new(),
            columns: [
                DisplayColumn::new(180.0),
                DisplayColumn::new(80.0),
                DisplayColumn::new(180.0),
                DisplayColumn::new(90.0),
                DisplayColumn::new(180.0),
                DisplayColumn::new(180.0),
            ],
            header_id: Id::unique(),
            body_id: Id::unique(),
            sort_col: None,
            sort_ascending: true,
            path: None,
            path_text: String::new(),
            modifiers: Modifiers::empty(),
            last_row_selected: None,
        }
    }
}
impl Files {
    pub fn new<P: AsRef<Path>>(path: Option<P>) -> Self {
        let mut files = Self::default();
        let path = if let Some(p) = path {
            get_directory(p).map_or_else(|_| get_start_dir().ok(), Some)
        } else {
            get_start_dir().ok()
        };
        files.path = path;
        files.path_text = files
            .path
            .clone()
            .map_or_else(String::new, |p| p.display().to_string());
        files.populate();
        files
    }

    fn new_dir<S: AsRef<Path>>(&mut self, path: S) -> Option<PathBuf> {
        self.sort_col = None;
        self.sort_ascending = true;
        for col in self.columns.iter_mut() {
            col.sort = None;
        }
        if let Ok(p) = get_directory(path.as_ref()) {
            self.path = Some(p);
            self.populate();
        }
        self.path.clone()
    }

    fn populate(&mut self) {
        self.files.clear();
        self.rows.clear();
        self.selected.clear();
        self.last_row_selected = None;
        let Some(path) = &self.path else { return };
        // Only try to get files from the path if the path is valid and accessable.
        if let Ok(dir) = read_dir(path) {
            for pa in dir {
                if let Ok(p) = pa
                    && let Ok(file) = p.path().try_into()
                {
                    self.files.push(file);
                }
            }
        }
        self.files.sort();
        self.rows = self.files.iter_mut().map(FileData::from).collect();
    }

    pub fn update(&mut self, message: Message) -> Action {
        let updating = matches!(&message, &Message::NewDir(_));
        match message {
            Message::ClearOption(option) => {
                for (idx, (file, row)) in
                    self.files.iter_mut().zip(self.rows.iter_mut()).enumerate()
                {
                    file.remove_option(option.clone());
                    file.revert();
                    if self.selected.contains(&idx) {
                        row.renamed = file.preview().display().to_string();
                    }
                }
            }
            Message::ColumnDragged(index, offset) => {
                if let Some(col) = self.columns.get_mut(index) {
                    col.resize_offset = Some(offset);
                }
            }
            Message::ColumnReleased => {
                for col in &mut self.columns {
                    if let Some(offset) = col.resize_offset.take() {
                        col.width = (col.width + offset).max(col.min_size);
                    }
                }
            }
            Message::DirSelected(None) => {}
            Message::DirSelected(Some(p)) => {
                if self.new_dir(&p).is_some() {
                    self.path_text = p.display().to_string();
                }
            }
            Message::DirSubmitted => {
                self.new_dir(self.path_text.clone());
            }
            Message::FolderDialog => {
                let mut dialog = rfd::AsyncFileDialog::new();
                if let Some(p) = &self.path {
                    dialog = dialog.set_directory(p)
                }
                return Action::Run(Task::perform(dialog.pick_folder(), |res| {
                    Message::DirSelected(res.map(|dir| dir.path().to_path_buf()))
                }));
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
                    zipped.sort_by(|(a, f1), (b, f2)| {
                        let ord = match index {
                            0 => a.original.cmp(&b.original), // Sort by original name
                            1 => match (f1.is_dir(), f2.is_dir()) {
                                // Sort by extension with directories on top.
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.extension.cmp(&b.extension),
                            },
                            2 => std::cmp::Ordering::Equal, // Sort by new name. No-op.
                            3 => a.size.cmp(&b.size),       // Sort by size
                            4 => a.modified.cmp(&b.modified), // Sort by date modified.
                            5 => a.created.cmp(&b.created), // Sort by date created.
                            _ => unreachable!("Bad index"),
                        };
                        if self.sort_ascending {
                            ord
                        } else {
                            ord.reverse()
                        }
                    });
                    (self.rows, self.files) = zipped.into_iter().unzip();
                    self.selected.clear();
                    self.last_row_selected = None;
                    self.selected_changed();
                }
            }
            Message::Modifier(m) => self.modifiers = m,
            Message::NewDir(text) => self.path_text = text,
            Message::RowPressed(index) => {
                let ctrl = self.modifiers.command();
                let shift = self.modifiers.shift();
                match (ctrl, shift) {
                    (true, false) => {
                        // Control click. Add row.
                        if !self.selected.remove(&index) {
                            self.selected.insert(index);
                        }
                        self.last_row_selected = Some(index);
                    }
                    (false, true) => {
                        // Shift click. Select range.
                        if let Some(base) = self.last_row_selected {
                            let (low, high) = (base.min(index), base.max(index));
                            self.selected = (low..=high).collect();
                        } else {
                            self.selected = HashSet::from([index]);
                            self.last_row_selected = Some(index);
                        }
                    }
                    (true, true) => {
                        // Ctrl+Shift click. Extend selection.
                        if let Some(base) = self.last_row_selected {
                            let (low, high) = (base.min(index), base.max(index));
                            self.selected.extend(low..=high);
                        } else {
                            self.selected.insert(index);
                            self.last_row_selected = Some(index);
                        }
                    }
                    _ => {
                        self.selected = HashSet::from([index]);
                        self.last_row_selected = Some(index);
                    }
                }
                self.selected_changed();
                return Action::Reapply;
            }
            Message::SetOption(opt) => {
                let mut indices = self.selected.iter().copied().collect::<Vec<_>>();
                indices.sort();
                for idx in indices {
                    self.files[idx].revert();
                    self.files[idx].add_option(opt(idx));
                }
                self.preview();
            }
            Message::SyncHeader(offset) => {
                return Action::Run(widget::operation::scroll_to(self.header_id.clone(), offset));
            }
            Message::UpLevel => {
                if let Some(path) = self.path.clone() {
                    return Action::Run(Task::done(Message::DirSelected(
                        path.parent().map(|p| p.to_path_buf()),
                    )));
                }
            }
        }
        if let Some(p) = &self.path
            && !updating
        {
            self.path_text = p.display().to_string();
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
        .on_row_press(Message::RowPressed)
        .cell_padding(6)
        .min_width(890.0)
        .into();

        column![
            row![
                button("🗁").on_press(Message::FolderDialog),
                text_input(&self.path_text, &self.path_text)
                    .on_submit(Message::DirSubmitted)
                    .on_input(Message::NewDir),
                button("▲").on_press(Message::UpLevel),
            ],
            tbl
        ]
        .into()
    }

    fn selected_changed(&mut self) {
        for (row, (data, file)) in self.rows.iter_mut().zip(self.files.iter_mut()).enumerate() {
            data.is_selected = self.selected.contains(&row);
            if !self.selected.contains(&row) {
                file.clear_options();
            }
        }
        self.preview();
    }

    pub fn _process(&mut self) -> Vec<(PathBuf, FileError)> {
        let mut errors = Vec::new();
        for idx in &self.selected {
            let ren = mem::take(&mut self.files[*idx]);
            if let Err(e) = ren.rename() {
                errors.push(e);
            }
        }
        self.selected.clear();
        self.populate();
        errors
    }

    pub fn preview(&mut self) {
        for (idx, (row, file)) in self.rows.iter_mut().zip(self.files.iter_mut()).enumerate() {
            let name = if self.selected.contains(&idx) {
                file.preview()
            } else {
                file.revert();
                PathBuf::new()
            };
            row.renamed = name.display().to_string();
        }
    }
}

#[derive(Default, Copy, Clone)]
struct DisplayColumn {
    width: f32,
    min_size: f32,
    resize_offset: Option<f32>,
    sort: Option<bool>,
}

impl DisplayColumn {
    fn new(size: f32) -> Self {
        Self {
            width: size,
            min_size: size,
            ..Default::default()
        }
    }
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
        let cell = text(&row[col_index]);
        if row.is_selected {
            container(cell)
                .width(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.5, 0.5, 0.5).into()),
                    ..Default::default()
                })
                .into()
        } else {
            cell.into()
        }
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
    Reapply,
}

#[derive(Clone)]
pub enum Message {
    ClearOption(RenameOption),
    ColumnDragged(usize, f32),
    ColumnReleased,
    FolderDialog,
    DirSelected(Option<PathBuf>),
    DirSubmitted,
    HeaderPressed(usize),
    Modifier(Modifiers),
    NewDir(String),
    RowPressed(usize),
    SetOption(Arc<dyn Fn(usize) -> RenameOption + Send + Sync>),
    SyncHeader(scrollable::AbsoluteOffset),
    UpLevel,
}

#[derive(Clone)]
struct FileData {
    original: String,
    renamed: String,
    extension: String,
    size: Option<u64>,
    size_string: String,
    modified: Option<DateTime<Local>>,
    created: Option<DateTime<Local>>,
    modified_format: String,
    created_format: String,
    is_selected: bool,
}

impl FileData {
    fn format_data(&self, index: usize) -> &String {
        if index == 4 {
            &self.modified_format
        } else {
            &self.created_format
        }
    }
}

impl std::ops::Index<usize> for FileData {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.original,
            1 => &self.extension,
            2 => &self.renamed,
            3 => &self.size_string,
            4 | 5 => self.format_data(index),
            _ => unreachable!("Bad index"),
        }
    }
}

impl From<&mut Renamer> for FileData {
    fn from(file: &mut Renamer) -> Self {
        let data = file.info();
        Self {
            original: data.0.to_string(),
            extension: data.2.map_or_else(String::new, str::to_string),
            renamed: String::new(),
            size: data.3,
            size_string: data.3.map_or_else(String::new, |s| format!("{s} B")),
            modified: data.4,
            modified_format: data
                .4
                .map_or_else(String::new, |d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            created: data.5,
            created_format: data
                .5
                .map_or_else(String::new, |d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            is_selected: false,
        }
    }
}
