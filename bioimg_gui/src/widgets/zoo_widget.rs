use std::sync::Arc;

use crate::request::{send_bytes, send_reader};
use crate::result::Result;
use bioimg_runtime::zoo_model::ZooModel;
use bioimg_spec::rdf::HttpUrl;
use bioimg_zoo::auth::{AuthInProgress, AuthStart, Seconds, UserToken};
use bioimg_zoo::collection::{CollectionConfig, CollectionJson, ZooNickname, ZooNicknameGenerator};
use bioimg_zoo::client::ClientMethod;

use crate::result::GuiError;

use super::StatefulWidget;

type BytesResponse = http::Response<Vec<u8>>;

type ReqResult = Result<BytesResponse, ureq::Error>;

enum ZooLoginState{
    Start(AuthStart),
    FetchingLoginUrl{state: AuthStart, request_task: std::thread::JoinHandle<ReqResult>},
    AuthInProgress{
        login_url: HttpUrl,
        state: AuthInProgress,
        opened_web_browser: bool,
        request_task: std::thread::JoinHandle<ReqResult>
    },
    Authenticated(Arc<UserToken>),

    Failed(GuiError)
}

impl Default for ZooLoginState{
    fn default() -> Self {
        Self::Start(Default::default())
    }
}

impl ZooLoginState{
    pub fn fetching_login_url() -> Self{
        let start = AuthStart::new();
        let req: http::Request<_>  = start.as_ref().clone();
        ZooLoginState::FetchingLoginUrl{
            request_task: std::thread::spawn(move || send_bytes(req)),
            state: start,
        }
    }
}

#[derive(Default)]
pub struct ZooLoginWidget{
    state: ZooLoginState,
}

impl ZooLoginWidget{
    pub fn update(&mut self){
        self.state = match std::mem::take(&mut self.state){
            ZooLoginState::FetchingLoginUrl{state, request_task} => 'advancing_start: {
                if !request_task.is_finished(){
                    break 'advancing_start ZooLoginState::FetchingLoginUrl{state, request_task}
                }
                let response = match request_task.join().unwrap(){ //FIXME: report failure to join?
                    Ok(resp) => resp,
                    Err(reason) => break 'advancing_start ZooLoginState::Failed(GuiError::new(reason.to_string()))
                };
                match state.try_advance(&response){
                    Ok(needs_browser_interaction) => {
                        let (login_url, auth_in_progress) = needs_browser_interaction.advance(Seconds(3600));
                        let req: http::Request<_> = auth_in_progress.as_ref().clone();
                        eprintln!("Login here: {}", login_url.to_string());
                        ZooLoginState::AuthInProgress {
                            login_url,
                            state: auth_in_progress,
                            request_task: std::thread::spawn(move ||{
                                send_bytes(req)
                            }),
                            opened_web_browser: false,
                        }
                    },
                    Err((_current_state, reason)) => ZooLoginState::Failed(GuiError::new(reason.to_string()))
                }
            },
            ZooLoginState::AuthInProgress { login_url, state, request_task, opened_web_browser } => 'fetching_token: {
                if ! request_task.is_finished(){
                    break 'fetching_token ZooLoginState::AuthInProgress { login_url, state, request_task, opened_web_browser }
                }
                let response = match request_task.join().unwrap(){ //FIXME: report failure to join?
                    Ok(resp) => resp,
                    Err(reason) => {
                        break 'fetching_token ZooLoginState::Failed(GuiError::new(reason.to_string()))
                    }
                };
                match state.try_advance(&response){
                    Ok(user_token) => ZooLoginState::Authenticated(Arc::new(user_token)),
                    Err((_current_state, reason)) => ZooLoginState::Failed(GuiError::new(reason.to_string())),
                }
            },
            state => state,
        }
    }
}

impl StatefulWidget for ZooLoginWidget{
    type Value<'p> = Result<Arc<UserToken>, GuiError>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        self.update();
        ui.vertical(|ui|{
            let button = egui::Button::new("👤 Login");
            self.state = match std::mem::take(&mut self.state){
                ZooLoginState::Start(state) => {
                    if ui.add(button).clicked(){
                        ZooLoginState::fetching_login_url()
                    }else{
                        ZooLoginState::Start(state)
                    }
                },
                ZooLoginState::Failed(state) => {
                    let clicked = ui.horizontal(|ui|{
                        let clicked = ui.add(button).clicked();
                        ui.label(egui::RichText::new("login failed").color(ui.visuals().error_fg_color))
                            .on_hover_ui(|ui|{
                                ui.label(state.to_string());
                            });
                        clicked
                    }).inner;
                    if clicked{
                        ZooLoginState::fetching_login_url()
                    }else{
                        ZooLoginState::Failed(state)
                    }
                },
                ZooLoginState::FetchingLoginUrl{state, request_task} => {
                    ui.horizontal(|ui|{
                        ui.add_enabled_ui(false, |ui| ui.add(button));
                        ui.weak("Requesting login URL...");
                    });
                    ui.ctx().request_repaint();
                    ZooLoginState::FetchingLoginUrl{state, request_task}
                },
                ZooLoginState::AuthInProgress { login_url, state, request_task, mut opened_web_browser } => {
                    opened_web_browser = opened_web_browser || {
                        ui.ctx().open_url(egui::OpenUrl {
                            url: login_url.to_string(),
                            new_tab: true,
                        });
                        true
                    };
                    ui.horizontal(|ui|{
                        ui.add_enabled_ui(false, |ui| ui.add(button));
                        ui.weak("Please login");
                        ui.hyperlink_to("here", login_url.to_string());
                    });
                    ui.ctx().request_repaint();
                    ZooLoginState::AuthInProgress { login_url, state, request_task, opened_web_browser }
                },
                ZooLoginState::Authenticated(user_token) => 'authenticated: {
                    let restart_login_clicked = ui.horizontal(|ui|{
                        let clicked = ui.add(button).clicked();
                        ui.weak("Logged in");
                        clicked
                    }).inner;
                    if restart_login_clicked{
                        ui.ctx().request_repaint();
                        break 'authenticated ZooLoginState::fetching_login_url();
                    }
                    ZooLoginState::Authenticated(user_token)
                },
            };
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        match &self.state{
            ZooLoginState::Authenticated(token) => Ok(Arc::clone(token)),
            ZooLoginState::Failed(reason) => Err(reason.clone()),
            _ => Err(GuiError::new("Auth not ready yet")), //FIXME:
        }
    }
}

pub fn upload_model(
    user_token: UserToken,
    model: ZooModel,
    on_progress: impl Fn(String),
) -> Result<ZooNickname>{
    let mut file_to_upload = model.pack_into_tmp()?;

    let collection_config: CollectionConfig = {
        let req = CollectionConfig::request();
        let collection_config_resp = send_bytes(req).unwrap();
        CollectionConfig::parse_response(&collection_config_resp).unwrap()
    };
    let collection_json: CollectionJson = {
        let req = CollectionJson::request();
        let collection_json_resp = send_bytes(req).unwrap();
        CollectionJson::parse_response(&collection_json_resp).unwrap()
    };
    let nickname_generator = ZooNicknameGenerator::new(collection_config, collection_json);
    let nickname = (0..50)
        .filter_map(|_| nickname_generator.generate_zoo_nickname())
        .next().unwrap();
    on_progress(format!("Chosen model nickname: {nickname}"));

    let client = bioimg_zoo::client::Client::new(user_token);

    let presigned_url = {
        on_progress(format!("Requesting a storage URL..."));
        let resp_signed_url = send_bytes(
            client.presigned_url_request(&nickname, Seconds(3600), ClientMethod::PutObject)
        )?;
        let url = client.parse_presigned_url_resp(&resp_signed_url)?;
        on_progress(format!("Got storage pressigned URL"));
        eprintln!("==>> And this is the signed url for PUT: {url}. Now lets try putting something in it");
        url
    };

    {
        let put_req = client.write_to_bucket_request(&presigned_url, &mut file_to_upload);
        on_progress(format!("Uploading model '{nickname}' to zoo"));
        let resp = send_reader(put_req)?;
        on_progress(format!("Model '{nickname}' successfully uploaded"));
        let upload_resp_str = String::from_utf8(resp.into_body()).unwrap();
        eprintln!("==>> And here's the response: {upload_resp_str}")
    }

    {
        let resp_signed_url = send_bytes(client.presigned_url_request(&nickname, Seconds(3600), ClientMethod::GetObject)).unwrap();
        let presigned_url = client.parse_presigned_url_resp(&resp_signed_url).unwrap();
        eprintln!("==>> And this is the signed GET url: {presigned_url}");

        eprintln!("Trying to stage it....");
        let req = client.stage_model_request(&nickname, &presigned_url);
        on_progress(format!("Staging model {nickname}"));
        let resp = send_bytes(req)?;
        on_progress(format!("Successfully staged model {nickname}. It will be available on the zoo once it's reviewed"));
        let resp_str = String::from_utf8(resp.into_body()).unwrap();
        eprintln!("==>> And here's the STAGING response: {resp_str}");
    }
    Ok(nickname)
}
