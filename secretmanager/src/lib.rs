wit_bindgen::generate!({
    world: "secretmanager-provider",
    path: "wit",
    generate_all,
});

use exports::gcloud::secretmanager::secrets::{
    Error, Guest, Replication, Secret, SecretPayload, SecretVersion,
};
use gcloud::auth::token_source::get_token;
use gcloud::secretmanager::types::SecretVersionState;

struct Component;

export!(Component);

const SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";
const BASE_URL: &str = "https://secretmanager.googleapis.com/v1";

fn auth_header() -> Result<String, Error> {
    let scopes = vec![SCOPE.to_string()];
    let token = get_token(&scopes).map_err(Error::Auth)?;
    Ok(format!("{} {}", token.token_type, token.access_token))
}

async fn do_get(url: &str) -> Result<Vec<u8>, Error> {
    use wstd::http::{Client, Request};

    let auth = auth_header()?;
    let request = Request::get(url)
        .header("Authorization", &auth)
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
    Ok(contents.to_vec())
}

async fn do_post(url: &str, json_body: &[u8]) -> Result<Vec<u8>, Error> {
    use wstd::http::{Client, Request};

    let auth = auth_header()?;
    let request = Request::post(url)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .body(wstd::http::Body::from(json_body.to_vec()))
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
    Ok(contents.to_vec())
}

async fn do_delete(url: &str) -> Result<(), Error> {
    use wstd::http::{Client, Request};

    let auth = auth_header()?;
    let request = Request::delete(url)
        .header("Authorization", &auth)
        .body(wstd::http::Body::empty())
        .map_err(|e| Error::RequestFailed(format!("request build error: {e}")))?;

    Client::new()
        .send(request)
        .await
        .map_err(|e| Error::RequestFailed(format!("HTTP error: {e}")))?;

    Ok(())
}

fn parse_json<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(data).map_err(|e| Error::RequestFailed(format!("JSON parse error: {e}")))
}

// --- JSON response types ---

#[derive(serde::Deserialize)]
struct AccessResponse {
    payload: PayloadData,
}

#[derive(serde::Deserialize)]
struct PayloadData {
    data: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecretJson {
    name: String,
    create_time: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecretVersionJson {
    name: String,
    #[serde(default)]
    state: String,
    create_time: Option<String>,
    destroy_time: Option<String>,
}

#[derive(serde::Deserialize)]
struct ListSecretsResponse {
    #[serde(default)]
    secrets: Vec<SecretJson>,
}

#[derive(serde::Deserialize)]
struct ListSecretVersionsResponse {
    #[serde(default)]
    versions: Vec<SecretVersionJson>,
}

impl From<SecretJson> for Secret {
    fn from(s: SecretJson) -> Self {
        Secret {
            name: s.name,
            create_time: s.create_time,
        }
    }
}

fn parse_version_state(s: &str) -> SecretVersionState {
    match s {
        "ENABLED" => SecretVersionState::Enabled,
        "DISABLED" => SecretVersionState::Disabled,
        "DESTROYED" => SecretVersionState::Destroyed,
        _ => SecretVersionState::StateUnspecified,
    }
}

impl From<SecretVersionJson> for SecretVersion {
    fn from(v: SecretVersionJson) -> Self {
        SecretVersion {
            name: v.name,
            state: parse_version_state(&v.state),
            create_time: v.create_time,
            destroy_time: v.destroy_time,
        }
    }
}

// --- Guest implementation ---

impl Guest for Component {
    fn access_secret_version(name: String) -> Result<SecretPayload, Error> {
        wstd::runtime::block_on(access_secret_version(&name))
    }

    fn list_secrets(parent: String) -> Result<Vec<Secret>, Error> {
        wstd::runtime::block_on(list_secrets(&parent))
    }

    fn get_secret(name: String) -> Result<Secret, Error> {
        wstd::runtime::block_on(get_secret(&name))
    }

    fn list_secret_versions(parent: String) -> Result<Vec<SecretVersion>, Error> {
        wstd::runtime::block_on(list_secret_versions(&parent))
    }

    fn add_secret_version(parent: String, payload: SecretPayload) -> Result<SecretVersion, Error> {
        wstd::runtime::block_on(add_secret_version(&parent, payload))
    }

    fn create_secret(
        parent: String,
        secret_id: String,
        _replication: Replication,
    ) -> Result<Secret, Error> {
        wstd::runtime::block_on(create_secret(&parent, &secret_id))
    }

    fn delete_secret(name: String) -> Result<(), Error> {
        wstd::runtime::block_on(delete_secret(&name))
    }
}

// --- Async implementations ---

async fn access_secret_version(name: &str) -> Result<SecretPayload, Error> {
    let url = format!("{BASE_URL}/{name}:access");
    let contents = do_get(&url).await?;

    let res: AccessResponse = parse_json(&contents)?;

    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD
        .decode(&res.payload.data)
        .map_err(|e| Error::RequestFailed(format!("base64 decode error: {e}")))?;

    Ok(SecretPayload { data })
}

async fn list_secrets(parent: &str) -> Result<Vec<Secret>, Error> {
    let url = format!("{BASE_URL}/{parent}/secrets");
    let contents = do_get(&url).await?;
    let res: ListSecretsResponse = parse_json(&contents)?;
    Ok(res.secrets.into_iter().map(Into::into).collect())
}

async fn get_secret(name: &str) -> Result<Secret, Error> {
    let url = format!("{BASE_URL}/{name}");
    let contents = do_get(&url).await?;
    let res: SecretJson = parse_json(&contents)?;
    Ok(res.into())
}

async fn list_secret_versions(parent: &str) -> Result<Vec<SecretVersion>, Error> {
    let url = format!("{BASE_URL}/{parent}/versions");
    let contents = do_get(&url).await?;
    let res: ListSecretVersionsResponse = parse_json(&contents)?;
    Ok(res.versions.into_iter().map(Into::into).collect())
}

async fn add_secret_version(parent: &str, payload: SecretPayload) -> Result<SecretVersion, Error> {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&payload.data);

    let body = serde_json::json!({
        "payload": {
            "data": encoded,
        }
    });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| Error::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url = format!("{BASE_URL}/{parent}:addVersion");
    let contents = do_post(&url, &body_bytes).await?;
    let res: SecretVersionJson = parse_json(&contents)?;
    Ok(res.into())
}

async fn create_secret(parent: &str, secret_id: &str) -> Result<Secret, Error> {
    let body = serde_json::json!({
        "replication": {
            "automatic": {}
        }
    });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| Error::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url = format!("{BASE_URL}/{parent}/secrets?secretId={secret_id}");
    let contents = do_post(&url, &body_bytes).await?;
    let res: SecretJson = parse_json(&contents)?;
    Ok(res.into())
}

async fn delete_secret(name: &str) -> Result<(), Error> {
    let url = format!("{BASE_URL}/{name}");
    do_delete(&url).await
}
