wit_bindgen::generate!({
    world: "storage-provider",
    path: "wit",
    generate_all,
});

use exports::gcloud::storage::buckets::{Bucket, Error as BucketError, Guest as BucketsGuest};
use exports::gcloud::storage::objects::{Error as ObjectError, Guest as ObjectsGuest, Object};
use gcloud::auth::token_source::get_token;

struct Component;

export!(Component);

const SCOPE: &str = "https://www.googleapis.com/auth/devstorage.full_control";
const BASE_URL: &str = "https://storage.googleapis.com/storage/v1";
const UPLOAD_URL: &str = "https://storage.googleapis.com/upload/storage/v1";

fn auth_header() -> Result<String, gcloud::auth::types::Error> {
    let scopes = vec![SCOPE.to_string()];
    let token = get_token(&scopes)?;
    Ok(format!("{} {}", token.token_type, token.access_token))
}

async fn do_get(url: &str, auth: &str) -> Result<Vec<u8>, String> {
    use wstd::http::{Client, Request};

    let request = Request::get(url)
        .header("Authorization", auth)
        .body(wstd::http::Body::empty())
        .map_err(|e| format!("request build error: {e}"))?;

    let response = Client::new()
        .send(request)
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    let status = response.status();
    let mut body = response.into_body();
    let contents = body
        .contents()
        .await
        .map_err(|e| format!("body read error: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "HTTP {}: {}",
            status.as_u16(),
            String::from_utf8_lossy(contents)
        ));
    }

    Ok(contents.to_vec())
}

async fn do_post(url: &str, auth: &str, content_type: &str, body_bytes: &[u8]) -> Result<Vec<u8>, String> {
    use wstd::http::{Client, Request};

    let request = Request::post(url)
        .header("Authorization", auth)
        .header("Content-Type", content_type)
        .body(wstd::http::Body::from(body_bytes.to_vec()))
        .map_err(|e| format!("request build error: {e}"))?;

    let response = Client::new()
        .send(request)
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    let status = response.status();
    let mut body = response.into_body();
    let contents = body
        .contents()
        .await
        .map_err(|e| format!("body read error: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "HTTP {}: {}",
            status.as_u16(),
            String::from_utf8_lossy(contents)
        ));
    }

    Ok(contents.to_vec())
}

async fn do_delete(url: &str, auth: &str) -> Result<(), String> {
    use wstd::http::{Client, Request};

    let request = Request::delete(url)
        .header("Authorization", auth)
        .body(wstd::http::Body::empty())
        .map_err(|e| format!("request build error: {e}"))?;

    let response = Client::new()
        .send(request)
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let mut body = response.into_body();
        let contents = body
            .contents()
            .await
            .map_err(|e| format!("body read error: {e}"))?;
        return Err(format!(
            "HTTP {}: {}",
            status.as_u16(),
            String::from_utf8_lossy(contents)
        ));
    }

    Ok(())
}

fn parse_json<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T, String> {
    serde_json::from_slice(data).map_err(|e| format!("JSON parse error: {e}"))
}

// --- JSON response types ---

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BucketJson {
    name: String,
    location: Option<String>,
    storage_class: Option<String>,
    time_created: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObjectJson {
    name: String,
    bucket: String,
    size: Option<String>,
    content_type: Option<String>,
    time_created: Option<String>,
}

#[derive(serde::Deserialize)]
struct ListBucketsResponse {
    #[serde(default)]
    items: Vec<BucketJson>,
}

#[derive(serde::Deserialize)]
struct ListObjectsResponse {
    #[serde(default)]
    items: Vec<ObjectJson>,
}

impl From<BucketJson> for Bucket {
    fn from(b: BucketJson) -> Self {
        Bucket {
            name: b.name,
            location: b.location,
            storage_class: b.storage_class,
            time_created: b.time_created,
        }
    }
}

impl From<ObjectJson> for Object {
    fn from(o: ObjectJson) -> Self {
        Object {
            name: o.name,
            bucket: o.bucket,
            size: o.size,
            content_type: o.content_type,
            time_created: o.time_created,
        }
    }
}

// --- Buckets Guest implementation ---

impl BucketsGuest for Component {
    fn list_buckets(project: String) -> Result<Vec<Bucket>, BucketError> {
        wstd::runtime::block_on(list_buckets(&project))
    }

    fn get_bucket(name: String) -> Result<Bucket, BucketError> {
        wstd::runtime::block_on(get_bucket(&name))
    }

    fn create_bucket(project: String, name: String) -> Result<Bucket, BucketError> {
        wstd::runtime::block_on(create_bucket(&project, &name))
    }

    fn delete_bucket(name: String) -> Result<(), BucketError> {
        wstd::runtime::block_on(delete_bucket(&name))
    }
}

// --- Objects Guest implementation ---

impl ObjectsGuest for Component {
    fn list_objects(bucket: String, prefix: Option<String>) -> Result<Vec<Object>, ObjectError> {
        wstd::runtime::block_on(list_objects(&bucket, prefix.as_deref()))
    }

    fn get_object(bucket: String, name: String) -> Result<Vec<u8>, ObjectError> {
        wstd::runtime::block_on(get_object(&bucket, &name))
    }

    fn upload_object(
        bucket: String,
        name: String,
        data: Vec<u8>,
        content_type: String,
    ) -> Result<Object, ObjectError> {
        wstd::runtime::block_on(upload_object(&bucket, &name, data, &content_type))
    }

    fn delete_object(bucket: String, name: String) -> Result<(), ObjectError> {
        wstd::runtime::block_on(delete_object(&bucket, &name))
    }
}

// --- Async implementations: Buckets ---

async fn list_buckets(project: &str) -> Result<Vec<Bucket>, BucketError> {
    let auth = auth_header().map_err(BucketError::Auth)?;
    let url = format!("{BASE_URL}/b?project={project}");
    let contents = do_get(&url, &auth).await.map_err(BucketError::RequestFailed)?;
    let res: ListBucketsResponse = parse_json(&contents).map_err(BucketError::RequestFailed)?;
    Ok(res.items.into_iter().map(Into::into).collect())
}

async fn get_bucket(name: &str) -> Result<Bucket, BucketError> {
    let auth = auth_header().map_err(BucketError::Auth)?;
    let url = format!("{BASE_URL}/b/{name}");
    let contents = do_get(&url, &auth).await.map_err(BucketError::RequestFailed)?;
    let res: BucketJson = parse_json(&contents).map_err(BucketError::RequestFailed)?;
    Ok(res.into())
}

async fn create_bucket(project: &str, name: &str) -> Result<Bucket, BucketError> {
    let auth = auth_header().map_err(BucketError::Auth)?;
    let body = serde_json::json!({ "name": name });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| BucketError::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url = format!("{BASE_URL}/b?project={project}");
    let contents = do_post(&url, &auth, "application/json", &body_bytes)
        .await
        .map_err(BucketError::RequestFailed)?;
    let res: BucketJson = parse_json(&contents).map_err(BucketError::RequestFailed)?;
    Ok(res.into())
}

async fn delete_bucket(name: &str) -> Result<(), BucketError> {
    let auth = auth_header().map_err(BucketError::Auth)?;
    let url = format!("{BASE_URL}/b/{name}");
    do_delete(&url, &auth).await.map_err(BucketError::RequestFailed)
}

// --- Async implementations: Objects ---

async fn list_objects(bucket: &str, prefix: Option<&str>) -> Result<Vec<Object>, ObjectError> {
    let auth = auth_header().map_err(ObjectError::Auth)?;
    let mut url = format!("{BASE_URL}/b/{bucket}/o");
    if let Some(prefix) = prefix {
        url.push_str(&format!("?prefix={prefix}"));
    }
    let contents = do_get(&url, &auth).await.map_err(ObjectError::RequestFailed)?;
    let res: ListObjectsResponse = parse_json(&contents).map_err(ObjectError::RequestFailed)?;
    Ok(res.items.into_iter().map(Into::into).collect())
}

async fn get_object(bucket: &str, name: &str) -> Result<Vec<u8>, ObjectError> {
    let auth = auth_header().map_err(ObjectError::Auth)?;
    let url = format!("{BASE_URL}/b/{bucket}/o/{name}?alt=media");
    do_get(&url, &auth).await.map_err(ObjectError::RequestFailed)
}

async fn upload_object(
    bucket: &str,
    name: &str,
    data: Vec<u8>,
    content_type: &str,
) -> Result<Object, ObjectError> {
    let auth = auth_header().map_err(ObjectError::Auth)?;
    let url = format!("{UPLOAD_URL}/b/{bucket}/o?uploadType=media&name={name}");
    let contents = do_post(&url, &auth, content_type, &data)
        .await
        .map_err(ObjectError::RequestFailed)?;
    let res: ObjectJson = parse_json(&contents).map_err(ObjectError::RequestFailed)?;
    Ok(res.into())
}

async fn delete_object(bucket: &str, name: &str) -> Result<(), ObjectError> {
    let auth = auth_header().map_err(ObjectError::Auth)?;
    let url = format!("{BASE_URL}/b/{bucket}/o/{name}");
    do_delete(&url, &auth).await.map_err(ObjectError::RequestFailed)
}
