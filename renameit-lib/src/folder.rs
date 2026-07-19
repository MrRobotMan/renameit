use super::{Process, Renamer};
use std::{
    cmp::min,
    env,
    fmt::Write,
    path::{Component, Prefix},
};

/// Add the name of the containing folder or hierarchy of folders.
/// These can be added in prefix or suffix `Mode`, with a `Sep`arator specified and the
/// maximum number of `Levels` selected.
///
/// On Windows, if the hierarchy reaches the drive root (i.e. C:\ on windows, \\ on linux)
/// the ":\" or "\\"characters will be automatically removed.
#[derive(Default, Debug, Clone)]
pub struct FolderOptions {
    pub mode: FolderMode,
    pub sep: String,
    pub levels: i32,
}

impl Process for FolderOptions {
    fn process(&self, file: &mut Renamer) {
        let mut parts = file.original.components().rev();
        parts.next(); // Skip the file itself.
        let components: Vec<_> = parts
            .filter_map(|p| match p {
                Component::Normal(s) => Some(s.to_str()),
                Component::Prefix(prefix) => match prefix.kind() {
                    Prefix::Verbatim(s) => Some(s.to_str()),
                    Prefix::VerbatimUNC(_, s) => Some(s.to_str()),
                    Prefix::VerbatimDisk(_) => Some(prefix.as_os_str().to_str()),
                    Prefix::DeviceNS(s) => Some(s.to_str()),
                    Prefix::UNC(_, s) => Some(s.to_str()),
                    Prefix::Disk(_) => Some(prefix.as_os_str().to_str()),
                },
                _ => None,
            })
            .collect();
        let end = min(components.len(), self.levels.unsigned_abs() as usize);
        let start = if self.levels >= 0 {
            0
        } else {
            end.saturating_sub(1)
        };
        match self.mode {
            FolderMode::Prefix => {
                for component in components[start..end].iter().flatten() {
                    let mut component = component.replace(r"\\?\", "");
                    if env::consts::OS == "windows" {
                        component = component.replace(':', "")
                    }
                    file.stem
                        .insert_str(0, &format!("{}{}", component, self.sep));
                }
            }
            FolderMode::Suffix => {
                for component in components[start..end].iter().flatten() {
                    let mut component = component.replace(r"\\?\", "");
                    if env::consts::OS == "windows" {
                        component = component.replace(':', "")
                    }
                    write!(file.stem, "{}{}", component, self.sep)
                        .expect("Unexpected error appending string.")
                }
            }
            _ => (),
        };
    }
}

/// Select from
/// `FolderMode::Prefix` or
/// `FolderMode::Suffix`.
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum FolderMode {
    #[default]
    None,
    Prefix,
    Suffix,
}

/*
#[derive(Default)]
pub struct FolderView {
    mode: FolderMode,
    level: ValText<i32>,
    sep: String,
    width: f32,
}

impl FolderView {
    pub fn new(width: f32) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }
}

impl OptionBuilder for FolderView {
    type Processor = FolderOptions;

    fn build(&self) -> FolderOptions {
        FolderOptions {
            mode: self.mode,
            sep: self.sep.clone(),
            levels: self.level.get_val().unwrap_or(0),
        }
    }
}

impl Incrementer for &mut FolderView {
    fn increment(&mut self, _field: &str) {
        self.level.set_val(self.level.get_val().unwrap_or(0) + 1)
    }

    fn decrement(&mut self, _field: &str) {
        self.level.set_val(self.level.get_val().unwrap_or(0) - 1)
    }
}

impl Widget for &mut FolderView {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.set_width(self.width);
            ui.label("Append Folder Name");
            ui.horizontal(|ui| {
                ComboBox::new("Append File Name", "")
                    .selected_text(format!("{:?}", &self.mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.mode, FolderMode::None, "None");
                        ui.selectable_value(&mut self.mode, FolderMode::Prefix, "Prefix");
                        ui.selectable_value(&mut self.mode, FolderMode::Suffix, "Suffix")
                    });
                ui.label("Sep.");
                ui.add(TextEdit::singleline(&mut self.sep).desired_width(NUM_WIDTH * 2.0));
                ui.separator();
                ui.label("Pos.");
                if ui
                    .add(TextEdit::singleline(&mut self.level).desired_width(NUM_WIDTH))
                    .changed()
                    && !self.level.is_valid()
                {
                    self.level.revert();
                };
                ui.add(Arrows::new("Folder Arrows", &mut self, "level"));
            });
        })
        .response
    }
}
*/

#[cfg(test)]
mod folder_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn prefix_two_levels() {
        let mut file = Renamer::new(Path::new("/some/file/path/to/test file.txt")).unwrap();
        let mode = FolderMode::Prefix;
        let sep = "~".into();
        let levels = 2;
        let opt = FolderOptions { mode, sep, levels };
        opt.process(&mut file);
        assert_eq!(file.stem, "path~to~test file".to_string())
    }

    #[test]
    fn suffix_negative_two_levels() {
        let mut file = Renamer::new(Path::new(r"/some/file/path/to/test file.txt")).unwrap();
        let mode = FolderMode::Prefix;
        let sep = "~".into();
        let levels = -2;
        let opt = FolderOptions { mode, sep, levels };
        opt.process(&mut file);
        assert_eq!(file.stem, "path~test file".to_string())
    }
}
