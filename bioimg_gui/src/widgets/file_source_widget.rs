use std::fmt::Write;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;
use std::path::Path;

use parking_lot as pl;

use bioimg_runtime as rt;
use bioimg_runtime::zip_archive_ext::ZipArchiveIdentifier;
use bioimg_runtime::zip_archive_ext::SharedZipArchive;

use crate::project_data::{FileSourceWidgetRawData, LocalFileSourceWidgetRawData};
use crate::result::{GuiError, Result};
use crate::widgets::popup_widget::draw_fullscreen_popup;

use super::collapsible_widget::SummarizableWidget;
use super::util::fire_and_forget;
use super::{
    error_display::show_error,
    popup_widget::PopupResult,
    search_and_pick_widget::SearchAndPickWidget,
    url_widget::StagingUrl,
    Restore, StatefulWidget, ValueWidget,
};


#[derive(Default)]
pub enum LocalFileState{
    #[default]
    Empty,
    Failed(GuiError),
    InMemoryFile{name: Option<String>, data: Arc<[u8]>},
    #[cfg(not(target_arch="wasm32"))]
    PickedNormalFile{path: Arc<Path>},
    #[cfg(not(target_arch="wasm32"))]
    PickingInner{archive: SharedZipArchive, inner_options_widget: SearchAndPickWidget<String>}
}

impl LocalFileState{
    #[cfg(not(target_arch="wasm32"))]
    fn from_local_path(path: &Path, inner_path: Option<String>) -> LocalFileState{
        if !path.exists(){ //FIXME: use smol and await?
            return LocalFileState::Failed(GuiError::new("File does not exist"))
        }
        if path.extension().is_none() || matches!(path.extension(), Some(ext) if ext != "zip"){
            return LocalFileState::PickedNormalFile { path: Arc::from(path) }
        }
        let archive = match SharedZipArchive::open(&path){
            Ok(arch) => arch,
            Err(err) => return LocalFileState::Failed(GuiError::from(err))
        };
        let mut inner_options: Vec<String> = archive.with_file_names(|file_names| {
            file_names
                .filter(|fname| !fname.ends_with('/') && !fname.ends_with('\\'))
                .map(|fname| fname.to_owned())
                .collect()
        });
        inner_options.sort();
        let selected_inner_path = match inner_options.first(){
            None => return LocalFileState::Failed(GuiError::new("Empty zip file")),
            Some(first) => inner_path.unwrap_or(first.clone())
        };
        LocalFileState::PickingInner {
            archive,
            inner_options_widget: SearchAndPickWidget::new(selected_inner_path, inner_options)
        }
    }
}

pub struct LocalFileSourceWidget{
    state: Arc<pl::Mutex<(i64, LocalFileState)>>,
}

impl SummarizableWidget for LocalFileSourceWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        let guard = self.state.lock();
        let (_, state): &(_, LocalFileState) = &*guard;
        match state{
            LocalFileState::Empty => {
                ui.label("Empty");
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
        Self{ state: Arc::new(pl::Mutex::new(state)) }
    }
}

impl Restore for LocalFileSourceWidget{
    type RawData = LocalFileSourceWidgetRawData;
    fn dump(&self) -> Self::RawData {
        let guard = self.state.lock();
        let gen_state: &(i64, LocalFileState) = &*guard;
        match &gen_state.1{
            LocalFileState::Empty | LocalFileState::Failed(_) => Self::RawData::Empty,
            LocalFileState::InMemoryFile{name, data} => {
                let data = Arc::clone(data);
                Self::RawData::InMemoryData{name: name.clone(), data }
            },
            #[cfg(not(target_arch="wasm32"))]
            LocalFileState::PickedNormalFile {path} => {
                Self::RawData::AboutToLoad{path: path.to_string_lossy().into(), inner_path: None}
            },
            LocalFileState::PickingInner { archive, inner_options_widget, .. } => {
                match archive.identifier(){
                    ZipArchiveIdentifier::Path(path) => Self::RawData::AboutToLoad{
                        path: path.to_string_lossy().into(),
                        inner_path: Some(inner_options_widget.value.clone())
                    },
                    _ => Self::RawData::Empty,
                }
            }
        }
    }
    fn restore(&mut self, raw: Self::RawData) {
        match raw{
            Self::RawData::Empty => {
                self.state = Arc::new(pl::Mutex::new((0, LocalFileState::Empty)));
                return
            },
            Self::RawData::InMemoryData{name, data} => {
                self.state = Arc::new(pl::Mutex::new(
                    (0, LocalFileState::InMemoryFile { name, data })
                ));
                return
            },
            Self::RawData::AboutToLoad{path, inner_path} => {
                let pathbuf = PathBuf::from(path);
                *self = LocalFileSourceWidget::from_outer_path(
                    Arc::from(pathbuf.as_path()),
                    inner_path,
                    None,
                );
                return
            }
        };
    } 
}

impl LocalFileSourceWidget{
    pub fn new(state: LocalFileState) -> Self{
        Self{
            state: Arc::new(pl::Mutex::new((0, state)))
        }
    }
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
            state: Arc::new(pl::Mutex::new(
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
    state: Arc<pl::Mutex<(i64, LocalFileState)>>,
    ctx: Option<egui::Context>, //FIXME: always require ctx?
){
    let fut = async move {
        let next_state = 'next: {
            let Some(handle) = rfd::AsyncFileDialog::new().pick_file().await else {
                break 'next LocalFileState::Empty;
            };
            #[cfg(target_arch="wasm32")]
            {
                break 'next LocalFileState::InMemoryFile { name: handle.name(), data: handle.read() } //FIXME: read can panicS
            }
            #[cfg(not(target_arch="wasm32"))]
            LocalFileState::from_local_path(handle.path(), inner_path) //FIXME: maybe use async/await?
        };
        let mut guard = state.lock();
        if guard.0 == generation{
            guard.1 = next_state;
        }
        drop(guard);
        ctx.as_ref().map(|ctx| ctx.request_repaint());
    };
    fire_and_forget(fut);
}

impl StatefulWidget for LocalFileSourceWidget{
    type Value<'p> = Result<rt::FileSource>;
    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        let mut guard = self.state.lock();
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
                    LocalFileState::Empty => (),
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
        let mut guard = self.state.lock();
        let gen_state: &mut (i64, LocalFileState) = &mut *guard;
        let state = &mut gen_state.1;

        match state{
            LocalFileState::Failed(err) => Err(err.clone()),
            LocalFileState::Empty => {
                Err(GuiError::new("Empty"))
            },
            LocalFileState::InMemoryFile { name, data } => {
                let data = Arc::clone(data);
                Ok(rt::FileSource::Data { data, name: name.clone() })
            },
            LocalFileState::PickingInner{archive, inner_options_widget, ..} => Ok(
                rt::FileSource::FileInZipArchive {
                    archive: archive.clone(),
                    inner_path: Arc::from(inner_options_widget.value.as_ref())
                }
            ),
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
    type RawData = FileSourceWidgetRawData;
    fn dump(&self) -> Self::RawData {
        match self.mode {
            FileSourceWidgetMode::Local => {
                Self::RawData::Local(self.local_file_source_widget.dump())
            },
            FileSourceWidgetMode::Url => {
                Self::RawData::Url(self.http_url_widget.dump())
            }
        }
    }
    fn restore(&mut self, raw: Self::RawData) {
        match raw{
            Self::RawData::Local(local) => self.local_file_source_widget.restore(local),
            Self::RawData::Url(url) => self.http_url_widget.restore(url)
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

pub trait FileSourcePopupConfig{
    const BUTTON_TEXT: &'static str = "Open...";
    const TITLE: &'static str = "Choose a file";
}

pub struct DefaultFileSourcePopupConfig;
impl FileSourcePopupConfig for DefaultFileSourcePopupConfig{}

#[derive(Default)]
pub enum FileSourceWidgetPopupButton<C: FileSourcePopupConfig = DefaultFileSourcePopupConfig>{
    #[default]
    Empty,
    Picking{file_source_widget: FileSourceWidget},
    Ready{file_source: rt::FileSource, marker: PhantomData<C>},
}

impl<C: FileSourcePopupConfig> ValueWidget for FileSourceWidgetPopupButton<C>{
    type Value<'v> = rt::FileSource;

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        *self = Self::Ready { file_source: value, marker: PhantomData }
    }
}

impl<C: FileSourcePopupConfig> StatefulWidget for FileSourceWidgetPopupButton<C>{
    type Value<'p> = Result<rt::FileSource> where C: 'p;
    
    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id){
        ui.horizontal(|ui|{
            let open_button_clicked = ui.button(C::BUTTON_TEXT).clicked();
            *self = match (std::mem::take(self), open_button_clicked) {
                (Self::Empty, false) => Self::Empty,
                (Self::Ready { file_source, marker }, false) => {
                    ui.weak(&file_source.to_string());
                    Self::Ready { file_source, marker }
                },
                (Self::Picking { mut file_source_widget }, _) => {
                    let file_source_result: PopupResult<rt::FileSource> = draw_fullscreen_popup(ui, id.with("pop".as_ptr()), C::TITLE, |ui, id|{
                        let mut out = PopupResult::Continued;
                        ui.vertical(|ui|{
                            file_source_widget.draw_and_parse(ui, id);
                            let state = file_source_widget.state();
                            ui.add_space(10.0);
                            ui.horizontal(|ui|{
                                match state {
                                    Ok(file_source) => if ui.button("Ok").clicked(){
                                        out = PopupResult::Finished(file_source);
                                    },
                                    Err(_) => {
                                        ui.add_enabled_ui(false, |ui| ui.button("Ok"));
                                    }
                                };
                                if ui.button("Cancel").clicked(){
                                     out = PopupResult::Closed
                                }
                            });
                        });
                        out
                    });
                    match file_source_result{
                        PopupResult::Continued => Self::Picking { file_source_widget },
                        PopupResult::Closed => Self::Empty,
                        PopupResult::Finished(file_source) => Self::Ready { file_source, marker: PhantomData },
                    }
                },
                (Self::Empty, true) => Self::Picking{ file_source_widget: Default::default() },
                (Self::Ready{ file_source, .. }, true) => Self::Picking{
                    file_source_widget: {
                        let mut widget = FileSourceWidget::default();
                        widget.set_value(file_source);
                        widget
                    }
                },
            };
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        match self {
            Self::Ready { file_source, .. } => {
                Ok(file_source.clone())
            },
            _ => Err(GuiError::new("not ready".to_owned()))
        }
    }
}
