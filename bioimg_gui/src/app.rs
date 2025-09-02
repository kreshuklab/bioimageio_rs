use std::sync::Arc;
use std::thread::JoinHandle;

use bioimg_runtime::zip_archive_ext::SharedZipArchive;
use bioimg_spec::rdf::model::model_rdf_0_5::PartialModelRdfV0_5;
use bioimg_spec::rdf::model::ModelRdfName;
use bioimg_zoo::collection::ZooNickname;
use indoc::indoc;

use bioimg_runtime as rt;
use bioimg_runtime::zoo_model::ZooModel;
use bioimg_spec::rdf;
use bioimg_spec::rdf::ResourceId;
use bioimg_spec::rdf::bounded_string::BoundedString;
use bioimg_spec::rdf::non_empty_list::NonEmptyList;

use crate::project_data::{AppState1SavedData};
#[cfg(not(target_arch="wasm32"))]
use crate::project_data::{AppStateSavedData, ProjectLoadError};
use crate::result::{GuiError, Result, VecResultExt};
use crate::widgets::attachments_widget::AttachmentsWidget;

use crate::widgets::code_editor_widget::MarkdwownLang;
use crate::widgets::collapsible_widget::SummarizableWidget;
use crate::widgets::icon_widget::IconWidgetValue;
use crate::widgets::image_widget_2::SpecialImageWidget;
use crate::widgets::json_editor_widget::JsonObjectEditorWidget;
use crate::widgets::model_interface_widget::ModelInterfaceWidget;
use crate::widgets::model_links_widget::ModelLinksWidget;
use crate::widgets::notice_widget::{Notification, NotificationsWidget};
use crate::widgets::pipeline_widget::PipelineWidget;
use crate::widgets::search_and_pick_widget::SearchAndPickWidget;
use crate::widgets::staging_opt::StagingOpt;
use crate::widgets::staging_string::{InputLines, StagingString};
use crate::widgets::staging_vec::StagingVec;
use crate::widgets::util::{widget_vec_from_values, TaskChannel, VecItemRender, VecWidget};
use crate::widgets::version_widget::VersionWidget;
use crate::widgets::weights_widget::WeightsWidget;
#[cfg(not(target_arch="wasm32"))]
use crate::widgets::zoo_widget::{upload_model, ZooLoginWidget};
use crate::widgets::ValueWidget;
use crate::widgets::Restore;
use crate::widgets::{
    author_widget::AuthorWidget, cite_widget::CiteEntryWidget, code_editor_widget::CodeEditorWidget,
    icon_widget::IconWidget, maintainer_widget::MaintainerWidget, url_widget::StagingUrl,
    util::group_frame, StatefulWidget,
};

pub struct AppStateFromPartial{
    state: AppState1SavedData,
    warnings: String,
}

#[must_use]
pub enum TaskResult{
    Notification(Result<String, String>),
    ModelImport(Box<rt::zoo_model::ZooModel>),
    PartialModelLoad(AppStateFromPartial),
}

impl TaskResult{
    pub fn ok_message(msg: impl Into<String>) -> Self{
        Self::Notification(Ok(msg.into()))
    }
    pub fn err_message(msg: impl Into<String>) -> Self{
        Self::Notification(Err(msg.into()))
    }
}

#[derive(Default, Copy, Clone)]
enum ExitingStatus{
    #[default]
    NotExiting,
    Confirming,
    Exiting,
}

#[derive(Restore)]
#[restore(saved_data=crate::project_data::AppState1SavedData)]
pub struct AppState1 {
    pub staging_name: StagingString<ModelRdfName>,
    pub staging_description: StagingString<BoundedString<0, 1024>>,
    pub cover_images: Vec<SpecialImageWidget<rt::CoverImage>>,
    pub model_id_widget: StagingOpt<StagingString<ResourceId>, false>,
    pub staging_authors: Vec<AuthorWidget>,
    pub attachments_widget: Vec<AttachmentsWidget>,
    pub staging_citations: Vec<CiteEntryWidget>,
    pub custom_config_widget: StagingOpt<JsonObjectEditorWidget, false>, //FIXME
    pub staging_git_repo: StagingOpt<StagingUrl, false>,
    pub icon_widget: StagingOpt<IconWidget>,
    pub links_widget: ModelLinksWidget,
    pub staging_maintainers: Vec<MaintainerWidget>,
    pub staging_tags: StagingVec<StagingString<rdf::Tag>>,
    pub staging_version: StagingOpt<VersionWidget, false>,
    pub staging_version_comment: StagingOpt<StagingString<BoundedString<0, 512>>, false>,

    pub staging_documentation: CodeEditorWidget<MarkdwownLang>,
    pub staging_license: SearchAndPickWidget<rdf::LicenseId>,
    //badges
    pub model_interface_widget: ModelInterfaceWidget,
    ////
    pub weights_widget: WeightsWidget,



    #[restore(default)]
    pub pipeline_widget: PipelineWidget,



    #[cfg(not(target_arch="wasm32"))]
    #[restore(default)]
    pub zoo_login_widget: ZooLoginWidget,
    #[restore(default)]
    pub zoo_model_creation_task: Option<JoinHandle<Result<ZooNickname>>>,

    #[restore(default)]
    pub notifications_widget: NotificationsWidget,
    #[restore(default)]
    pub notifications_channel: TaskChannel<TaskResult>,
    #[restore(default)]
    exiting_status: ExitingStatus,
}

impl ValueWidget for AppState1{
    type Value<'v> = rt::zoo_model::ZooModel;

    fn set_value<'v>(&mut self, zoo_model: Self::Value<'v>) {
        self.staging_name.set_value(zoo_model.name);
        self.staging_description.set_value(zoo_model.description);
        self.cover_images = zoo_model.covers.into_iter()
            .map(|cover| {
                let mut widget = SpecialImageWidget::default();
                widget.set_value((None, Some(cover)));
                widget
            })
            .collect();
        self.model_id_widget.set_value(zoo_model.id);
        self.staging_authors = zoo_model.authors.into_inner().into_iter()
            .map(|descr| {
                let mut widget = AuthorWidget::default();
                widget.set_value(descr);
                widget
            })
            .collect();
        self.attachments_widget = widget_vec_from_values(zoo_model.attachments);
        self.staging_citations = zoo_model.cite.into_inner().into_iter()
            .map(|descr|{
                let mut widget = CiteEntryWidget::default();
                widget.set_value(descr);
                widget
            })
            .collect();
        self.custom_config_widget.set_value(
            if zoo_model.config.is_empty(){
                None
            } else {
                Some(zoo_model.config)
            }
        );
        self.staging_git_repo.set_value(zoo_model.git_repo.map(|val| Arc::new(val)));
        self.icon_widget.set_value(zoo_model.icon.map(IconWidgetValue::from));
        self.links_widget.set_value(zoo_model.links);
        self.staging_maintainers = zoo_model.maintainers.into_iter()
            .map(|val| {
                let mut widget = MaintainerWidget::default();
                widget.set_value(val);
                widget
            })
            .collect();
        self.staging_tags.set_value(zoo_model.tags);
        self.staging_version.set_value(zoo_model.version);
        self.staging_documentation.set_value(&zoo_model.documentation);
        self.staging_license.set_value(zoo_model.license);

        self.model_interface_widget.set_value(zoo_model.interface);

        self.weights_widget.set_value(zoo_model.weights);
    }
}

impl Default for AppState1 {
    fn default() -> Self {
        Self {
            staging_name: StagingString::new(InputLines::SingleLine),
            staging_description: StagingString::new(InputLines::Multiline),
            cover_images: Vec::default(),
            model_id_widget: Default::default(),
            staging_authors: Default::default(),
            attachments_widget: Default::default(),
            staging_citations: Default::default(),
            custom_config_widget: Default::default(),
            staging_git_repo: Default::default(),
            icon_widget: Default::default(),
            links_widget: Default::default(),
            staging_maintainers: Default::default(),
            staging_tags: StagingVec::default(),
            staging_version: Default::default(),
            staging_version_comment: Default::default(),
            staging_documentation: Default::default(),
            staging_license: SearchAndPickWidget::from_enum(Default::default()),

            model_interface_widget: Default::default(),

            weights_widget: Default::default(),
            notifications_widget: NotificationsWidget::new(),
            notifications_channel: Default::default(),
            #[cfg(not(target_arch="wasm32"))]
            zoo_login_widget: Default::default(),
            zoo_model_creation_task: Default::default(),
            pipeline_widget: Default::default(),

            exiting_status: Default::default(),
        }
    }
}

impl AppState1{
    pub fn create_model(&self) -> Result<ZooModel>{
        let name = self.staging_name.state()
            .cloned()
            .map_err(|e| GuiError::new_with_rect("Check resoure name for errors", e.failed_widget_rect))?;
        let description = self.staging_description.state()
            .cloned()
            .map_err(|e| GuiError::new_with_rect("Check resource text description for errors", e.failed_widget_rect))?;
        let covers: Vec<_> = self.cover_images.iter()
            .map(|cover_img_widget|{
                cover_img_widget.state()
                    .map(|val| val.clone())
                    .map_err(|e| GuiError::new_with_rect("Check cover images for errors", e.failed_widget_rect))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let model_id = self.model_id_widget.state().transpose()
            .map_err(|e| GuiError::new_with_rect("Check model id for errors", e.failed_widget_rect))?
            .cloned();
        let authors = NonEmptyList::try_from(
                self.staging_authors.iter()
                    .enumerate()
                    .map(|(idx, widget)| {
                        widget.state().map_err(|err| {
                            GuiError::new_with_rect(format!("Check author #{} for errors", idx + 1), err.failed_widget_rect)
                        })
                    })
                    .collect::<Result<Vec<_>>>()?
            )
            .map_err(|_| GuiError::new("Empty authors"))?;
        let attachments = self.attachments_widget.iter()
            .enumerate()
            .map(|(idx, widget)| {
                widget.state().map_err(|_| GuiError::new(format!("Check attachment #{} for errors", idx + 1)))
            })
            .collect::<Result<Vec<_>>>()?;
            // .collect_result()
            // .map_err(|e| GuiError::new_with_rect("Check model attachments for errors", e.failed_widget_rect))?;
        let cite = self.staging_citations.iter().enumerate()
            .map(|(idx, widget)| {
                widget.state().map_err(|_| GuiError::new(format!("Check citation #{} for errors", idx + 1)))
            })
            .collect::<Result<Vec<_>>>()?;
        let non_empty_cites = NonEmptyList::try_from(cite)
            .map_err(|_| GuiError::new("Cites are empty"))?;
        let config = self.custom_config_widget.state().cloned()
            .transpose()
            .map_err(|e| GuiError::new_with_rect("Check custom configs for errors", e.failed_widget_rect))?
            .unwrap_or(serde_json::Map::default());
        let git_repo = self.staging_git_repo.state()
            .transpose()
            .map_err(|e| GuiError::new_with_rect("Check git repo field for errors", e.failed_widget_rect))?
            .map(|val| val.as_ref().clone());
        let icon = self.icon_widget.state().transpose().map_err(|_| GuiError::new("Check icons field for errors"))?;
        let links = self.links_widget.state()
            .collect_result()
            .map_err(|e| GuiError::new_with_rect("Check links for errors", e.failed_widget_rect))?
            .into_iter()
            .map(|s| s.clone())
            .collect();
        let maintainers = self.staging_maintainers.iter()
            .enumerate()
            .map(|(idx, w)| {
                w.state().map_err(|_| GuiError::new(format!("Check maintainer #{} for errors", idx + 1)))
            })
            .collect::<Result<Vec<_>>>()?;
        let tags: Vec<rdf::Tag> = self.staging_tags.state()
            .into_iter()
            .map(|res_ref| res_ref.cloned())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| GuiError::new_with_rect("Check tags for errors", e.failed_widget_rect))?;
        let version = self.staging_version.state()
            .transpose()
            .map_err(|e| GuiError::new_with_rect("Review resource version field", e.failed_widget_rect))?
            .cloned();
        let version_comment = self
            .staging_version_comment
            .state()
            .transpose()
            .map_err(|e| GuiError::new_with_rect("Review resource version comment field", e.failed_widget_rect))?
            .cloned();
        let documentation = self.staging_documentation.state().to_owned();
        let license = self.staging_license.state();
        let model_interface = self.model_interface_widget.get_value()
            .map_err(|_| GuiError::new("Check model interface for errors"))?;
        let weights = self.weights_widget.get_value()
            .map_err(|e| GuiError::new_with_rect("Check model weights for errors", e.failed_widget_rect))?
            .as_ref().clone();

        Ok(ZooModel {
            name,
            description,
            covers,
            attachments,
            cite: non_empty_cites,
            config,
            git_repo,
            icon,
            links,
            maintainers,
            tags,
            version,
            version_comment,
            authors,
            documentation,
            license,
            id: model_id,
            weights,
            interface: model_interface,
        })
    }

    #[cfg(not(target_arch="wasm32"))]
    fn save_project(&self, project_file: &std::path::Path) -> Result<String, String>{
        let writer = std::fs::File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(project_file).map_err(|err| format!("Could not open project file for writing: {err}"))?;
        AppStateSavedData::Version1(self.dump()).save(writer)
            .map_err(|err| format!("Could not serialize project to bytes: {err}"))
            .map(|_| format!("Saved project to {}", project_file.to_string_lossy()))
    }

    #[cfg(not(target_arch="wasm32"))]
    fn load_project(&mut self, project_file: &std::path::Path) -> Result<(), String>{
        let reader = std::fs::File::open(&project_file).map_err(|err| format!("Could not open project file: {err}"))?;
        let proj_data = match AppStateSavedData::load(reader){
            Err(ProjectLoadError::FutureVersion{found_version}) => return Err(format!(
                "Found project data version {found_version}, but this program only supports project data up to {}\n\
                You can try downloading the newest version at https://github.com/kreshuklab/bioimg_rs/releases",
                AppStateSavedData::highest_supported_version(),
            )),
            Err(err) => return Err(format!("Could not load project file at {}: {err}", project_file.to_string_lossy())),
            Ok(proj_data) => proj_data,
        };
        match proj_data{
            AppStateSavedData::Version1(ver1) => self.restore(ver1),
        }
        Ok(())
    }
    fn launch_model_saving(&mut self, zoo_model: ZooModel) {
        let sender = self.notifications_channel.sender().clone();
        let fut = async move {
            let model_name = format!("{}.zip", zoo_model.name);
            let Some(file_handle) = rfd::AsyncFileDialog::new().set_file_name(model_name).save_file().await else {
                return;
            };

            #[cfg(target_arch="wasm32")]
            let message = 'packing_wasm: {
                let mut buffer = Vec::<u8>::new(); //FIXME: check FileSystemWritableFileStream: seek() 
                let cursor = std::io::Cursor::new(&mut buffer);
                if let Err(err) = zoo_model.pack_into(cursor) {
                    let msg = TaskResult::err_message(format!("Error saving model: {err:?}"));
                    break 'packing_wasm msg;
                };
                match file_handle.write(&buffer).await {
                    Ok(()) => TaskResult::ok_message("Model exported successfully"),
                    Err(err) => TaskResult::err_message(format!("{:?}", err)),
                }
            };

            #[cfg(not(target_arch="wasm32"))]
            let message = 'packing: {
                use std::borrow::Cow;

                let file_name = file_handle.file_name();
                if !file_name.ends_with(".zip"){
                    let msg = TaskResult::err_message(format!("Model extension must be '.zip'. Provided '{file_name}'"));
                    sender.send(msg).unwrap();
                    return
                }
                let notification_message = format!("Packing into {file_name}...");
                sender.send(TaskResult::ok_message(notification_message)).unwrap();

                let temp_path = {
                    let current_extension = file_handle.path().extension().map(|s| s.to_string_lossy()).unwrap_or(Cow::Borrowed(""));
                    let temp_extension = format!("{current_extension}.partial");
                    let mut temp_path = file_handle.path().to_owned();
                    temp_path.set_extension(temp_extension);
                    temp_path
                };

                let file = match std::fs::File::create(&temp_path){
                    Ok(file) => file,
                    Err(err) => {
                        break 'packing TaskResult::err_message(format!("Could not create zip file: {err}"));
                    }
                };

                if let Err(err) = zoo_model.pack_into(file){
                    break 'packing TaskResult::err_message(format!("Error saving model: {err}"));
                }
                if let Err(err) = std::fs::rename(&temp_path, file_handle.path()) {
                    if let Err(rm_err) = std::fs::remove_file(&temp_path){
                        let msg = format!("Could not delete temp file {}: {rm_err}", temp_path.to_string_lossy());
                        sender.send(TaskResult::err_message(msg)).unwrap();
                    }
                    break 'packing TaskResult::err_message(format!("Error saving model: {err}"))
                }
                TaskResult::ok_message(format!("Model saved to {file_name}"))
            };

            sender.send(message).unwrap();
        };

        #[cfg(target_arch="wasm32")]
        wasm_bindgen_futures::spawn_local(fut);
        #[cfg(not(target_arch="wasm32"))]
        std::thread::spawn(move || smol::block_on(fut));
    }

    fn load_partial_model(archive: &SharedZipArchive) -> Result<AppStateFromPartial>{
        let model_rdf_bytes: Vec<u8> = 'model_rdf: {
            for file_name in ["rdf.yaml", "bioimageio.yaml"]{
                match archive.read_full_entry(file_name) {
                    Ok(bytes) => break 'model_rdf bytes,
                    Err(zip_err) => match zip_err{
                        zip::result::ZipError::FileNotFound => continue,
                        err => return Err(GuiError::new(format!("Could not read rdf file: {err}")))
                    }
                };
            }
            return Err(GuiError::new("Could not find rdf file inside archive"))
        };
        let yaml_deserializer = serde_yaml::Deserializer::from_slice(&model_rdf_bytes);
        let partial: PartialModelRdfV0_5 = ::serde_path_to_error::deserialize(yaml_deserializer)?;
        let mut warnings = String::with_capacity(16 * 1024);
        let state = AppState1SavedData::from_partial(&archive, partial, &mut warnings); //FIXME: retrieve errors and notify
        warnings += indoc!("
            PLEASE BE AWARE THAT RECOVERING AND THEN RE-EXPORTING A MODEL MIGHT PRODUCE A NEW, VALID MODEL THAT DOES NOT
            BEHAVE LIKE THE ORIGINAL\n"
        );
        Ok(AppStateFromPartial { state, warnings})
    }
}


impl eframe::App for AppState1 {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch="wasm32"))]
                ui.menu_button("Zoo", |ui|{ ui.add_enabled_ui(false, |ui|{
                    self.zoo_login_widget.draw_and_parse(ui, egui::Id::from("zoo login"));

                    let upload_button = egui::Button::new("⬆ Upload Model");
                    let Ok(user_token) = self.zoo_login_widget.state() else {
                        ui.add_enabled_ui(false, |ui|{
                            ui.add(upload_button).on_disabled_hover_text("Please login first");
                        });
                        return;
                    };
                    let Some(packing_task) = self.zoo_model_creation_task.take() else {
                        if !ui.add(upload_button).clicked(){
                            return;
                        }
                        let model = match self.create_model(){
                            Ok(model) => model,
                            Err(err) => {
                                self.notifications_widget.push(Notification::error(err.to_string(), None));
                                return;
                            }
                        };
                        let user_token = user_token.as_ref().clone();
                        let sender = self.notifications_channel.sender().clone();
                        let on_progress = move |msg: String|{
                            sender.send(TaskResult::Notification(Ok(msg))).unwrap(); //FIXME: is there anything sensible to do if this fails?
                        };
                        self.zoo_model_creation_task = Some(
                            std::thread::spawn(|| upload_model(user_token, model, on_progress))
                        );
                        return
                    };
                    if !packing_task.is_finished() {
                        ui.add_enabled_ui(false, |ui|{
                            ui.add(upload_button).on_disabled_hover_text("Uploading model...");
                        });
                        self.zoo_model_creation_task = Some(packing_task);
                        return;
                    }
                    match packing_task.join().unwrap(){
                        Ok(nickname) => self.notifications_widget.push(
                            Notification::info(format!("Model successfully uploaded: {nickname}"), None)
                        ),
                        Err(upload_err) => self.notifications_widget.push(
                            Notification::error(format!("Could not upload model: {upload_err}"), None)
                        ),
                    };
                })});
                ui.menu_button("File", |ui| {
                    if ui.button("📦⤴ Import Model")
                        .on_hover_text("Import a .zip model file, like the ones you'd get from bioimage.io")
                        .clicked()
                    {
                        ui.close_menu();
                        let sender = self.notifications_channel.sender().clone();

                        #[cfg(target_arch="wasm32")]
                        wasm_bindgen_futures::spawn_local(async move {
                            use bioimg_runtime::zip_archive_ext::SharedZipArchive;

                            if let Some(handle) = rfd::AsyncFileDialog::new().add_filter("bioimage model", &["zip"],).pick_file().await {
                                let contents = handle.read().await;
                                let shared_archive = SharedZipArchive::from_raw_data(contents, handle.file_name());
                                let message = match rt::zoo_model::ZooModel::try_load_archive(shared_archive){
                                    Err(err) => TaskResult::Notification(Err(format!("Could not import model: {err}"))),
                                    Ok(zoo_model) => TaskResult::ModelImport(Box::new(zoo_model)),
                                };
                                sender.send(message).unwrap();
                            }
                        });

                        #[cfg(not(target_arch="wasm32"))]
                        if let Some(model_path) = rfd::FileDialog::new().add_filter("bioimage model", &["zip"],).pick_file() {
                            let model_path_str = model_path.to_string_lossy();
                            let message = match rt::zoo_model::ZooModel::try_load(&model_path){
                                Err(err) => TaskResult::Notification(Err(format!("Could not import model {model_path_str}: {err}"))),
                                Ok(zoo_model) => TaskResult::ModelImport(Box::new(zoo_model)),
                            };
                            sender.send(message).unwrap();
                        }
                    }
                    if ui.button("♻📦⤴ Recover Model")
                        .on_hover_text(
                            "Import data from a model .zip archive that is potentially broken or incompatible with this application"
                        )
                        .clicked()
                    {
                        ui.close_menu();
                        let sender = self.notifications_channel.sender().clone();
                        let fut = async move {
                            // On web, file picker is always async, so we make a future.
                            // Also, we don't want to pass in an entire Arc<Mutex<App>> to the future,
                            // so it gets a sender: Sender<TaskResult> instead to report its result back.
                            let Some(handle) = rfd::AsyncFileDialog::new().add_filter("bioimage model", &["zip"]).pick_file().await else {
                                return
                            };
                            // #[cfg(target_arch="wasm32")]
                            let archive = SharedZipArchive::from_raw_data(handle.read().await, handle.file_name());
                            // #[cfg(not(target_arch="wasm32"))]
                            // let archive = match SharedZipArchive::open(handle.path()){
                            //     Ok(archive) => archive,
                            //     Err(e) => {
                            //         let err = Err(format!("Could not open {}: {e}", handle.path().to_string_lossy()));
                            //         _ = sender.send(TaskResult::Notification(err));
                            //         return
                            //     }
                            // };
                            let message = match Self::load_partial_model(&archive) {
                                Err(err) => TaskResult::Notification(Err(format!("Could not recover model: {err}"))),
                                Ok(state_from_partial) => TaskResult::PartialModelLoad(state_from_partial),
                            };
                            sender.send(message).unwrap();
                        };
                        #[cfg(target_arch="wasm32")]
                        wasm_bindgen_futures::spawn_local(fut);
                        #[cfg(not(target_arch="wasm32"))]
                        std::thread::spawn(move || smol::block_on(fut));
                    }
                    #[cfg(not(target_arch="wasm32"))]
                    if ui.button("🗊⤵ Save Draft ")
                        .on_hover_text("Save your current work as-is, even with unresolved errors")
                        .clicked()
                    { 'save_project: {
                        ui.close_menu();
                        let Some(path) = rfd::FileDialog::new().set_file_name("MyDraft.bmb").save_file() else {
                            break 'save_project;
                        };
                        let result = self.save_project(&path);
                        self.notifications_widget.push(result.into());
                    }}
                    #[cfg(not(target_arch="wasm32"))]
                    if ui.button("🗊⤴ Load Draft")
                        .on_hover_text("Load a previously saved, potentially unfinished session")
                        .clicked()
                    { 'load_project: {
                        ui.close_menu();
                        let Some(path) = rfd::FileDialog::new().add_filter("bioimage model builder", &["bmb"]).pick_file() else {
                            break 'load_project;
                        };
                        if let Err(err) = self.load_project(&path){
                            self.notifications_widget.push(Notification::error(err, None));
                        }
                    }}
                });
                ui.menu_button("View", |ui|{
                    egui::widgets::global_theme_preference_buttons(ui);
                });
                ui.menu_button("About", |ui|{
                    ui.label(format!("bioimage.io model builder version {}", env!("CARGO_PKG_VERSION")))
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            while let Ok(msg) = self.notifications_channel.receiver().try_recv(){
                match msg{
                    TaskResult::Notification(msg) => self.notifications_widget.push(msg.into()),
                    TaskResult::ModelImport(model) => self.set_value(*model),
                    TaskResult::PartialModelLoad(AppStateFromPartial{state, warnings}) => {
                        self.restore(state);
                        self.notifications_widget.push(Notification::warning(warnings, None));
                    }
                }
            }
            if let Some(error_rect) = self.notifications_widget.draw(ui, egui::Id::from("messages_widget")){
                ui.scroll_to_rect(error_rect, None);
            }

            ui.style_mut().spacing.item_spacing = egui::Vec2 { x: 10.0, y: 10.0 };
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Model Metadata");
                ui.separator();

                ui.horizontal_top(|ui| {
                    ui.strong("Name: ").on_hover_text(
                        "A human-friendly name of the resource description. \
                        May only contains letters, digits, underscore, minus, parentheses and spaces."
                    );
                    self.staging_name.draw_and_parse(ui, egui::Id::from("Name"));
                    let _name_result = self.staging_name.state();
                });

                ui.horizontal_top(|ui| {
                    ui.strong("Description: ").on_hover_text("A brief description of the model.");
                    self.staging_description.draw_and_parse(ui, egui::Id::from("Name"));
                    let _description_result = self.staging_description.state();
                });

                ui.horizontal_top(|ui| {
                    ui.strong("Cover Images: ").on_hover_text(
                        "Images to be shown to users on the model zoo, preferrably showing what the input \
                        and output look like."
                    );
                    let covers_base_id = egui::Id::from("cover images");
                    let vec_widget = VecWidget{
                        items: &mut self.cover_images,
                        item_label: "Cover Image",
                        min_items: 1,
                        show_reorder_buttons: true,
                        new_item: Some(SpecialImageWidget::default),
                        item_renderer: VecItemRender::HeaderAndBody{
                            render_header: |widget: &mut SpecialImageWidget<_>, idx, ui|{
                                ui.horizontal(|ui|{
                                    ui.weak(format!("Cover image #{idx}"));
                                    ui.add_space(3.0);
                                    widget.summarize(ui, covers_base_id.with(idx));
                                });
                            },
                            render_body: |widg: &mut SpecialImageWidget<_>, idx, ui|{
                                widg.draw_and_parse(ui, covers_base_id.with(("body".as_ptr(), idx)));
                            },
                            collapsible_id_source: Some(covers_base_id),
                            marker: Default::default(),
                        }
                    };
                    ui.add(vec_widget);
                });

                ui.horizontal_top(|ui| {
                    ui.strong("Model Id: ").on_hover_text(
                        "A model zoo id of the form <adjective>-<animal>, like 'affable-shark'.\
                        If you're creating a model from scratch, leave this empty and an id will be generated \
                        for you when you upload your model to the zoo."
                    );
                    self.model_id_widget.draw_and_parse(ui, egui::Id::from("Model Id"));
                    // let cover_img_results = self.cover_images.state();
                });

                ui.horizontal_top(|ui| {
                    let authors_base_id = egui::Id::from("authors");
                    ui.strong("Authors: ").on_hover_text(
                        "The authors are the creators of this resource description and the primary points of contact."
                    );
                    let vec_widget = VecWidget{
                        items: &mut self.staging_authors,
                        item_label: "Author",
                        min_items: 1,
                        show_reorder_buttons: true,
                        new_item: Some(AuthorWidget::default),
                        item_renderer: VecItemRender::HeaderAndBody{
                            render_header: |widg: &mut AuthorWidget, idx, ui|{
                                widg.summarize(ui, authors_base_id.with(("header".as_ptr(), idx)));
                            },
                            render_body: |widg: &mut AuthorWidget, idx, ui|{
                                widg.draw_and_parse(ui, authors_base_id.with(("body".as_ptr(), idx)));
                            },
                            collapsible_id_source: Some(authors_base_id),
                            marker: Default::default(),
                        }
                    };
                    ui.add(vec_widget);
                });

                ui.horizontal_top(|ui| {
                    let attachments_base_id = egui::Id::from("attachments");
                    ui.strong("Attachments: ").on_hover_text(
                        "Any other files that are relevant to your model can be listed as 'attachments'"
                    );
                    let vec_widget = VecWidget{
                        items: &mut self.attachments_widget,
                        min_items: 0,
                        item_label: "Attachment",
                        show_reorder_buttons: true,
                        new_item: Some(AttachmentsWidget::default),
                        item_renderer: VecItemRender::HeaderAndBody{
                            render_header: |widg: &mut AttachmentsWidget, idx, ui|{
                                widg.summarize(ui, attachments_base_id.with(("header".as_ptr(), idx)));
                            },
                            render_body: |widg: &mut AttachmentsWidget, idx, ui|{
                                widg.draw_and_parse(ui, attachments_base_id.with(("body".as_ptr(), idx)));
                            },
                            collapsible_id_source: Some(attachments_base_id),
                            marker: Default::default(),
                        }
                    };
                    ui.add(vec_widget);
                });

                ui.horizontal_top(|ui| {
                    let cite_base_id = egui::Id::from("cite");
                    ui.strong("Cite: ").on_hover_text("How this model should be cited in other publications.");

                    let vec_widget = VecWidget{
                        items: &mut self.staging_citations,
                        min_items: 1,
                        item_label: "Citation Entry",
                        show_reorder_buttons: true,
                        new_item: Some(CiteEntryWidget::default),
                        item_renderer: VecItemRender::HeaderAndBody{
                            render_header: |widg: &mut CiteEntryWidget, idx, ui|{
                                widg.summarize(ui, cite_base_id.with(("header".as_ptr(), idx)));
                            },
                            render_body: |widg: &mut CiteEntryWidget, idx, ui|{
                                widg.draw_and_parse(ui, cite_base_id.with(("body".as_ptr(), idx)));
                            },
                            collapsible_id_source: Some(cite_base_id),
                            marker: Default::default(),
                        }
                    };
                    ui.add(vec_widget);
                });

                ui.horizontal_top(|ui| {
                    ui.weak("Custom configs: ").on_hover_text(
                        "A JSON value representing any extra, 'proprietary' parameters your model might need during runtime. \
                        This field is still available for legacy reasons and its use is strongly discouraged"
                    );
                    self.custom_config_widget.draw_and_parse(ui, egui::Id::from("Custom configs"));
                    // let citation_results = self.staging_citations.state();
                });

                ui.horizontal_top(|ui| {
                    ui.strong("Git Repo: ").on_hover_text(
                        "A URL to the git repository with the source code that produced this model"
                    );
                    self.staging_git_repo.draw_and_parse(ui, egui::Id::from("Git Repo"));
                    // let git_repo_result = self.staging_git_repo.state();
                });

                ui.horizontal_top(|ui| {
                    ui.strong("Icon: ").on_hover_text(indoc!("
                        An icon for quick identification on bioimage.io.
                        This can either be an emoji or a small square image."
                    ));
                    self.icon_widget.draw_and_parse(ui, egui::Id::from("Icon"));
                });

                ui.horizontal_top(|ui| {
                    ui.strong("Model Zoo Links: ").on_hover_text("IDs of other bioimage.io resources");
                    group_frame(ui, |ui| {
                        self.links_widget.draw_and_parse(ui, egui::Id::from("Model Zoo Links"));
                    });
                });

                ui.horizontal_top(|ui| {
                    let maintainers_base_id = egui::Id::from("maintainers");
                    ui.strong("Maintainers: ").on_hover_text(
                        "Maintainers of this resource. If not specified, 'authors' are considered maintainers \
                        and at least one of them must specify their `github_user` name."
                    );

                    let vec_widget = VecWidget{
                        items: &mut self.staging_maintainers,
                        min_items: 0,
                        item_label: "Maintainer",
                        show_reorder_buttons: true,
                        new_item: Some(MaintainerWidget::default),
                        item_renderer: VecItemRender::HeaderAndBody{
                            render_header: |widg: &mut MaintainerWidget, idx, ui|{
                                widg.summarize(ui, maintainers_base_id.with(("header".as_ptr(), idx)));
                            },
                            render_body: |widg: &mut MaintainerWidget, idx, ui|{
                                widg.draw_and_parse(ui, maintainers_base_id.with(("body".as_ptr(), idx)));
                            },
                            collapsible_id_source: Some(maintainers_base_id),
                            marker: Default::default(),
                        }
                    };
                    ui.add(vec_widget);
                });

                ui.horizontal_top(|ui| {
                    ui.strong("Tags: ").on_hover_text("Tags to help search and classifying your model in the model zoo");
                    self.staging_tags.draw_and_parse(ui, egui::Id::from("Tags"));
                });

                ui.horizontal_top(|ui| {
                    ui.strong("Resource Version: ").on_hover_ui(|ui|{
                        ui.horizontal(|ui|{
                            ui.label("The version of this model, following");
                            ui.hyperlink_to("SermVer 2.0", "https://semver.org/#semantic-versioning-200");
                        });

                        ui.label(indoc!("
                            If you upload an updated version of this model to the zoo, you should bump this version \
                            to differentiate it from the previous uploads"
                        ));
                    });
                    self.staging_version.draw_and_parse(ui, egui::Id::from("Version"));
                });

                ui.horizontal_top(|ui| {
                    ui.strong("Resource Version comment: ").on_hover_text(indoc!(
                        "
                        A comment about what changed in this version of this model.
                        Here you can explain why you bumped the resource version"
                    ));

                    self.staging_version_comment
                        .draw_and_parse(ui, egui::Id::from("Version Comment"));
                });

                ui.horizontal(|ui| {
                    ui.strong("License: ").on_hover_text("A standard software licence, specifying how this model can be used and for what purposes.");
                    self.staging_license.draw_and_parse(ui, egui::Id::from("License"));
                });
                ui.add_space(20.0);


                ui.heading("Documentation (markdown): ").on_hover_text(
                    "All model documentation should be written here. This field accepts Markdown syntax"
                );
                ui.separator();
                self.staging_documentation.draw_and_parse(ui, egui::Id::from("Documentation"));
                ui.add_space(20.0);


                ui.heading("Model Interface");
                ui.separator();
                egui::ScrollArea::horizontal().show(ui, |ui|{
                    self.pipeline_widget.draw(
                        ui,
                        egui::Id::from("pipeline"),
                        &mut self.model_interface_widget,
                        &mut self.weights_widget,
                    );
                });
                ui.add_space(20.0);

                ui.separator();

                let save_button_clicked = ui.button("Export Model ⤵📦")
                    .on_hover_text("Exports this model to a .zip file, ready to be used or uploaded to the Model Zoo")
                    .clicked();

                if save_button_clicked {
                    match self.create_model(){
                        Ok(zoo_model) => self.launch_model_saving(zoo_model),
                        Err(err) => self.notifications_widget.push(
                            Notification::error(format!("Could not create zoo model: {err}"), None)
                        ),
                    }
                }
            });
        });

        let close_requested = ctx.input(|i| i.viewport().close_requested());
        self.exiting_status = match self.exiting_status {
            ExitingStatus::NotExiting => {
                if close_requested {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    ExitingStatus::Confirming
                } else {
                    ExitingStatus::NotExiting
                }
            },
            ExitingStatus::Confirming => {
                if close_requested {
                    ExitingStatus::Exiting
                } else {
                    ExitingStatus::Confirming
                }
            },
            ExitingStatus::Exiting => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                ExitingStatus::Exiting
            },
        };

        #[cfg(not(target_arch="wasm32"))]
        if matches!(self.exiting_status, ExitingStatus::Confirming) {
            egui::Modal::new(egui::Id::from("confirmation dialog"))
                .show(ctx, |ui| {
                    ui.label("Save draft before quitting?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes 💾").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)){ 'save_draft: {
                            let Some(path) = rfd::FileDialog::new().set_file_name("MyDraft.bmb").save_file() else {
                                self.exiting_status = ExitingStatus::NotExiting;
                                break 'save_draft;
                            };
                            let result = self.save_project(&path);
                            if result.is_ok(){
                                self.exiting_status = ExitingStatus::Exiting;
                            }
                            self.notifications_widget.push(result.into());
                        }}
                        if ui.button("No 🗑").clicked() {
                            self.exiting_status = ExitingStatus::Exiting;
                        }
                        if ui.button("Cancel 🗙").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.exiting_status = ExitingStatus::NotExiting;
                        }
                    });
                });
        }
    }
}
