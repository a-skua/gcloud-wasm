use crate::{Error, Token};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{SignatureEncoding, Signer};
use sha2::Sha256;
use wstd::http::{Body, Client, Request};

pub(crate) async fn fetch_token(
    client_email: &str,
    private_key: &str,
    private_key_id: &str,
    token_uri: &str,
    scopes: &[String],
) -> Result<Token, Error> {
    let jwt = build_jwt(client_email, private_key, private_key_id, token_uri, scopes)
        .map_err(Error::InvalidCredentials)?;

    let form_body = format!(
        "grant_type={}&assertion={}",
        percent_encoding::utf8_percent_encode(
            "urn:ietf:params:oauth:grant-type:jwt-bearer",
            percent_encoding::NON_ALPHANUMERIC,
        ),
        percent_encoding::utf8_percent_encode(&jwt, percent_encoding::NON_ALPHANUMERIC),
    );

    let request = Request::post(token_uri)
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

    let res: TokenResponse = serde_json::from_slice(contents)
        .map_err(|e| Error::TokenFetchFailed(format!("JSON parse error: {e}")))?;

    Ok(Token {
        access_token: res.access_token,
        token_type: res.token_type,
        expires_in: res.expires_in,
    })
}

fn build_jwt(
    client_email: &str,
    private_key_pem: &str,
    private_key_id: &str,
    audience: &str,
    scopes: &[String],
) -> Result<String, String> {
    let now = unix_timestamp();

    let header = serde_json::json!({
        "alg": "RS256",
        "typ": "JWT",
        "kid": private_key_id,
    });

    let claims = serde_json::json!({
        "iss": client_email,
        "scope": scopes.join(" "),
        "aud": audience,
        "iat": now,
        "exp": now + 3600,
    });

    let header_b64 = URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());
    let claims_b64 = URL_SAFE_NO_PAD.encode(claims.to_string().as_bytes());
    let signing_input = format!("{header_b64}.{claims_b64}");

    let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(private_key_pem)
        .map_err(|e| format!("failed to parse private key: {e}"))?;
    let signing_key = SigningKey::<Sha256>::new(private_key);
    let signature = signing_key.sign(signing_input.as_bytes());
    let signature_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

    Ok(format!("{signing_input}.{signature_b64}"))
}

fn unix_timestamp() -> u64 {
    wasip2::clocks::wall_clock::now().seconds
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}
