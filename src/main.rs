use std::net::{Ipv4Addr, SocketAddr};

use axum::{
    extract::Extension,
    http::StatusCode,
    Json,
    response::IntoResponse,
    Router, routing::{get, post},
};
use k8s_openapi::api::batch::v1::{CronJob, Job};
use kube::{
    api::{Api, ListParams, PostParams},
    Client,
};
use serde::Deserialize;
use serde_json::{json, to_value, Value};
use tracing::info;

#[derive(Deserialize)]
pub struct CreateJob {
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateCronJob {
    pub name: String,
    pub syntax: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;
    let jobs: Api<Job> = Api::default_namespaced(client.clone());
    let cronjob: Api<CronJob> = Api::default_namespaced(client);

    let app = Router::new()
        .route("/jobs", post(create_job))
        .route("/jobs/stats", get(get_jobs))
        .route("/jobs/schedule", post(schedule_job))
        .layer(Extension((jobs, cronjob)));

    let addr = SocketAddr::from((Ipv4Addr::new(0, 0, 0, 0), 8080));

    info!("server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn get_jobs(Extension((jobs, _)): Extension<(Api<Job>, Api<CronJob>)>) -> impl IntoResponse {
    let data = match jobs.list(&ListParams::default()).await {
        Ok(data) => data,
        Err(e) => return handle_resp_err(e, StatusCode::INTERNAL_SERVER_ERROR),
    };
    (
        StatusCode::OK,
        Json(json!(data
            .into_iter()
            .map(|d| {
                let value = to_value(d).unwrap();
                json!({
                    "jobName": value["spec"]["template"]["metadata"]["labels"]["job-name"],
                    "type": value["status"]["conditions"][0]["type"],
                    "message": value["status"]["conditions"][0]["message"],
                    "reason": value["status"]["conditions"][0]["reason"],
                    "retryCount": value["status"]["failed"],
                    "startTime": value["status"]["startTime"],
                    "completionTime": value["status"]["completionTime"]
                })
            })
            .collect::<Value>())),
    )
}

async fn create_job(
    Extension((jobs, _)): Extension<(Api<Job>, Api<CronJob>)>,
    Json(payload): Json<CreateJob>,
) -> impl IntoResponse {
    let data = match serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": payload.name,
        },
        "spec": {
            "backoffLimit": 1,
            "activeDeadlineSeconds": 100,
            // "ttlSecondsAfterFinished": 300, // Clean up finished job
            "template": {
                "metadata": {
                    "name": "data-processing-job-pod"
                },
                "spec": {
                    "containers": [{
                        "name": "data-processor-container",
                        "image": "data-processor:latest",
                        "imagePullPolicy": "IfNotPresent"
                    }],
                    "restartPolicy": "Never",
                }
            }
        }
    })) {
        Ok(data) => data,
        Err(e) => return handle_resp_err(e, StatusCode::BAD_REQUEST),
    };
    if let Err(e) = jobs.create(&PostParams::default(), &data).await {
        return handle_resp_err(e, StatusCode::BAD_REQUEST);
    }
    (StatusCode::OK, Json(json!({})))
}

async fn schedule_job(
    Extension((_, cronjob)): Extension<(Api<Job>, Api<CronJob>)>,
    Json(payload): Json<CreateCronJob>,
) -> impl IntoResponse {
    let data = match serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "CronJob",
        "metadata": {
            "name": payload.name,
        },
        "spec": {
            "schedule": payload.syntax,
            "jobTemplate": {
                "spec":{
                    "template": {
                        "spec": {
                            "containers": [{
                                "name": "data-processor-container",
                                "image": "data-processor:latest",
                                "imagePullPolicy": "IfNotPresent"
                            }],
                            "restartPolicy": "Never",
                        }
                    }
                }
            }
        }
    })) {
        Ok(data) => data,
        Err(e) => return handle_resp_err(e, StatusCode::BAD_REQUEST),
    };
    if let Err(e) = cronjob.create(&PostParams::default(), &data).await {
        return handle_resp_err(e, StatusCode::BAD_REQUEST);
    }
    (StatusCode::OK, Json(json!({})))
}

fn handle_resp_err<E: core::fmt::Display>(
    e: E,
    status_code: StatusCode,
) -> (StatusCode, Json<Value>) {
    (status_code, Json(json!({ "message": format!("{e}") })))
}
