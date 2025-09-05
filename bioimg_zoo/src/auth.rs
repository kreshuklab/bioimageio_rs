//! !!! This crate is deprecated. It implements the API for the old zoo !!!
//!
//! Communication logic with the model zoo is encoded as a type-state-machine,
//! where each type expresses a particular state of communication. Users of
//! the library are expected to perform the web requests whose specifications
//! are obtained from those type-states and feed in the results back into the
//! type-states, which will then inform the user if the request succeeded and
//! produce the next state.
//!
//! The reasoning behind is is to unify the communication logic in a way that
//! is not reliant on sync or async styles of requesting. You can check example
//! usage of the library in the tests cases of this crate.

use std::num::ParseIntError;

use bioimg_spec::rdf::HttpUrl;

use crate::{read_json_response, BadResponse, APPLICATION_JSON};

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
pub struct Seconds(pub u32);


#[derive(thiserror::Error, Debug)]
pub enum TokenParsingError{
    #[error("Token is garbled")]
    GarbledToken{raw_token: String},
    #[error("Could not deserialize claims from b64")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("Could not deserialize claims from json: {0}")]
    JsonDeserialize(#[from] serde_json::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum GithubUserParsingError{
    #[error("Bad github user name: {raw}")]
    Garbled{raw: String},
    #[error("Could not parse user id: {0}")]
    UserIdParsing(#[from] ParseIntError),
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(try_from="String")]
pub struct GithubUser{
    id: u64,
}

impl GithubUser{
    pub fn to_hypha_workspace_name(&self) -> String{
        format!("ws-user-github|{}", self.id)
    }
}

impl TryFrom<String> for GithubUser{
    type Error = GithubUserParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut parts = value.split('|');
        let Some(github_marker) = parts.next() else {
            return Err(GithubUserParsingError::Garbled { raw: value });
        };
        if github_marker != "github"{
            return Err(GithubUserParsingError::Garbled { raw: value });
        }
        let Some(user_id_raw) = parts.next() else {
            return Err(GithubUserParsingError::Garbled { raw: value });
        };
        use std::str::FromStr;
        let user_id = u64::from_str(user_id_raw)?;

        if parts.next().is_some(){
            return Err(GithubUserParsingError::Garbled { raw: value });
        }
        
        Ok(Self{id: user_id})
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Claims{
    sub: GithubUser,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct UserToken{
    raw: String,
    claims: Claims,
}

impl UserToken{
    pub fn to_hypha_workspace_name(&self) -> String{
        self.claims.sub.to_hypha_workspace_name()
    }
}

impl TryFrom<String> for UserToken{
    type Error = TokenParsingError;
    fn try_from(raw_token: String) -> Result<Self, Self::Error> {
        use base64::prelude::*;
        
        let mut parts = raw_token.split(".");
        let Some(_b64_header) = parts.next() else {
            return Err(TokenParsingError::GarbledToken { raw_token })
        };
        let Some(b64_claims_json) = parts.next() else {
            return Err(TokenParsingError::GarbledToken { raw_token })
        };
        let Some(_b64_signature) = parts.next() else {
            return Err(TokenParsingError::GarbledToken { raw_token })
        };
        if parts.next().is_some(){
            return Err(TokenParsingError::GarbledToken { raw_token })
        }

        let claims_json = BASE64_STANDARD_NO_PAD.decode(b64_claims_json.as_bytes())?;
        let claims: Claims = serde_json::from_slice(&claims_json)?;
        
        Ok(Self{raw: raw_token, claims})
    }
}

impl UserToken{
    pub fn as_header(&self) -> (http::HeaderName, String){
        (http::header::AUTHORIZATION, format!("Bearer {}", self.raw))
    }
    pub fn as_str(&self) -> &str{
        &self.raw
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LoginStartResponseMessage{
    pub login_url: HttpUrl,
    pub key: String,
    pub report_url: HttpUrl,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HyphaLoginCheckMessage{
    pub key: String,
    pub timeout: Seconds,
}

type ResponseBytes = http::Response<Vec<u8>>;

#[derive(Debug, Default, derive_more::AsRef)]
pub struct AuthStart(http::Request<[u8; 2]>);

impl AuthStart{
    pub fn new() -> Self{
        Self(
            http::Request::builder()
                .method("POST")
                .uri("https://hypha.aicell.io/public/services/hypha-login/start")
                .header(http::header::CONTENT_TYPE, APPLICATION_JSON)
                .body([b'{', b'}'])
                .unwrap()
        )
    }

    pub fn try_advance(self, login_response: &ResponseBytes) -> Result<AuthNeedsBrowserLogin, (Self, BadResponse)>{
        let login_data: LoginStartResponseMessage = match read_json_response(login_response){
            Ok(data) => data,
            Err(err) => return Err((self, err))
        };
        Ok(AuthNeedsBrowserLogin { login_url: login_data.login_url, key: login_data.key })
    }
}

pub struct AuthNeedsBrowserLogin{
    key: String,
    login_url: HttpUrl,
}

impl AuthNeedsBrowserLogin{
    pub fn login_url(&self) -> &HttpUrl{
        &self.login_url
    }
    pub fn advance(self, timeout: Seconds) -> (HttpUrl, AuthInProgress){
        return (
            self.login_url,
            AuthInProgress::new(self.key, timeout),
        )
    }
}

#[derive(Debug, derive_more::AsRef)]
pub struct AuthInProgress(http::Request<Vec<u8>>);

impl AuthInProgress{
    fn new(key: String, token_fetch_timeout: Seconds) -> Self{
        Self(
            http::Request::builder()
                .method(http::Method::POST)
                .uri("https://hypha.aicell.io/public/services/hypha-login/check")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(
                    serde_json::to_vec(
                        &HyphaLoginCheckMessage{key: key.clone(), timeout: token_fetch_timeout}
                    ).unwrap())
                .unwrap()
        )
    }

    pub fn try_advance(self, token_response: &http::Response<Vec<u8>>) -> Result<UserToken, (Self, BadResponse)>{
        read_json_response(token_response).map_err(|err| (self, err))
    }
}


