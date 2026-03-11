wit_bindgen::generate!({
    world: "secretmanager-provider",
    path: "wit",
    generate_all,
});

use exports::gcloud::secretmanager::secrets::{Error, Guest, SecretPayload};
use gcloud::auth::token_source::get_token;

struct Component;

export!(Component);

impl Guest for Component {
    fn access(name: String) -> Result<SecretPayload, Error> {
        wstd::runtime::block_on(access(&name))
    }
}

async fn access(name: &str) -> Result<SecretPayload, Error> {
    let scopes = vec!["https://www.googleapis.com/auth/cloud-platform".to_string()];
    let token = get_token(&scopes).map_err(Error::Auth)?;

    let url = format!("https://secretmanager.googleapis.com/v1/{name}:access");

    use wstd::http::{Client, Request};

    let request = Request::get(&url)
        .header(
            "Authorization",
            &format!("{} {}", token.token_type, token.access_token),
        )
        .body(wstd::http::Body::empty())
        .map_err(|e| Error::RequestFailed(format!("request build error: {e}")))?;

    let response = Client::new()
        .send(request)
        .await
        .map_err(|e| Error::RequestFailed(format!("HTTP error: {e}")))?;

    let mut body = response.into_body();
    let contents = body
        .contents()
        .await
        .map_err(|e| Error::RequestFailed(format!("body read error: {e}")))?;

    #[derive(serde::Deserialize)]
    struct AccessResponse {
        payload: PayloadData,
    }

    #[derive(serde::Deserialize)]
    struct PayloadData {
        data: String,
    }

    let res: AccessResponse = serde_json::from_slice(contents)
        .map_err(|e| Error::RequestFailed(format!("JSON parse error: {e}")))?;

    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD
        .decode(&res.payload.data)
        .map_err(|e| Error::RequestFailed(format!("base64 decode error: {e}")))?;

    Ok(SecretPayload { data })
}
