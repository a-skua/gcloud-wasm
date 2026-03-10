wit_bindgen::generate!({
    world: "token-provider",
    path: "wit",
});

use exports::gcloud::auth::token_source::{Error, Guest, Token};

struct Component;

export!(Component);

impl Guest for Component {
    fn get_token(scopes: Vec<String>) -> Result<Token, Error> {
        wstd::runtime::block_on(get_access_token(&scopes))
    }
}

async fn get_access_token(_scopes: &[String]) -> Result<Token, Error> {
    let adc = read_adc().map_err(Error::InvalidCredentials)?;

    match adc {
        Adc::AuthorizedUser {
            client_id,
            client_secret,
            refresh_token,
        } => refresh_access_token(&client_id, &client_secret, &refresh_token).await,
    }
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum Adc {
    #[serde(rename = "authorized_user")]
    AuthorizedUser {
        client_id: String,
        client_secret: String,
        refresh_token: String,
    },
}

fn read_adc() -> Result<Adc, String> {
    let adc_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME is not set");
        format!("{home}/.config/gcloud/application_default_credentials.json")
    });

    let json = std::fs::read_to_string(&adc_path)
        .map_err(|e| format!("failed to read ADC file {adc_path}: {e}"))?;

    serde_json::from_str(&json).map_err(|e| format!("failed to parse ADC file: {e}"))
}

async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<Token, Error> {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    use wstd::http::{Body, Client, Request};

    let form_body = format!(
        "grant_type=refresh_token&client_id={}&client_secret={}&refresh_token={}",
        utf8_percent_encode(client_id, NON_ALPHANUMERIC),
        utf8_percent_encode(client_secret, NON_ALPHANUMERIC),
        utf8_percent_encode(refresh_token, NON_ALPHANUMERIC),
    );

    let request = Request::post("https://oauth2.googleapis.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_body))
        .map_err(|e| Error::TokenFetchFailed(format!("request build error: {e}")))?;

    let response = Client::new()
        .send(request)
        .await
        .map_err(|e| Error::TokenFetchFailed(format!("HTTP error: {e}")))?;

    let mut body = response.into_body();
    let contents = body
        .contents()
        .await
        .map_err(|e| Error::TokenFetchFailed(format!("body read error: {e}")))?;

    #[derive(serde::Deserialize)]
    struct TokenResponse {
        access_token: String,
        token_type: String,
        expires_in: u64,
    }

    let res: TokenResponse = serde_json::from_slice(contents)
        .map_err(|e| Error::TokenFetchFailed(format!("JSON parse error: {e}")))?;

    Ok(Token {
        access_token: res.access_token,
        token_type: res.token_type,
        expires_in: res.expires_in,
    })
}
