wit_bindgen::generate!({
    world: "tasks-provider",
    path: "wit",
    generate_all,
});

use crate::gcloud::tasks::types::{HttpMethod, QueueState};
use exports::gcloud::tasks::queues::{Error as QueueError, Guest as QueuesGuest, Queue};
use exports::gcloud::tasks::tasks::{Error as TaskError, Guest as TasksGuest, HttpRequest, Task};
use gcloud::auth::token_source::get_token;

struct Component;

export!(Component);

const SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";
const BASE_URL: &str = "https://cloudtasks.googleapis.com/v2";

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
struct QueueJson {
    name: String,
    state: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskJson {
    name: String,
    schedule_time: Option<String>,
    create_time: Option<String>,
}

#[derive(serde::Deserialize)]
struct ListQueuesResponse {
    #[serde(default)]
    queues: Vec<QueueJson>,
}

#[derive(serde::Deserialize)]
struct ListTasksResponse {
    #[serde(default)]
    tasks: Vec<TaskJson>,
}

fn parse_queue_state(s: &str) -> QueueState {
    match s {
        "RUNNING" => QueueState::Running,
        "PAUSED" => QueueState::Paused,
        "DISABLED" => QueueState::Disabled,
        _ => QueueState::StateUnspecified,
    }
}

fn http_method_to_str(m: &HttpMethod) -> &'static str {
    match m {
        HttpMethod::Post => "POST",
        HttpMethod::Get => "GET",
        HttpMethod::Head => "HEAD",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Options => "OPTIONS",
        HttpMethod::MethodUnspecified => "POST",
    }
}

impl From<QueueJson> for Queue {
    fn from(q: QueueJson) -> Self {
        Queue {
            name: q.name,
            state: q.state.map(|s| parse_queue_state(&s)),
        }
    }
}

impl From<TaskJson> for Task {
    fn from(t: TaskJson) -> Self {
        Task {
            name: t.name,
            schedule_time: t.schedule_time,
            create_time: t.create_time,
        }
    }
}

// --- Queues Guest implementation ---

impl QueuesGuest for Component {
    fn list_queues(parent: String) -> Result<Vec<Queue>, QueueError> {
        wstd::runtime::block_on(list_queues(&parent))
    }

    fn get_queue(name: String) -> Result<Queue, QueueError> {
        wstd::runtime::block_on(get_queue(&name))
    }

    fn create_queue(parent: String, queue_id: String) -> Result<Queue, QueueError> {
        wstd::runtime::block_on(create_queue(&parent, &queue_id))
    }

    fn delete_queue(name: String) -> Result<(), QueueError> {
        wstd::runtime::block_on(delete_queue(&name))
    }

    fn pause_queue(name: String) -> Result<Queue, QueueError> {
        wstd::runtime::block_on(pause_queue(&name))
    }

    fn resume_queue(name: String) -> Result<Queue, QueueError> {
        wstd::runtime::block_on(resume_queue(&name))
    }
}

// --- Tasks Guest implementation ---

impl TasksGuest for Component {
    fn list_tasks(parent: String) -> Result<Vec<Task>, TaskError> {
        wstd::runtime::block_on(list_tasks(&parent))
    }

    fn get_task(name: String) -> Result<Task, TaskError> {
        wstd::runtime::block_on(get_task(&name))
    }

    fn create_task(
        parent: String,
        http_request: HttpRequest,
        schedule_time: Option<String>,
    ) -> Result<Task, TaskError> {
        wstd::runtime::block_on(create_task(&parent, http_request, schedule_time))
    }

    fn delete_task(name: String) -> Result<(), TaskError> {
        wstd::runtime::block_on(delete_task(&name))
    }

    fn run_task(name: String) -> Result<Task, TaskError> {
        wstd::runtime::block_on(run_task(&name))
    }
}

// --- Async implementations: Queues ---

async fn list_queues(parent: &str) -> Result<Vec<Queue>, QueueError> {
    let auth = auth_header().map_err(QueueError::Auth)?;
    let url = format!("{BASE_URL}/{parent}/queues");
    let contents = do_get(&url, &auth)
        .await
        .map_err(QueueError::RequestFailed)?;
    let res: ListQueuesResponse = parse_json(&contents).map_err(QueueError::RequestFailed)?;
    Ok(res.queues.into_iter().map(Into::into).collect())
}

async fn get_queue(name: &str) -> Result<Queue, QueueError> {
    let auth = auth_header().map_err(QueueError::Auth)?;
    let url = format!("{BASE_URL}/{name}");
    let contents = do_get(&url, &auth)
        .await
        .map_err(QueueError::RequestFailed)?;
    let res: QueueJson = parse_json(&contents).map_err(QueueError::RequestFailed)?;
    Ok(res.into())
}

async fn create_queue(parent: &str, queue_id: &str) -> Result<Queue, QueueError> {
    let auth = auth_header().map_err(QueueError::Auth)?;
    let body = serde_json::json!({ "name": format!("{parent}/queues/{queue_id}") });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| QueueError::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url = format!("{BASE_URL}/{parent}/queues");
    let contents = do_post(&url, &auth, "application/json", &body_bytes)
        .await
        .map_err(QueueError::RequestFailed)?;
    let res: QueueJson = parse_json(&contents).map_err(QueueError::RequestFailed)?;
    Ok(res.into())
}

async fn delete_queue(name: &str) -> Result<(), QueueError> {
    let auth = auth_header().map_err(QueueError::Auth)?;
    let url = format!("{BASE_URL}/{name}");
    do_delete(&url, &auth)
        .await
        .map_err(QueueError::RequestFailed)
}

async fn pause_queue(name: &str) -> Result<Queue, QueueError> {
    let auth = auth_header().map_err(QueueError::Auth)?;
    let url = format!("{BASE_URL}/{name}:pause");
    let contents = do_post(&url, &auth, "application/json", b"{}")
        .await
        .map_err(QueueError::RequestFailed)?;
    let res: QueueJson = parse_json(&contents).map_err(QueueError::RequestFailed)?;
    Ok(res.into())
}

async fn resume_queue(name: &str) -> Result<Queue, QueueError> {
    let auth = auth_header().map_err(QueueError::Auth)?;
    let url = format!("{BASE_URL}/{name}:resume");
    let contents = do_post(&url, &auth, "application/json", b"{}")
        .await
        .map_err(QueueError::RequestFailed)?;
    let res: QueueJson = parse_json(&contents).map_err(QueueError::RequestFailed)?;
    Ok(res.into())
}

// --- Async implementations: Tasks ---

async fn list_tasks(parent: &str) -> Result<Vec<Task>, TaskError> {
    let auth = auth_header().map_err(TaskError::Auth)?;
    let url = format!("{BASE_URL}/{parent}/tasks");
    let contents = do_get(&url, &auth)
        .await
        .map_err(TaskError::RequestFailed)?;
    let res: ListTasksResponse = parse_json(&contents).map_err(TaskError::RequestFailed)?;
    Ok(res.tasks.into_iter().map(Into::into).collect())
}

async fn get_task(name: &str) -> Result<Task, TaskError> {
    let auth = auth_header().map_err(TaskError::Auth)?;
    let url = format!("{BASE_URL}/{name}");
    let contents = do_get(&url, &auth)
        .await
        .map_err(TaskError::RequestFailed)?;
    let res: TaskJson = parse_json(&contents).map_err(TaskError::RequestFailed)?;
    Ok(res.into())
}

async fn create_task(
    parent: &str,
    http_request: HttpRequest,
    schedule_time: Option<String>,
) -> Result<Task, TaskError> {
    use base64::Engine;

    let auth = auth_header().map_err(TaskError::Auth)?;

    let headers: serde_json::Map<String, serde_json::Value> = http_request
        .headers
        .iter()
        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        .collect();

    let encoded_body = base64::engine::general_purpose::STANDARD.encode(&http_request.body);

    let mut task_obj = serde_json::json!({
        "httpRequest": {
            "url": http_request.url,
            "httpMethod": http_method_to_str(&http_request.method),
            "headers": headers,
            "body": encoded_body,
        }
    });

    if let Some(ref st) = schedule_time {
        task_obj["scheduleTime"] = serde_json::Value::String(st.clone());
    }

    let body = serde_json::json!({ "task": task_obj });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| TaskError::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url = format!("{BASE_URL}/{parent}/tasks");
    let contents = do_post(&url, &auth, "application/json", &body_bytes)
        .await
        .map_err(TaskError::RequestFailed)?;
    let res: TaskJson = parse_json(&contents).map_err(TaskError::RequestFailed)?;
    Ok(res.into())
}

async fn delete_task(name: &str) -> Result<(), TaskError> {
    let auth = auth_header().map_err(TaskError::Auth)?;
    let url = format!("{BASE_URL}/{name}");
    do_delete(&url, &auth)
        .await
        .map_err(TaskError::RequestFailed)
}

async fn run_task(name: &str) -> Result<Task, TaskError> {
    let auth = auth_header().map_err(TaskError::Auth)?;
    let url = format!("{BASE_URL}/{name}:run");
    let contents = do_post(&url, &auth, "application/json", b"{}")
        .await
        .map_err(TaskError::RequestFailed)?;
    let res: TaskJson = parse_json(&contents).map_err(TaskError::RequestFailed)?;
    Ok(res.into())
}
