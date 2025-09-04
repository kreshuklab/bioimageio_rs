use colored::Colorize;

use bioimg_zoo::client::{Client, ClientMethod};
use bioimg_zoo::auth::{AuthStart, Seconds};
use bioimg_zoo::collection::{CollectionConfig, CollectionJson, ZooNicknameGenerator};

#[ignore]
#[test]
fn test_model_upload(){
    fn send_bytes<T: AsRef<[u8]>>(req: http::Request<T>) -> Result<http::Response<Vec<u8>>, String>{
        let (http_parts, body) = req.into_parts();
        let reader_req = http::Request::from_parts(http_parts, std::io::Cursor::new(body));
        send_reader(reader_req)
    }

    fn send_reader<R: std::io::Read>(req: http::Request<R>) -> Result<http::Response<Vec<u8>>, String>{
        eprintln!("Requesting {}", req.uri().to_string().yellow());
        let (http_parts, body) = req.into_parts();
        let request: ureq::Request = http_parts.into();
        let resp: http::Response<Vec<u8>> = request.send(body)
            .map_err(|err| {
                eprintln!("Ok, so something went wrong!!!!!!!!!!!!!!!!!!!!!!!");
                let ureq::Error::Status(status, resp) = err else {
                    eprintln!(">>>>>>> iit was transport error");
                    return "Some transport error".to_owned();
                };
                let mut reader = resp.into_reader();
                let mut buf = vec![];
                reader.read_to_end(&mut buf).unwrap();
                let s = String::from_utf8_lossy(&buf);
                eprintln!(">>>>>>> iit was something else {status}: {s}");
                s.to_string()
            })?
            .into();
        if !resp.status().is_success(){
            let payload = String::from_utf8_lossy(resp.body());
            eprintln!("Error!!\n{}", payload.red());
            return Err(format!("Request failed with result {}\n{payload}", resp.status()))
        }
        Ok(resp)
    }

    let start = AuthStart::new();

    let response = send_bytes(start.as_ref().clone()).expect("Could not start login");
    let expecting_browser_interaction = start.try_advance(&response).expect("Could not advance to login in progress");

    let (login_url, auth_in_progress) = expecting_browser_interaction.advance(Seconds(3600));
    eprintln!("Login here: {login_url}");

    let resp = send_bytes(auth_in_progress.as_ref().clone()).expect("Could not fetch token?");
    let user_token = auth_in_progress.try_advance(&resp).unwrap();
    eprintln!("Here's the user token: {user_token:?}");

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

    let client = Client::new(user_token);

    let presigned_url = {
        let resp_signed_url = send_bytes(client.presigned_url_request(&nickname, Seconds(3600), ClientMethod::PutObject)).unwrap();
        let url = client.parse_presigned_url_resp(&resp_signed_url).unwrap();
        eprintln!("==>> And this is the signed url for PUT: {url}. Now lets try putting something in it");
        url
    };

    {
        let put_req = client.write_to_bucket_request(&presigned_url, std::io::Cursor::new(b"This is just a bunch of test bytes"));
        let resp = send_reader(put_req).unwrap();
        let upload_resp_str = String::from_utf8(resp.into_body()).unwrap();
        eprintln!("==>> And here's the response: {upload_resp_str}")
    }

    {
        let resp_signed_url = send_bytes(client.presigned_url_request(&nickname, Seconds(3600), ClientMethod::GetObject)).unwrap();
        let presigned_url = client.parse_presigned_url_resp(&resp_signed_url).unwrap();
        eprintln!("==>> And this is the signed GET url: {presigned_url}");

        eprintln!("Trying to stage it....");
        let req = client.stage_model_request(&nickname, &presigned_url);
        let resp = send_bytes(req).unwrap();
        let resp_str = String::from_utf8(resp.into_body()).unwrap();
        eprintln!("==>> And here's the STAGING response: {resp_str}")
    }

}
