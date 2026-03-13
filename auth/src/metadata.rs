use crate::{Error, Token};
use wstd::http::{Client, Request};

pub(crate) async fn fetch_token(scopes: &[String]) -> Result<Token, Error> {
    let host = std::env::var("GCE_METADATA_HOST")
        .unwrap_or_else(|_| "metadata.google.internal".to_string());

    let url = if scopes.is_empty() {
        format!("http://{host}/computeMetadata/v1/instance/service-accounts/default/token")
    } else {
        let joined = scopes.join(",");
        let scopes_param = percent_encoding::utf8_percent_encode(&joined, QUERY_ENCODE_SET);
        format!(
            "http://{host}/computeMetadata/v1/instance/service-accounts/default/token?scopes={scopes_param}"
        )
    };

    let request = Request::get(&url)
        .header("Metadata-Flavor", "Google")
        .body(wstd::http::Body::empty())
        .map_err(|e| Error::TokenFetchFailed(format!("request build error: {e}")))?;

    let response = Client::new()
        .send(request)
        .await
        .map_err(|e| Error::TokenFetchFailed(format!("metadata server error: {e}")))?;

    let mut body = response.into_body();
    let contents = body
        .contents()
        .await
        .map_err(|e| Error::TokenFetchFailed(format!("body read error: {e}")))?;

    let res: TokenResponse = serde_json::from_slice(contents)
        .map_err(|e| Error::TokenFetchFailed(format!("JSON parse error: {e}")))?;

    Ok(Token {
        access_token: res.access_token,
        token_type: res.token_type,
        expires_in: res.expires_in,
    })
}

/// Characters that need encoding in query parameter values.
const QUERY_ENCODE_SET: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~')
    .remove(b',');

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}
