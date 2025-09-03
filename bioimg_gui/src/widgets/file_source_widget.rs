use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(not(target_arch="wasm32"))]
use std::path::Path;

use bioimg_runtime as rt;
use bioimg_runtime::zip_archive_ext::ZipArchiveIdentifier;
use bioimg_runtime::zip_archive_ext::SharedZipArchive;

use crate::project_data::{FileSourceWidgetSavedData, LocalFileSourceWidgetSavedData};
use crate::result::{GuiError, Result};

use super::collapsible_widget::SummarizableWidget;

use super::{Restore, StatefulWidget, ValueWidget};
use super::error_display::show_error;
use super::url_widget::StagingUrl;
use super::search_and_pick_widget::SearchAndPickWidget;

#[derive(Default)]
pub enum LocalFileState{
    #[default]
    Empty,
    Failed(GuiError),
    InMemoryFile{name: Option<String>, data: Arc<[u8]>},
    #[cfg(not(target_arch="wasm32"))]
    PickedNormalFile{path: Arc<Path>},
    PickingInner{archive: SharedZipArchive, inner_options_widget: SearchAndPickWidget<String>}
}

impl LocalFileState{
    pub fn from_failure_msg(message: impl AsRef<str>) -> Self{
        Self::Failed(GuiError::new(message))
    }
    #[cfg(not(target_arch="wasm32"))]
    fn from_local_path(path: &Path, inner_path: Option<String>) -> LocalFileState{
        if !path.exists(){ //FIXME: use smol and await?
            let path_display = path.to_string_lossy();
            return LocalFileState::Failed(GuiError::new(format!("File does not exist: {path_display}")))
        }
        if path.extension().is_none() || matches!(path.extension(), Some(ext) if ext != "zip"){
            return LocalFileState::PickedNormalFile { path: Arc::from(path) }
        }
        let archive = match SharedZipArchive::open(&path){
            Ok(arch) => arch,
            Err(err) => return LocalFileState::Failed(GuiError::from(err))
        };
        Self::from_zip(archive, inner_path)
    }
    fn from_zip(archive: SharedZipArchive, inner_path: Option<String>) -> Self{
        let mut inner_options: Vec<String> = archive.with_file_names(|file_names| {
            file_names
                .filter(|fname| !fname.ends_with('/') && !fname.ends_with('\\'))
                .map(|fname| fname.to_owned())
                .collect()
        });
        inner_options.sort();

        let selected_inner_path = match inner_path{
            Some(inner_path) => {
                if !inner_options.contains(&inner_path){
                    let message = format!("File {} does not contain entry '{inner_path}'", archive.identifier());
                    return Self::from_failure_msg(message)
                }
                inner_path
            },
            None => match inner_options.first(){
                None => return Self::from_failure_msg(format!("Empty zip file: {}", archive.identifier())),
                Some(first) => first.clone(),
            }
        };

        LocalFileState::PickingInner {
            archive,
            inner_options_widget: SearchAndPickWidget::new(selected_inner_path, inner_options)
        }
    }
}

pub struct LocalFileSourceWidget{
    state: Arc<std::sync::Mutex<(i64, LocalFileState)>>,
}

impl SummarizableWidget for LocalFileSourceWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        let guard = self.state.lock().unwrap();
        let (_, state): &(_, LocalFileState) = &*guard;
        match state{
            LocalFileState::Empty => {
                show_error(ui, "Empty");
            },
            LocalFileState::Failed(err) => {
                show_error(ui, err);
            },
            LocalFileState::InMemoryFile{ name, data } => {
                let mut label = String::with_capacity(32);
                if let Some(name) = name {
                    write!(&mut label, "{name} ").unwrap();
                }
                write!(&mut label, "{} bytes", data.len()).unwrap();
                ui.label(label);
            },
            #[cfg(not(target_arch="wasm32"))]
            LocalFileState::PickedNormalFile{path} => {
                ui.label(path.to_string_lossy());
            },
            LocalFileState::PickingInner{ archive, inner_options_widget} => {
                ui.label(format!(
                    "{}/{}",
                    archive.identifier(),
                    inner_options_widget.value,
                ));
            },
        }
    }
}

impl Default for LocalFileSourceWidget{
    fn default() -> Self {
        let state = (0, LocalFileState::default());
        Self{ state: Arc::new(std::sync::Mutex::new(state)) }
    }
}

impl Restore for LocalFileSourceWidget{
    type SavedData = LocalFileSourceWidgetSavedData;
    fn dump(&self) -> Self::SavedData {
        let guard = self.state.lock().unwrap();
        let gen_state: &(i64, LocalFileState) = &*guard;
        match &gen_state.1{
            LocalFileState::Empty | LocalFileState::Failed(_) => Self::SavedData::Empty,
            LocalFileState::InMemoryFile{name, data} => {
                let data = Arc::clone(data);
                Self::SavedData::InMemoryData{name: name.clone(), data }
            },
            #[cfg(not(target_arch="wasm32"))]
            LocalFileState::PickedNormalFile {path} => {
                Self::SavedData::AboutToLoad{path: path.to_string_lossy().into(), inner_path: None}
            },
            LocalFileState::PickingInner { archive, inner_options_widget, .. } => {
                match archive.identifier(){
                    ZipArchiveIdentifier::Path(path) => Self::SavedData::AboutToLoad{
                        path: path.to_string_lossy().into(),
                        inner_path: Some(inner_options_widget.value.clone())
                    },
                    _ => Self::SavedData::Empty,
                }
            }
        }
    }
    fn restore(&mut self, saved_data: Self::SavedData) {
        match saved_data{
            Self::SavedData::Empty => {
                self.state = Arc::new(std::sync::Mutex::new((0, LocalFileState::Empty)));
                return
            },
            Self::SavedData::InMemoryData{name, data} => {
                self.state = Arc::new(std::sync::Mutex::new(
                    (0, LocalFileState::InMemoryFile { name, data })
                ));
                return
            },
            Self::SavedData::AboutToLoad{path, inner_path} => {
                #[cfg(target_arch="wasm32")]
                eprintln!("Warning: can't load local path {path}/{inner_path:?} in wasm32"); //FIXME
                #[cfg(not(target_arch="wasm32"))]
                {
                    let pathbuf = PathBuf::from(path);
                    *self = LocalFileSourceWidget::from_outer_path(
                        Arc::from(pathbuf.as_path()),
                        inner_path,
                        None,
                    );
                }
            }
        };
    } 
}

impl LocalFileSourceWidget{
    pub fn new(state: LocalFileState) -> Self{
        Self{
            state: Arc::new(std::sync::Mutex::new((0, state)))
        }
    }
    #[cfg(not(target_arch="wasm32"))]
    pub fn from_outer_path(
        path: Arc<Path>,
        inner_path: Option<String>,
        ctx: Option<egui::Context>,
    ) -> Self{
        ctx.map(|ctx| ctx.request_repaint());
        Self::new(LocalFileState::from_local_path(&path, inner_path))
    }
    pub fn from_data(name: Option<String>, data: Arc<[u8]>) -> Self{
        Self{
            state: Arc::new(std::sync::Mutex::new(
                (
                    0,
                    LocalFileState::InMemoryFile {
                        name,
                        data,
                    }
                )
            ))
        }
    }
}



pub fn spawn_load_file_task(
    inner_path: Option<String>,
    generation: i64,
    state: Arc<std::sync::Mutex<(i64, LocalFileState)>>,
    ctx: Option<egui::Context>, //FIXME: always require ctx?
){
    let fut = async move {
        let next_state = 'next: {
            let Some(handle) = rfd::AsyncFileDialog::new().pick_file().await else {
                break 'next LocalFileState::Empty;
            };
            #[cfg(target_arch="wasm32")]
            {
                let contents = handle.read().await; //FIXME: read can panic
                if matches!(PathBuf::from(handle.file_name()).extension(), Some(ext) if ext == "zip"){
                    let archive = SharedZipArchive::from_raw_data(contents, handle.file_name());
                    break 'next LocalFileState::from_zip(archive, inner_path)
                } else {
                    let data: Arc<[u8]> = Arc::from(contents.as_slice());
                    break 'next LocalFileState::InMemoryFile { name: Some(handle.file_name()), data }
                }
            }
            #[cfg(not(target_arch="wasm32"))]
            LocalFileState::from_local_path(handle.path(), inner_path) //FIXME: maybe use async/await?
        };
        let mut guard = state.lock().unwrap();
        if guard.0 == generation{
            guard.1 = next_state;
        }
        drop(guard);
        ctx.as_ref().map(|ctx| ctx.request_repaint());
    };

    #[cfg(target_arch="wasm32")]
    wasm_bindgen_futures::spawn_local(fut);
    #[cfg(not(target_arch="wasm32"))]
    std::thread::spawn(move || smol::block_on(fut));
}

impl StatefulWidget for LocalFileSourceWidget{
    type Value<'p> = Result<rt::FileSource>;
    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        let mut guard = self.state.lock().unwrap();
        let gen_state: &mut (i64, LocalFileState) = &mut *guard;
        let generation = &mut gen_state.0;
        let state = &mut gen_state.1;

        ui.vertical(|ui|{
            ui.horizontal(|ui|{
                if ui.button("Open...").clicked(){
                    *generation += 1;
                    spawn_load_file_task(
                        None,
                        *generation,
                        Arc::clone(&self.state),
                        Some(ui.ctx().clone()),
                    );
                }
                match state{
                    LocalFileState::Empty => {
                        show_error(ui, "Please select a file");
                    },
                    LocalFileState::Failed(err) => {
                        show_error(ui, err);
                    },
                    LocalFileState::InMemoryFile { name, data } => {
                        let mut label = String::new();
                        if let Some(name) = name {
                            write!(&mut label, "{name} ").unwrap();
                        }
                        write!(&mut label, "({} bytes)", data.len()).unwrap();
                        ui.weak(label);
                    },
                    #[cfg(not(target_arch="wasm32"))]
                    LocalFileState::PickedNormalFile{path} => {
                        ui.weak(path.to_string_lossy());
                    },
                    LocalFileState::PickingInner{archive, ..} => {
                        ui.weak(archive.identifier().to_string());
                    }
                }
            });
            if let LocalFileState::PickingInner{inner_options_widget, ..} = state {
                ui.horizontal(|ui|{
                    ui.strong("Inner Path: ");
                    inner_options_widget.draw_and_parse(ui, id.with("inner_widget".as_ptr()));
                });
            }
        });
    }
    fn state<'p>(&'p self) -> Self::Value<'p> {
        let mut guard = self.state.lock().unwrap();
        let gen_state: &mut (i64, LocalFileState) = &mut *guard;
        let state = &mut gen_state.1;

        match state{
            LocalFileState::Failed(err) => Err(err.clone()),
            LocalFileState::Empty => {
                Err(GuiError::new("Empty"))
            },
            LocalFileState::InMemoryFile { name, data, } => {
                let data = Arc::clone(data);
                Ok(rt::FileSource::Data { data, name: name.clone() })
            },
            LocalFileState::PickingInner{archive, inner_options_widget, ..} => Ok(
                rt::FileSource::FileInZipArchive {
                    archive: archive.clone(),
                    inner_path: Arc::from(inner_options_widget.value.as_ref())
                }
            ),
            #[cfg(not(target_arch="wasm32"))]
            LocalFileState::PickedNormalFile{path} => {
                Ok(rt::FileSource::LocalFile{path: path.clone()})
            },
        }
    }
}

#[derive(Default, PartialEq, Eq, strum::VariantArray, Copy, Clone, strum::Display, strum::AsRefStr)]
pub enum FileSourceWidgetMode{
    #[default]
    #[strum(to_string = "Local File")]
    Local,
    Url,
}

#[derive(Default)]
pub struct FileSourceWidget{
    pub mode: FileSourceWidgetMode,
    pub local_file_source_widget: LocalFileSourceWidget,
    pub http_url_widget: StagingUrl,
}

impl SummarizableWidget for FileSourceWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        match self.mode {
            FileSourceWidgetMode::Local => self.local_file_source_widget.summarize(ui, id.with("local".as_ptr())),
            FileSourceWidgetMode::Url => match self.http_url_widget.state(){
                Ok(url) => {
                    ui.label(url.to_string());
                },
                Err(err) => show_error(ui, err),
            }
        }
    }
}

impl Restore for FileSourceWidget{
    type SavedData = FileSourceWidgetSavedData;
    fn dump(&self) -> Self::SavedData {
        match self.mode {
            FileSourceWidgetMode::Local => {
                Self::SavedData::Local(self.local_file_source_widget.dump())
            },
            FileSourceWidgetMode::Url => {
                Self::SavedData::Url(self.http_url_widget.dump())
            }
        }
    }
    fn restore(&mut self, saved_data: Self::SavedData) {
        match saved_data{
            Self::SavedData::Local(local) => self.local_file_source_widget.restore(local),
            Self::SavedData::Url(url) => self.http_url_widget.restore(url)
        }
    }
}

impl ValueWidget for FileSourceWidget{
    type Value<'v> = rt::FileSource;

    fn set_value(&mut self, value: rt::FileSource){
        match value{
            rt::FileSource::Data { data, name } => {
                self.mode = FileSourceWidgetMode::Local; //FIXME: double check
                self.local_file_source_widget = LocalFileSourceWidget::from_data(name.clone(), data);
            },
            #[cfg(not(target_arch="wasm32"))]
            rt::FileSource::LocalFile { path } => {
                self.mode = FileSourceWidgetMode::Local;
                self.local_file_source_widget = LocalFileSourceWidget::from_outer_path(path, None, None);
            },
            rt::FileSource::FileInZipArchive { inner_path, archive} => {
                self.mode = FileSourceWidgetMode::Local;
                self.local_file_source_widget = {
                    let mut inner_options: Vec<String> = archive.with_file_names(|file_names| {
                        file_names
                            .filter(|fname| !fname.ends_with('/') && !fname.ends_with('\\'))
                            .map(|fname| fname.to_owned())
                            .collect()
                    });
                    inner_options.sort();
                    LocalFileSourceWidget::new(LocalFileState::PickingInner {
                        archive,
                        inner_options_widget: SearchAndPickWidget::new(inner_path.as_ref().to_owned(), inner_options),
                    })
                };
            },
            rt::FileSource::HttpUrl(url) => {
                self.mode = FileSourceWidgetMode::Url;
                self.http_url_widget.set_value(url);
            },
        };
    }
}

impl FileSourceWidget{
    pub fn update(&mut self){
        self.http_url_widget.update();
        // self.local_file_source_widget.update();
    }
}

impl StatefulWidget for FileSourceWidget{
    type Value<'p> = Result<rt::FileSource>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui|{
                ui.radio_value(&mut self.mode, FileSourceWidgetMode::Local, "Local File")
                    .on_hover_text("Pick a file form the local filesystem");
                ui.radio_value(&mut self.mode, FileSourceWidgetMode::Url, "Url")
                    .on_hover_text("Specify a file on the web by its HTTP URL");
            });
            match self.mode{
                FileSourceWidgetMode::Local => {
                    self.local_file_source_widget.draw_and_parse(ui, id.with("local".as_ptr()));
                },
                FileSourceWidgetMode::Url => {
                    self.http_url_widget.draw_and_parse(ui, id.with("url".as_ptr()));
                },
            }
        });
    }

    fn state(&self) -> Result<rt::FileSource>{
        return match self.mode {
            FileSourceWidgetMode::Local => self.local_file_source_widget.state(),
            FileSourceWidgetMode::Url => Ok(
                rt::FileSource::HttpUrl(
                    self.http_url_widget.state().map_err(|_| GuiError::new("Invalid HTTP URL"))?
                )
            ),
        }
    }
}
