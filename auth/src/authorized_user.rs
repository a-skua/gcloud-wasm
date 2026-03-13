use crate::Token;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use wstd::http::{Body, Client, Request};

pub(crate) async fn fetch_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<Token, crate::Error> {
    let form_body = format!(
        "grant_type=refresh_token&client_id={}&client_secret={}&refresh_token={}",
        utf8_percent_encode(client_id, NON_ALPHANUMERIC),
        utf8_percent_encode(client_secret, NON_ALPHANUMERIC),
        utf8_percent_encode(refresh_token, NON_ALPHANUMERIC),
    );

    let request = Request::post("https://oauth2.googleapis.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_body))
        .map_err(|e| crate::Error::TokenFetchFailed(format!("request build error: {e}")))?;

    let response = Client::new()
        .send(request)
        .await
        .map_err(|e| crate::Error::TokenFetchFailed(format!("HTTP error: {e}")))?;

    let mut body = response.into_body();
    let contents = body
        .contents()
        .await
        .map_err(|e| crate::Error::TokenFetchFailed(format!("body read error: {e}")))?;

    let res: TokenResponse = serde_json::from_slice(contents)
        .map_err(|e| crate::Error::TokenFetchFailed(format!("JSON parse error: {e}")))?;

    Ok(Token {
        access_token: res.access_token,
        token_type: res.token_type,
        expires_in: res.expires_in,
    })
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}
