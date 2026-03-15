wit_bindgen::generate!({
    world: "bigquery-provider",
    path: "wit",
    generate_all,
});

use exports::gcloud::bigquery::datasets::{Dataset, Error as DatasetError, Guest as DatasetsGuest};
use exports::gcloud::bigquery::jobs::{Error as JobError, Guest as JobsGuest, QueryResponse};
use exports::gcloud::bigquery::tabledata::{
    Error as TabledataError, Guest as TabledataGuest, InsertAllResponse,
};
use exports::gcloud::bigquery::tables::{Error as TableError, Guest as TablesGuest, Table};
use gcloud::auth::token_source::get_token;
use gcloud::bigquery::types::{
    DatasetReference, ErrorDetail, Field, InsertError, Row, TableReference,
};

struct Component;

export!(Component);

const SCOPE: &str = "https://www.googleapis.com/auth/bigquery";
const BASE_URL: &str = "https://bigquery.googleapis.com/bigquery/v2";

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

fn parse_json<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T, String> {
    serde_json::from_slice(data).map_err(|e| format!("JSON parse error: {e}"))
}

// --- JSON response types ---

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DatasetJson {
    id: Option<String>,
    dataset_reference: DatasetReferenceJson,
    friendly_name: Option<String>,
    location: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DatasetReferenceJson {
    project_id: String,
    dataset_id: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TableJson {
    id: Option<String>,
    table_reference: TableReferenceJson,
    friendly_name: Option<String>,
    #[serde(rename = "type")]
    type_: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TableReferenceJson {
    project_id: String,
    dataset_id: String,
    table_id: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryResponseJson {
    job_reference: JobReferenceJson,
    job_complete: bool,
    #[serde(default)]
    rows: Vec<RowJson>,
    total_rows: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct JobReferenceJson {
    job_id: String,
}

#[derive(serde::Deserialize)]
struct RowJson {
    f: Vec<FieldJson>,
}

#[derive(serde::Deserialize)]
struct FieldJson {
    v: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InsertAllResponseJson {
    #[serde(default)]
    insert_errors: Vec<InsertErrorJson>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InsertErrorJson {
    index: u32,
    errors: Vec<ErrorDetailJson>,
}

#[derive(serde::Deserialize)]
struct ErrorDetailJson {
    reason: Option<String>,
    location: Option<String>,
    message: Option<String>,
}

#[derive(serde::Deserialize)]
struct ListDatasetsResponse {
    #[serde(default)]
    datasets: Vec<DatasetJson>,
}

#[derive(serde::Deserialize)]
struct ListTablesResponse {
    #[serde(default)]
    tables: Vec<TableJson>,
}

// --- Conversions ---

impl From<DatasetJson> for Dataset {
    fn from(d: DatasetJson) -> Self {
        Dataset {
            id: d.id,
            dataset_reference: DatasetReference {
                project_id: d.dataset_reference.project_id,
                dataset_id: d.dataset_reference.dataset_id,
            },
            friendly_name: d.friendly_name,
            location: d.location,
        }
    }
}

impl From<TableJson> for Table {
    fn from(t: TableJson) -> Self {
        Table {
            id: t.id,
            table_reference: TableReference {
                project_id: t.table_reference.project_id,
                dataset_id: t.table_reference.dataset_id,
                table_id: t.table_reference.table_id,
            },
            friendly_name: t.friendly_name,
            type_: t.type_,
        }
    }
}

fn convert_query_response(r: QueryResponseJson) -> QueryResponse {
    QueryResponse {
        job_id: r.job_reference.job_id,
        job_complete: r.job_complete,
        rows: r
            .rows
            .into_iter()
            .map(|row| Row {
                fields: row
                    .f
                    .into_iter()
                    .map(|f| Field {
                        value: f.v.and_then(|v| match v {
                            serde_json::Value::Null => None,
                            serde_json::Value::String(s) => Some(s),
                            other => Some(other.to_string()),
                        }),
                    })
                    .collect(),
            })
            .collect(),
        total_rows: r.total_rows,
    }
}

fn convert_insert_rows_response(r: InsertAllResponseJson) -> InsertAllResponse {
    InsertAllResponse {
        insert_errors: r
            .insert_errors
            .into_iter()
            .map(|e| InsertError {
                index: e.index,
                errors: e
                    .errors
                    .into_iter()
                    .map(|d| ErrorDetail {
                        reason: d.reason,
                        location: d.location,
                        message: d.message,
                    })
                    .collect(),
            })
            .collect(),
    }
}

// --- Datasets Guest implementation ---

impl DatasetsGuest for Component {
    fn list_datasets(project: String) -> Result<Vec<Dataset>, DatasetError> {
        wstd::runtime::block_on(list_datasets(&project))
    }

    fn get_dataset(project: String, dataset_id: String) -> Result<Dataset, DatasetError> {
        wstd::runtime::block_on(get_dataset(&project, &dataset_id))
    }
}

// --- Tables Guest implementation ---

impl TablesGuest for Component {
    fn list_tables(project: String, dataset_id: String) -> Result<Vec<Table>, TableError> {
        wstd::runtime::block_on(list_tables(&project, &dataset_id))
    }

    fn get_table(
        project: String,
        dataset_id: String,
        table_id: String,
    ) -> Result<Table, TableError> {
        wstd::runtime::block_on(get_table(&project, &dataset_id, &table_id))
    }
}

// --- Jobs Guest implementation ---

impl JobsGuest for Component {
    fn query(project: String, sql: String) -> Result<QueryResponse, JobError> {
        wstd::runtime::block_on(query(&project, &sql))
    }

    fn get_query_results(project: String, job_id: String) -> Result<QueryResponse, JobError> {
        wstd::runtime::block_on(get_query_results(&project, &job_id))
    }
}

// --- Tabledata Guest implementation ---

impl TabledataGuest for Component {
    fn insert_rows(
        project: String,
        dataset_id: String,
        table_id: String,
        rows: Vec<String>,
    ) -> Result<InsertAllResponse, TabledataError> {
        wstd::runtime::block_on(insert_rows(&project, &dataset_id, &table_id, rows))
    }
}

// --- Async implementations: Datasets ---

async fn list_datasets(project: &str) -> Result<Vec<Dataset>, DatasetError> {
    let auth = auth_header().map_err(DatasetError::Auth)?;
    let url = format!("{BASE_URL}/projects/{project}/datasets");
    let contents = do_get(&url, &auth)
        .await
        .map_err(DatasetError::RequestFailed)?;
    let res: ListDatasetsResponse = parse_json(&contents).map_err(DatasetError::RequestFailed)?;
    Ok(res.datasets.into_iter().map(Into::into).collect())
}

async fn get_dataset(project: &str, dataset_id: &str) -> Result<Dataset, DatasetError> {
    let auth = auth_header().map_err(DatasetError::Auth)?;
    let url = format!("{BASE_URL}/projects/{project}/datasets/{dataset_id}");
    let contents = do_get(&url, &auth)
        .await
        .map_err(DatasetError::RequestFailed)?;
    let res: DatasetJson = parse_json(&contents).map_err(DatasetError::RequestFailed)?;
    Ok(res.into())
}

// --- Async implementations: Tables ---

async fn list_tables(project: &str, dataset_id: &str) -> Result<Vec<Table>, TableError> {
    let auth = auth_header().map_err(TableError::Auth)?;
    let url = format!("{BASE_URL}/projects/{project}/datasets/{dataset_id}/tables");
    let contents = do_get(&url, &auth)
        .await
        .map_err(TableError::RequestFailed)?;
    let res: ListTablesResponse = parse_json(&contents).map_err(TableError::RequestFailed)?;
    Ok(res.tables.into_iter().map(Into::into).collect())
}

async fn get_table(project: &str, dataset_id: &str, table_id: &str) -> Result<Table, TableError> {
    let auth = auth_header().map_err(TableError::Auth)?;
    let url = format!("{BASE_URL}/projects/{project}/datasets/{dataset_id}/tables/{table_id}");
    let contents = do_get(&url, &auth)
        .await
        .map_err(TableError::RequestFailed)?;
    let res: TableJson = parse_json(&contents).map_err(TableError::RequestFailed)?;
    Ok(res.into())
}

// --- Async implementations: Jobs ---

async fn query(project: &str, sql: &str) -> Result<QueryResponse, JobError> {
    let auth = auth_header().map_err(JobError::Auth)?;
    let body = serde_json::json!({
        "query": sql,
        "useLegacySql": false,
    });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| JobError::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url = format!("{BASE_URL}/projects/{project}/queries");
    let contents = do_post(&url, &auth, "application/json", &body_bytes)
        .await
        .map_err(JobError::RequestFailed)?;
    let res: QueryResponseJson = parse_json(&contents).map_err(JobError::RequestFailed)?;
    Ok(convert_query_response(res))
}

async fn get_query_results(project: &str, job_id: &str) -> Result<QueryResponse, JobError> {
    let auth = auth_header().map_err(JobError::Auth)?;
    let url = format!("{BASE_URL}/projects/{project}/queries/{job_id}");
    let contents = do_get(&url, &auth).await.map_err(JobError::RequestFailed)?;
    let res: QueryResponseJson = parse_json(&contents).map_err(JobError::RequestFailed)?;
    Ok(convert_query_response(res))
}

// --- Async implementations: Tabledata ---

async fn insert_rows(
    project: &str,
    dataset_id: &str,
    table_id: &str,
    rows: Vec<String>,
) -> Result<InsertAllResponse, TabledataError> {
    let auth = auth_header().map_err(TabledataError::Auth)?;

    let json_rows: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let value: serde_json::Value =
                serde_json::from_str(r).map_err(|e| format!("JSON parse error for row: {e}"))?;
            Ok(serde_json::json!({ "json": value }))
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(TabledataError::RequestFailed)?;

    let body = serde_json::json!({ "rows": json_rows });
    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| TabledataError::RequestFailed(format!("JSON serialize error: {e}")))?;

    let url =
        format!("{BASE_URL}/projects/{project}/datasets/{dataset_id}/tables/{table_id}/insertAll");
    let contents = do_post(&url, &auth, "application/json", &body_bytes)
        .await
        .map_err(TabledataError::RequestFailed)?;
    let res: InsertAllResponseJson =
        parse_json(&contents).map_err(TabledataError::RequestFailed)?;
    Ok(convert_insert_rows_response(res))
}
