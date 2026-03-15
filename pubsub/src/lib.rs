wit_bindgen::generate!({
    world: "pubsub-provider",
    path: "wit",
    generate_all,
});

use exports::gcloud::pubsub::subscriptions::{
    Error as SubscriptionError, Guest as SubscriptionsGuest, ReceivedMessage, Subscription,
};
use exports::gcloud::pubsub::topics::{
    Error as TopicError, Guest as TopicsGuest, PublishMessage, Topic,
};
use gcloud::auth::token_source::get_token;

struct Component;

export!(Component);

const SCOPE: &str = "https://www.googleapis.com/auth/pubsub";
const BASE_URL: &str = "https://pubsub.googleapis.com/v1";

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

async fn do_post(
    url: &str,
    auth: &str,
    content_type: &str,
    body_bytes: &[u8],
) -> Result<Vec<u8>, String> {
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

async fn do_put(
    url: &str,
    auth: &str,
    content_type: &str,
    body_bytes: &[u8],
) -> Result<Vec<u8>, String> {
    use wstd::http::{Client, Request};

    let request = Request::put(url)
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

// --- JSON response/request types ---

#[derive(serde::Deserialize)]
struct TopicJson {
    name: String,
}

#[derive(serde::Deserialize)]
struct SubscriptionJson {
    name: String,
    #[serde(default)]
    topic: String,
}

#[derive(serde::Deserialize)]
struct ListTopicsResponse {
    #[serde(default)]
    topics: Vec<TopicJson>,
}

#[derive(serde::Deserialize)]
struct ListSubscriptionsResponse {
    #[serde(default)]
    subscriptions: Vec<SubscriptionJson>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublishResponse {
    #[serde(default)]
    message_ids: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullResponse {
    #[serde(default)]
    received_messages: Vec<ReceivedMessageJson>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReceivedMessageJson {
    ack_id: String,
    message: PubsubMessageJson,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PubsubMessageJson {
    #[serde(default)]
    data: Option<String>,
    #[serde(default)]
    attributes: Option<std::collections::HashMap<String, String>>,
    message_id: String,
}

impl From<TopicJson> for Topic {
    fn from(t: TopicJson) -> Self {
        Topic { name: t.name }
    }
}

impl From<SubscriptionJson> for Subscription {
    fn from(s: SubscriptionJson) -> Self {
        Subscription {
            name: s.name,
            topic: s.topic,
        }
    }
}

// --- Topics Guest implementation ---

impl TopicsGuest for Component {
    fn list_topics(project: String) -> Result<Vec<Topic>, TopicError> {
        wstd::runtime::block_on(list_topics(&project))
    }

    fn get_topic(name: String) -> Result<Topic, TopicError> {
        wstd::runtime::block_on(get_topic(&name))
    }

    fn create_topic(name: String) -> Result<Topic, TopicError> {
        wstd::runtime::block_on(create_topic(&name))
    }

    fn delete_topic(name: String) -> Result<(), TopicError> {
        wstd::runtime::block_on(delete_topic(&name))
    }

    fn publish(topic: String, messages: Vec<PublishMessage>) -> Result<Vec<String>, TopicError> {
        wstd::runtime::block_on(publish(&topic, messages))
    }
}

// --- Subscriptions Guest implementation ---

impl SubscriptionsGuest for Component {
    fn list_subscriptions(project: String) -> Result<Vec<Subscription>, SubscriptionError> {
        wstd::runtime::block_on(list_subscriptions(&project))
    }

    fn get_subscription(name: String) -> Result<Subscription, SubscriptionError> {
        wstd::runtime::block_on(get_subscription(&name))
    }

    fn create_subscription(name: String, topic: String) -> Result<Subscription, SubscriptionError> {
        wstd::runtime::block_on(create_subscription(&name, &topic))
    }

    fn delete_subscription(name: String) -> Result<(), SubscriptionError> {
        wstd::runtime::block_on(delete_subscription(&name))
    }

    fn pull(
        subscription: String,
        max_messages: u32,
    ) -> Result<Vec<ReceivedMessage>, SubscriptionError> {
        wstd::runtime::block_on(pull(&subscription, max_messages))
    }

    fn acknowledge(subscription: String, ack_ids: Vec<String>) -> Result<(), SubscriptionError> {
        wstd::runtime::block_on(acknowledge(&subscription, ack_ids))
    }
}

// --- Async implementations: Topics ---

async fn list_topics(project: &str) -> Result<Vec<Topic>, TopicError> {
    let auth = auth_header().map_err(TopicError::Auth)?;
    let url = format!("{BASE_URL}/{project}/topics");
    let contents = do_get(&url, &auth)
        .await
        .map_err(TopicError::RequestFailed)?;
    let res: ListTopicsResponse = parse_json(&contents).map_err(TopicError::RequestFailed)?;
    Ok(res.topics.into_iter().map(Into::into).collect())
}

async fn get_topic(name: &str) -> Result<Topic, TopicError> {
    let auth = auth_header().map_err(TopicError::Auth)?;
    let url = format!("{BASE_URL}/{name}");
    let contents = do_get(&url, &auth)
        .await
        .map_err(TopicError::RequestFailed)?;
    let res: TopicJson = parse_json(&contents).map_err(TopicError::RequestFailed)?;
    Ok(res.into())
}

async fn create_topic(name: &str) -> Result<Topic, TopicError> {
    let auth = auth_header().map_err(TopicError::Auth)?;
    let url = format!("{BASE_URL}/{name}");
    let contents = do_put(&url, &auth, "application/json", b"{}")
        .await
        .map_err(TopicError::RequestFailed)?;
    let res: TopicJson = parse_json(&contents).map_err(TopicError::RequestFailed)?;
    Ok(res.into())
}

async fn delete_topic(name: &str) -> Result<(), TopicError> {
    let auth = auth_header().map_err(TopicError::Auth)?;
    let url = format!("{BASE_URL}/{name}");
    do_delete(&url, &auth)
        .await
        .map_err(TopicError::RequestFailed)
}

async fn publish(topic: &str, messages: Vec<PublishMessage>) -> Result<Vec<String>, TopicError> {
    use base64::Engine;

    let auth = auth_header().map_err(TopicError::Auth)?;

    let msgs: Vec<serde_json::Value> = messages
        .into_iter()
        .map(|m| {
            let encoded = base64::engine::general_purpose::STANDARD.encode(&m.data);
            let attrs: std::collections::HashMap<String, String> =
                m.attributes.into_iter().collect();
            serde_json::json!({
                "data": encoded,
                "attributes": attrs,
            })
        })
        .collect();

    let body = serde_json::json!({ "messages": msgs });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| TopicError::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url = format!("{BASE_URL}/{topic}:publish");
    let contents = do_post(&url, &auth, "application/json", &body_bytes)
        .await
        .map_err(TopicError::RequestFailed)?;
    let res: PublishResponse = parse_json(&contents).map_err(TopicError::RequestFailed)?;
    Ok(res.message_ids)
}

// --- Async implementations: Subscriptions ---

async fn list_subscriptions(project: &str) -> Result<Vec<Subscription>, SubscriptionError> {
    let auth = auth_header().map_err(SubscriptionError::Auth)?;
    let url = format!("{BASE_URL}/{project}/subscriptions");
    let contents = do_get(&url, &auth)
        .await
        .map_err(SubscriptionError::RequestFailed)?;
    let res: ListSubscriptionsResponse =
        parse_json(&contents).map_err(SubscriptionError::RequestFailed)?;
    Ok(res.subscriptions.into_iter().map(Into::into).collect())
}

async fn get_subscription(name: &str) -> Result<Subscription, SubscriptionError> {
    let auth = auth_header().map_err(SubscriptionError::Auth)?;
    let url = format!("{BASE_URL}/{name}");
    let contents = do_get(&url, &auth)
        .await
        .map_err(SubscriptionError::RequestFailed)?;
    let res: SubscriptionJson = parse_json(&contents).map_err(SubscriptionError::RequestFailed)?;
    Ok(res.into())
}

async fn create_subscription(name: &str, topic: &str) -> Result<Subscription, SubscriptionError> {
    let auth = auth_header().map_err(SubscriptionError::Auth)?;
    let body = serde_json::json!({ "topic": topic });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| SubscriptionError::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url = format!("{BASE_URL}/{name}");
    let contents = do_put(&url, &auth, "application/json", &body_bytes)
        .await
        .map_err(SubscriptionError::RequestFailed)?;
    let res: SubscriptionJson = parse_json(&contents).map_err(SubscriptionError::RequestFailed)?;
    Ok(res.into())
}

async fn delete_subscription(name: &str) -> Result<(), SubscriptionError> {
    let auth = auth_header().map_err(SubscriptionError::Auth)?;
    let url = format!("{BASE_URL}/{name}");
    do_delete(&url, &auth)
        .await
        .map_err(SubscriptionError::RequestFailed)
}

async fn pull(
    subscription: &str,
    max_messages: u32,
) -> Result<Vec<ReceivedMessage>, SubscriptionError> {
    use base64::Engine;

    let auth = auth_header().map_err(SubscriptionError::Auth)?;
    let body = serde_json::json!({ "maxMessages": max_messages });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| SubscriptionError::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url = format!("{BASE_URL}/{subscription}:pull");
    let contents = do_post(&url, &auth, "application/json", &body_bytes)
        .await
        .map_err(SubscriptionError::RequestFailed)?;
    let res: PullResponse = parse_json(&contents).map_err(SubscriptionError::RequestFailed)?;

    let messages = res
        .received_messages
        .into_iter()
        .map(|rm| {
            let data = rm
                .message
                .data
                .map(|d| {
                    base64::engine::general_purpose::STANDARD
                        .decode(&d)
                        .unwrap_or_default()
                })
                .unwrap_or_default();

            let attributes: Vec<(String, String)> = rm
                .message
                .attributes
                .map(|a| a.into_iter().collect())
                .unwrap_or_default();

            ReceivedMessage {
                ack_id: rm.ack_id,
                message_id: rm.message.message_id,
                data,
                attributes,
            }
        })
        .collect();

    Ok(messages)
}

async fn acknowledge(subscription: &str, ack_ids: Vec<String>) -> Result<(), SubscriptionError> {
    let auth = auth_header().map_err(SubscriptionError::Auth)?;
    let body = serde_json::json!({ "ackIds": ack_ids });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| SubscriptionError::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url = format!("{BASE_URL}/{subscription}:acknowledge");
    do_post(&url, &auth, "application/json", &body_bytes)
        .await
        .map_err(SubscriptionError::RequestFailed)?;

    Ok(())
}
