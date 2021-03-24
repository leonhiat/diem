// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    errors::{is_internal_error, JsonRpcError},
    methods::{build_registry, JsonRpcRequest, JsonRpcService, RpcRegistry},
    response::{JsonRpcResponse, X_DIEM_CHAIN_ID, X_DIEM_TIMESTAMP_USEC_ID, X_DIEM_VERSION_ID},
    util::{sdk_info_from_user_agent, SdkInfo},
};
use anyhow::{ensure, Result};
use diem_config::config::{NodeConfig, RoleType};
use diem_logger::{debug, Schema};
use diem_mempool::MempoolClientSender;
use diem_types::{chain_id::ChainId, ledger_info::LedgerInfoWithSignatures};
use futures::future::{join_all, Either};
use rand::{rngs::OsRng, RngCore};
use serde_json::{map::Map, Value};
use std::{
    net::SocketAddr,
    ops::Sub,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use storage_interface::DbReader;
use tokio::runtime::{Builder, Runtime};
use warp::{
    http::header,
    reject::{self, Reject},
    Filter, Reply,
};

// Counter labels for runtime metrics
const LABEL_FAIL: &str = "fail";
const LABEL_SUCCESS: &str = "success";
const LABEL_BATCH: &str = "batch";
const LABEL_SINGLE: &str = "single";

#[derive(Schema)]
struct HttpRequestLog<'a> {
    #[schema(display)]
    remote_addr: Option<std::net::SocketAddr>,
    method: String,
    path: String,
    status: u16,
    referer: Option<&'a str>,
    user_agent: Option<&'a str>,
    #[schema(debug)]
    elapsed: std::time::Duration,
    forwarded: Option<&'a str>,
}

#[derive(Schema)]
struct RpcRequestLog {
    trace_id: String,
    request: Value,
}

#[derive(Schema)]
struct RpcResponseLog<'a> {
    trace_id: String,
    is_batch: bool,
    response_error: bool,
    response: &'a JsonRpcResponse,
}

// HealthCheckParams is optional params for different layer's health check.
// If no param is provided, server return 200 by default to indicate HTTP server is running health.
#[derive(serde::Deserialize)]
struct HealthCheckParams {
    // Health check returns 200 when this param is provided and meet the following condition:
    //   server latest ledger info timestamp >= server current time timestamp + duration_secs
    pub duration_secs: Option<u64>,
}

#[macro_export]
macro_rules! log_response {
    ($trace_id: expr, $resp: expr, $is_batch: expr) => {
        let log = RpcResponseLog {
            trace_id: $trace_id,
            is_batch: $is_batch,
            response_error: $resp.error.is_some(),
            response: $resp,
        };
        match &$resp.error {
            Some(error) if is_internal_error(&error.code) => {
                diem_logger::error!(log)
            }
            _ => diem_logger::trace!(log),
        }
    };
}

/// Creates HTTP server (warp-based) that serves JSON RPC requests
/// Returns handle to corresponding Tokio runtime
pub fn bootstrap(
    address: SocketAddr,
    batch_size_limit: u16,
    page_size_limit: u16,
    content_len_limit: usize,
    tls_cert_path: &Option<String>,
    tls_key_path: &Option<String>,
    diem_db: Arc<dyn DbReader>,
    mp_sender: MempoolClientSender,
    role: RoleType,
    chain_id: ChainId,
) -> Runtime {
    let runtime = Builder::new_multi_thread()
        .thread_name("json-rpc")
        .enable_all()
        .build()
        .expect("[json-rpc] failed to create runtime");

    let registry = Arc::new(build_registry());
    let service = JsonRpcService::new(
        diem_db.clone(),
        mp_sender,
        role,
        chain_id,
        batch_size_limit,
        page_size_limit,
    );

    let base_route = warp::any()
        .and(warp::post())
        .and(warp::header::exact("content-type", "application/json"))
        .and(warp::body::content_length_limit(content_len_limit as u64))
        .and(warp::body::json())
        .and(warp::any().map(move || service.clone()))
        .and(warp::any().map(move || Arc::clone(&registry)))
        .and(warp::filters::header::optional::<String>("user-agent"))
        .and_then(rpc_endpoint)
        .with(warp::log::custom(|info| {
            debug!(HttpRequestLog {
                remote_addr: info.remote_addr(),
                method: info.method().to_string(),
                path: info.path().to_string(),
                status: info.status().as_u16(),
                referer: info.referer(),
                user_agent: info.user_agent(),
                elapsed: info.elapsed(),
                forwarded: info
                    .request_headers()
                    .get(header::FORWARDED)
                    .and_then(|v| v.to_str().ok())
            })
        }))
        // CORS is required for full node server to accept requests from different domain web pages.
        // It needs to be configured for the json-rpc request accepting method and headers.
        // Technically it's fine for any headers, but for simplicity we only set must have header
        // content-type.
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["POST"])
                .allow_headers(vec![header::CONTENT_TYPE]),
        );

    // For now we still allow user to use "/", but user should start to move to "/v1" soon
    let route_root = warp::path::end().and(base_route.clone());

    let route_v1 = warp::path::path("v1")
        .and(warp::path::end())
        .and(base_route);

    let health_route = warp::path!("-" / "healthy")
        .and(warp::path::end())
        .and(warp::query().map(move |params: HealthCheckParams| params))
        .and(warp::any().map(move || diem_db.clone()))
        .and(warp::any().map(SystemTime::now))
        .and_then(health_check);

    let full_route = health_route.or(route_v1.or(route_root));

    // Ensure that we actually bind to the socket first before spawning the
    // server tasks. This helps in tests to prevent races where a client attempts
    // to make a request before the server task is actually listening on the
    // socket.
    //
    // Note: we need to enter the runtime context first to actually bind, since
    //       tokio TcpListener can only be bound inside a tokio context.
    let _guard = runtime.enter();
    let server = match tls_cert_path {
        None => Either::Left(warp::serve(full_route).bind(address)),
        Some(cert_path) => Either::Right(
            warp::serve(full_route)
                .tls()
                .cert_path(cert_path)
                .key_path(tls_key_path.as_ref().unwrap())
                .bind(address),
        ),
    };
    runtime.handle().spawn(server);
    runtime
}

/// Creates JSON RPC endpoint by given node config
pub fn bootstrap_from_config(
    config: &NodeConfig,
    chain_id: ChainId,
    diem_db: Arc<dyn DbReader>,
    mp_sender: MempoolClientSender,
) -> Runtime {
    bootstrap(
        config.json_rpc.address,
        config.json_rpc.batch_size_limit,
        config.json_rpc.page_size_limit,
        config.json_rpc.content_length_limit,
        &config.json_rpc.tls_cert_path,
        &config.json_rpc.tls_key_path,
        diem_db,
        mp_sender,
        config.base.role,
        chain_id,
    )
}

async fn health_check(
    params: HealthCheckParams,
    db: Arc<dyn DbReader>,
    now: SystemTime,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    if let Some(duration) = params.duration_secs {
        let ledger_info = db
            .get_latest_ledger_info()
            .map_err(|_| reject::custom(HealthCheckError))?;
        let timestamp = ledger_info.ledger_info().timestamp_usecs();

        check_latest_ledger_info_timestamp(duration, timestamp, now)
            .map_err(|_| reject::custom(HealthCheckError))?;
    }
    Ok(Box::new("diem-node:ok"))
}

pub fn check_latest_ledger_info_timestamp(
    duration_sec: u64,
    timestamp_usecs: u64,
    now: SystemTime,
) -> Result<()> {
    let timestamp = Duration::from_micros(timestamp_usecs);
    let expectation = now
        .sub(Duration::from_secs(duration_sec))
        .duration_since(UNIX_EPOCH)?;
    ensure!(timestamp >= expectation);
    Ok(())
}

/// JSON RPC entry point
/// Handles all incoming rpc requests
/// Performs routing based on methods defined in `registry`
pub(crate) async fn rpc_endpoint(
    data: Value,
    service: JsonRpcService,
    registry: Arc<RpcRegistry>,
    user_agent: Option<String>,
) -> Result<warp::reply::Response, warp::Rejection> {
    let label = match data {
        Value::Array(_) => LABEL_BATCH,
        _ => LABEL_SINGLE,
    };
    counters::RPC_REQUESTS.with_label_values(&[label]).inc();
    let timer = counters::RPC_REQUEST_LATENCY
        .with_label_values(&[label])
        .start_timer();
    let ret = rpc_endpoint_without_metrics(data, service, registry, user_agent.as_deref()).await;
    timer.stop_and_record();
    ret
}

async fn rpc_endpoint_without_metrics(
    data: Value,
    service: JsonRpcService,
    registry: Arc<RpcRegistry>,
    user_agent: Option<&str>,
) -> Result<warp::reply::Response, warp::Rejection> {
    // take snapshot of latest version of DB to be used across all requests, especially for batched requests
    let ledger_info = service
        .get_latest_ledger_info()
        .map_err(|_| reject::custom(DatabaseError))?;

    let mut rng = OsRng;
    let trace_id = format!("{:x}", rng.next_u64());
    debug!(RpcRequestLog {
        trace_id: trace_id.clone(),
        request: data.clone(),
    });
    let chain_id = service.chain_id();
    let latest_ledger_version = ledger_info.ledger_info().version();
    let latest_ledger_timestamp_usecs = ledger_info.ledger_info().timestamp_usecs();
    let sdk_info = sdk_info_from_user_agent(user_agent);

    let resp = Ok(if let Value::Array(requests) = data {
        match service.validate_batch_size_limit(requests.len()) {
            Ok(_) => {
                // batch API call
                let futures = requests.into_iter().map(|req| {
                    rpc_request_handler(
                        req,
                        service.clone(),
                        Arc::clone(&registry),
                        ledger_info.clone(),
                        LABEL_BATCH,
                        trace_id.clone(),
                        sdk_info,
                    )
                });
                let responses = join_all(futures).await;
                for resp in &responses {
                    log_response!(trace_id.clone(), &resp, true);
                }
                warp::reply::json(&responses)
            }
            Err(err) => {
                let mut resp = JsonRpcResponse::new(
                    chain_id,
                    latest_ledger_version,
                    latest_ledger_timestamp_usecs,
                );
                set_response_error(&mut resp, err, LABEL_BATCH, "unknown", sdk_info);
                log_response!(trace_id.clone(), &resp, true);

                warp::reply::json(&resp)
            }
        }
    } else {
        // single API call
        let resp = rpc_request_handler(
            data,
            service,
            registry,
            ledger_info,
            LABEL_SINGLE,
            trace_id.clone(),
            sdk_info,
        )
        .await;
        log_response!(trace_id, &resp, false);

        warp::reply::json(&resp)
    });

    let mut http_response = resp.into_response();
    let headers = http_response.headers_mut();

    headers.insert(
        X_DIEM_CHAIN_ID,
        header::HeaderValue::from_str(&chain_id.id().to_string()).unwrap(),
    );
    headers.insert(
        X_DIEM_VERSION_ID,
        header::HeaderValue::from_str(&latest_ledger_version.to_string()).unwrap(),
    );
    headers.insert(
        X_DIEM_TIMESTAMP_USEC_ID,
        header::HeaderValue::from_str(&latest_ledger_timestamp_usecs.to_string()).unwrap(),
    );

    Ok(http_response)
}

/// Handler of single RPC request
/// Performs validation and executes corresponding rpc handler
async fn rpc_request_handler(
    req: Value,
    service: JsonRpcService,
    registry: Arc<RpcRegistry>,
    ledger_info: LedgerInfoWithSignatures,
    request_type_label: &str,
    trace_id: String,
    sdk_info: SdkInfo,
) -> JsonRpcResponse {
    let request: Map<String, Value>;
    let mut response = JsonRpcResponse::new(
        service.chain_id(),
        ledger_info.ledger_info().version(),
        ledger_info.ledger_info().timestamp_usecs(),
    );

    match req {
        Value::Object(data) => {
            request = data;
        }
        _ => {
            set_response_error(
                &mut response,
                JsonRpcError::invalid_format(),
                request_type_label,
                "unknown",
                sdk_info,
            );
            return response;
        }
    }

    // parse request id
    match parse_request_id(&request) {
        Ok(request_id) => {
            response.id = Some(request_id);
        }
        Err(err) => {
            set_response_error(&mut response, err, request_type_label, "unknown", sdk_info);
            return response;
        }
    };

    // verify protocol version
    if let Err(err) = verify_protocol(&request) {
        set_response_error(&mut response, err, request_type_label, "unknown", sdk_info);
        return response;
    }

    // parse parameters
    let params = match request.get("params") {
        Some(Value::Array(parameters)) => parameters.to_vec(),
        _ => {
            vec![]
        }
    };

    let request_params = JsonRpcRequest {
        trace_id,
        ledger_info,
        params,
    };
    // get rpc handler
    match request.get("method") {
        Some(Value::String(name)) => match registry.get(name) {
            Some(handler) => {
                let timer = counters::METHOD_LATENCY
                    .with_label_values(&[request_type_label, name])
                    .start_timer();
                match handler(service, request_params).await {
                    Ok(result) => {
                        response.result = Some(result);
                        counters::REQUESTS
                            .with_label_values(&[
                                request_type_label,
                                name,
                                LABEL_SUCCESS,
                                sdk_info.language.as_str(),
                                &sdk_info.version.to_string(),
                            ])
                            .inc();
                    }
                    Err(err) => {
                        // check for custom error
                        set_response_error(
                            &mut response,
                            err.downcast_ref::<JsonRpcError>()
                                .cloned()
                                .unwrap_or_else(|| JsonRpcError::internal_error(err.to_string())),
                            request_type_label,
                            &name,
                            sdk_info,
                        );
                        counters::REQUESTS
                            .with_label_values(&[
                                request_type_label,
                                name,
                                LABEL_FAIL,
                                sdk_info.language.as_str(),
                                &sdk_info.version.to_string(),
                            ])
                            .inc();
                    }
                }
                timer.stop_and_record();
            }
            None => set_response_error(
                &mut response,
                JsonRpcError::method_not_found(),
                request_type_label,
                "not_found",
                sdk_info,
            ),
        },
        _ => set_response_error(
            &mut response,
            JsonRpcError::method_not_found(),
            request_type_label,
            "not_found",
            sdk_info,
        ),
    }

    response
}

// Sets the JSON RPC error value for a given response.
// If a counter label is supplied, also increments the invalid request counter using the label,
fn set_response_error(
    response: &mut JsonRpcResponse,
    error: JsonRpcError,
    request_type: &str,
    method: &str,
    sdk_info: SdkInfo,
) {
    let err_code = error.code;
    if is_internal_error(&error.code) {
        counters::INTERNAL_ERRORS
            .with_label_values(&[
                request_type,
                method,
                &err_code.to_string(),
                sdk_info.language.as_str(),
                &sdk_info.version.to_string(),
            ])
            .inc();
    } else {
        let label = match err_code {
            -32600 => "invalid_request",
            -32601 => "method_not_found",
            -32602 => "invalid_params",
            -32604 => "invalid_format",
            _ => "unexpected_code",
        };
        counters::INVALID_REQUESTS
            .with_label_values(&[
                request_type,
                method,
                label,
                sdk_info.language.as_str(),
                &sdk_info.version.to_string(),
            ])
            .inc();
    }

    response.error = Some(error);
}

fn parse_request_id(request: &Map<String, Value>) -> Result<Value, JsonRpcError> {
    match request.get("id") {
        Some(req_id) => {
            if req_id.is_string() || req_id.is_number() || req_id.is_null() {
                Ok(req_id.clone())
            } else {
                Err(JsonRpcError::invalid_format())
            }
        }
        None => Ok(Value::Null),
    }
}

fn verify_protocol(request: &Map<String, Value>) -> Result<(), JsonRpcError> {
    if let Some(Value::String(protocol)) = request.get("jsonrpc") {
        if protocol == "2.0" {
            return Ok(());
        }
    }
    Err(JsonRpcError::invalid_request())
}

/// Warp rejection types
#[derive(Debug)]
struct DatabaseError;

impl Reject for DatabaseError {}

#[derive(Debug)]
struct HealthCheckError;
impl warp::reject::Reject for HealthCheckError {}
