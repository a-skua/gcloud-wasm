wit_bindgen::generate!({
    world: "storage-provider",
    path: "wit",
    generate_all,
});

use exports::gcloud::storage::buckets::{Bucket, Error, Guest};
use gcloud::auth::token_source::get_token;

struct Component;

export!(Component);

impl Guest for Component {
    fn list_buckets(project: String) -> Result<Vec<Bucket>, Error> {
        wstd::runtime::block_on(list_buckets(&project))
    }
}

async fn list_buckets(project: &str) -> Result<Vec<Bucket>, Error> {
    let scopes = vec!["https://www.googleapis.com/auth/devstorage.read_only".to_string()];
    let token = get_token(&scopes).map_err(Error::Auth)?;

    let url = format!(
        "https://storage.googleapis.com/storage/v1/b?project={project}",
    );

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
    struct ListResponse {
        #[serde(default)]
        items: Vec<BucketItem>,
    }

    #[derive(serde::Deserialize)]
    struct BucketItem {
        name: String,
    }

    let res: ListResponse = serde_json::from_slice(contents)
        .map_err(|e| Error::RequestFailed(format!("JSON parse error: {e}")))?;

    Ok(res.items.into_iter().map(|b| Bucket { name: b.name }).collect())
}
