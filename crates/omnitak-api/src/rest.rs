//! REST API endpoints using Axum

use crate::auth::{AuthService, AuthUser, RequireAdmin, RequireOperator};
use crate::middleware::AuditLogger;
use crate::types::*;
use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use std::net::SocketAddr;
use omnitak_pool::{ConnectionPool, MessageDistributor, PoolMessage, FilterRule};
use omnitak_client::{
    ClientConfig, CotMessage, ReconnectConfig, TakClient,
    tcp::{TcpClient, TcpClientConfig, FramingMode},
    tls::{TlsClient, TlsClientConfig},
    Bytes,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing::{error, info};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone)]
pub struct ApiState {
    pub auth_service: Arc<AuthService>,
    pub audit_logger: Arc<AuditLogger>,
    pub pool: Arc<ConnectionPool>,
    pub distributor: Arc<MessageDistributor>,
    pub connections: Arc<RwLock<Vec<ConnectionInfo>>>,
}

// ============================================================================
// Router Setup
// ============================================================================

pub fn create_rest_router(state: ApiState) -> Router {
    Router::new()
        // System endpoints
        .route("/api/v1/status", get(get_system_status))
        .route("/api/v1/health", get(health_check))
        // Connection management
        .route("/api/v1/connections", get(list_connections))
        .route("/api/v1/connections", post(create_connection))
        .route("/api/v1/connections/:id", get(get_connection))
        .route("/api/v1/connections/:id", delete(delete_connection))
        // Filter management
        .route("/api/v1/filters", get(list_filters))
        .route("/api/v1/filters", post(create_filter))
        .route("/api/v1/filters/:id", get(get_filter))
        .route("/api/v1/filters/:id", delete(delete_filter))
        // Metrics
        .route("/api/v1/metrics", get(get_metrics))
        // Authentication
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/api-keys", post(create_api_key))
        // Audit logs (admin only)
        .route("/api/v1/audit", get(get_audit_logs))
        .with_state(state)
}

// ============================================================================
// System Status Endpoints
// ============================================================================

/// GET /api/v1/status - Get overall system status
#[utoipa::path(
    get,
    path = "/api/v1/status",
    responses(
        (status = 200, description = "System status retrieved successfully", body = SystemStatus),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn get_system_status(
    State(_state): State<ApiState>,
    _user: AuthUser,
) -> Result<Json<SystemStatus>, ApiError> {
    // TODO: Get actual system metrics
    let status = SystemStatus {
        uptime_seconds: 3600,
        active_connections: 5,
        messages_processed: 12345,
        messages_per_second: 10.5,
        memory_usage_bytes: 50 * 1024 * 1024,
        active_filters: 3,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now(),
    };

    Ok(Json(status))
}

/// GET /api/v1/health - Health check endpoint (no auth required)
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
    }))
}

// ============================================================================
// Connection Management Endpoints
// ============================================================================

#[derive(Debug, Deserialize)]
struct ListQuery {
    #[serde(default)]
    offset: usize,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    100
}

/// GET /api/v1/connections - List all connections
#[utoipa::path(
    get,
    path = "/api/v1/connections",
    params(
        ("offset" = Option<usize>, Query, description = "Pagination offset"),
        ("limit" = Option<usize>, Query, description = "Pagination limit")
    ),
    responses(
        (status = 200, description = "Connections list retrieved successfully", body = ConnectionList),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn list_connections(
    State(state): State<ApiState>,
    Query(query): Query<ListQuery>,
    _user: AuthUser,
) -> Result<Json<ConnectionList>, ApiError> {
    // Get connections from state
    let all_connections = state.connections.read().await;
    let total = all_connections.len();

    // Apply pagination
    let connections: Vec<ConnectionInfo> = all_connections
        .iter()
        .skip(query.offset)
        .take(query.limit)
        .cloned()
        .collect();

    Ok(Json(ConnectionList {
        total,
        connections,
    }))
}

/// GET /api/v1/connections/:id - Get specific connection details
#[utoipa::path(
    get,
    path = "/api/v1/connections/{id}",
    params(
        ("id" = Uuid, Path, description = "Connection ID")
    ),
    responses(
        (status = 200, description = "Connection details retrieved successfully", body = ConnectionInfo),
        (status = 404, description = "Connection not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn get_connection(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    _user: AuthUser,
) -> Result<Json<ConnectionInfo>, ApiError> {
    // Get connection from state by ID
    let connections = state.connections.read().await;
    let connection = connections
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or_else(|| ApiError::NotFound(format!("Connection {} not found", id)))?;

    Ok(Json(connection))
}

/// POST /api/v1/connections - Create new connection
#[utoipa::path(
    post,
    path = "/api/v1/connections",
    request_body = CreateConnectionRequest,
    responses(
        (status = 201, description = "Connection created successfully", body = CreateConnectionResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - requires operator role", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn create_connection(
    State(state): State<ApiState>,
    RequireOperator(user): RequireOperator,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    Json(request): Json<CreateConnectionRequest>,
) -> Result<(StatusCode, Json<CreateConnectionResponse>), ApiError> {
    // Validate request
    request.validate()?;

    let connection_id = Uuid::new_v4();
    let id_str = connection_id.to_string();
    let address_with_port = format!("{}:{}", request.address, request.port);

    info!(
        id = %connection_id,
        name = request.name,
        connection_type = ?request.connection_type,
        address = %address_with_port,
        "Creating connection"
    );

    // Add connection to pool
    state.pool.add_connection(
        id_str.clone(),
        request.name.clone(),
        address_with_port.clone(),
        5, // Default priority
    ).await.map_err(|e| {
        error!(id = %connection_id, error = %e, "Failed to add connection to pool");
        ApiError::InternalError(format!("Failed to add connection: {}", e))
    })?;

    // Get the connection's channels from pool
    let connection = state.pool.get_connection(&id_str)
        .ok_or_else(|| ApiError::InternalError("Connection not found in pool after creation".to_string()))?;

    let pool_tx = connection.tx.clone();
    let pool_rx = connection.rx.clone();

    // Add filter for this connection
    state.distributor.add_filter(id_str.clone(), FilterRule::AlwaysSend);

    // Spawn client task based on connection type
    let address_clone = address_with_port.clone();
    let id_clone = id_str.clone();
    let auto_reconnect = request.auto_reconnect;

    match request.connection_type {
        ConnectionType::TcpClient => {
            info!(id = %connection_id, "Creating TCP client");

            let config = TcpClientConfig {
                base: ClientConfig {
                    server_addr: address_with_port.clone(),
                    connect_timeout: Duration::from_secs(10),
                    read_timeout: Duration::from_secs(30),
                    write_timeout: Duration::from_secs(10),
                    recv_buffer_size: 65536,
                    reconnect: ReconnectConfig {
                        enabled: auto_reconnect,
                        max_attempts: Some(5),
                        initial_backoff: Duration::from_secs(1),
                        max_backoff: Duration::from_secs(60),
                        backoff_multiplier: 2.0,
                    },
                },
                framing: FramingMode::Xml,
                keepalive: true,
                keepalive_interval: Some(Duration::from_secs(30)),
                nagle: false,
            };

            let mut client = TcpClient::new(config);

            // Spawn client task
            tokio::spawn(async move {
                info!(id = %id_clone, "Connecting TCP client");

                if let Err(e) = client.connect().await {
                    error!(id = %id_clone, error = %e, "Failed to connect TCP client");
                    return;
                }

                info!(id = %id_clone, address = %address_clone, "TCP client connected");

                let mut recv_stream = client.receive_cot();
                let client_arc = Arc::new(tokio::sync::Mutex::new(client));
                let client_write = Arc::clone(&client_arc);

                let id_read = id_clone.clone();
                let id_write = id_clone.clone();

                // Read task (TAK server → Pool)
                let read_task = tokio::spawn(async move {
                    while let Some(result) = recv_stream.next().await {
                        match result {
                            Ok(cot_msg) => {
                                info!(id = %id_read, bytes = cot_msg.data.len(), "Received CoT message");
                                if let Err(e) = pool_tx.send_async(PoolMessage::Cot(cot_msg.data.to_vec())).await {
                                    error!(id = %id_read, error = %e, "Failed to send to pool");
                                    break;
                                }
                            }
                            Err(e) => {
                                error!(id = %id_read, error = %e, "Error reading from TAK server");
                                break;
                            }
                        }
                    }
                    info!(id = %id_read, "TCP read task terminated");
                });

                // Write task (Pool → TAK server)
                let write_task = tokio::spawn(async move {
                    loop {
                        match pool_rx.recv_async().await {
                            Ok(PoolMessage::Cot(data)) => {
                                let cot_msg = CotMessage {
                                    data: Bytes::from(data),
                                    metadata: Default::default(),
                                };
                                let mut client = client_write.lock().await;
                                if let Err(e) = client.send_cot(cot_msg).await {
                                    error!(id = %id_write, error = %e, "Failed to send to TAK server");
                                    break;
                                }
                            }
                            Ok(PoolMessage::Shutdown) => {
                                info!(id = %id_write, "Shutdown signal received");
                                break;
                            }
                            Ok(PoolMessage::Ping) => continue,
                            Err(e) => {
                                error!(id = %id_write, error = %e, "Pool channel error");
                                break;
                            }
                        }
                    }
                    info!(id = %id_write, "TCP write task terminated");
                });

                tokio::select! {
                    _ = read_task => {}
                    _ = write_task => {}
                }
            });
        }
        ConnectionType::TlsClient => {
            info!(id = %connection_id, "Creating TLS client");

            // Validate TLS cert paths are provided
            let cert_path = request.tls_cert_path
                .ok_or_else(|| ApiError::BadRequest("TLS certificate path required for TLS connection".to_string()))?;
            let key_path = request.tls_key_path
                .ok_or_else(|| ApiError::BadRequest("TLS key path required for TLS connection".to_string()))?;

            let mut client_config = TlsClientConfig::new(
                std::path::PathBuf::from(cert_path),
                std::path::PathBuf::from(key_path),
            );
            client_config.base.server_addr = address_with_port.clone();
            client_config.base.connect_timeout = Duration::from_secs(10);
            client_config.base.read_timeout = Duration::from_secs(30);
            client_config.base.write_timeout = Duration::from_secs(10);
            client_config.base.recv_buffer_size = 65536;
            client_config.base.reconnect = ReconnectConfig {
                enabled: auto_reconnect,
                max_attempts: Some(5),
                initial_backoff: Duration::from_secs(1),
                max_backoff: Duration::from_secs(60),
                backoff_multiplier: 2.0,
            };
            client_config.verify_server = request.validate_certs;

            let mut client = TlsClient::new(client_config)
                .map_err(|e| ApiError::InternalError(format!("Failed to create TLS client: {}", e)))?;

            // Spawn client task (similar pattern to TCP)
            tokio::spawn(async move {
                info!(id = %id_clone, "Connecting TLS client");

                if let Err(e) = client.connect().await {
                    error!(id = %id_clone, error = %e, "Failed to connect TLS client");
                    return;
                }

                info!(id = %id_clone, address = %address_clone, "TLS client connected");

                let mut recv_stream = client.receive_cot();
                let client_arc = Arc::new(tokio::sync::Mutex::new(client));
                let client_write = Arc::clone(&client_arc);

                let id_read = id_clone.clone();
                let id_write = id_clone.clone();

                let read_task = tokio::spawn(async move {
                    while let Some(result) = recv_stream.next().await {
                        match result {
                            Ok(cot_msg) => {
                                info!(id = %id_read, bytes = cot_msg.data.len(), "Received CoT message (TLS)");
                                if let Err(e) = pool_tx.send_async(PoolMessage::Cot(cot_msg.data.to_vec())).await {
                                    error!(id = %id_read, error = %e, "Failed to send to pool");
                                    break;
                                }
                            }
                            Err(e) => {
                                error!(id = %id_read, error = %e, "Error reading from TLS TAK server");
                                break;
                            }
                        }
                    }
                    info!(id = %id_read, "TLS read task terminated");
                });

                let write_task = tokio::spawn(async move {
                    loop {
                        match pool_rx.recv_async().await {
                            Ok(PoolMessage::Cot(data)) => {
                                let cot_msg = CotMessage {
                                    data: Bytes::from(data),
                                    metadata: Default::default(),
                                };
                                let mut client = client_write.lock().await;
                                if let Err(e) = client.send_cot(cot_msg).await {
                                    error!(id = %id_write, error = %e, "Failed to send to TLS TAK server");
                                    break;
                                }
                            }
                            Ok(PoolMessage::Shutdown) => {
                                info!(id = %id_write, "Shutdown signal received");
                                break;
                            }
                            Ok(PoolMessage::Ping) => continue,
                            Err(e) => {
                                error!(id = %id_write, error = %e, "Pool channel error");
                                break;
                            }
                        }
                    }
                    info!(id = %id_write, "TLS write task terminated");
                });

                tokio::select! {
                    _ = read_task => {}
                    _ = write_task => {}
                }
            });
        }
        _ => {
            return Err(ApiError::BadRequest(format!(
                "Connection type {:?} not yet supported",
                request.connection_type
            )));
        }
    }

    // Store connection info
    let conn_info = ConnectionInfo {
        id: connection_id,
        name: request.name.clone(),
        connection_type: request.connection_type,
        status: ConnectionStatus::Connecting,
        address: request.address.clone(),
        port: request.port,
        messages_received: 0,
        messages_sent: 0,
        bytes_received: 0,
        bytes_sent: 0,
        connected_at: None,
        last_activity: None,
        error: None,
    };

    let mut connections = state.connections.write().await;
    connections.push(conn_info);
    drop(connections);

    // Audit log with actual client IP
    state.audit_logger.log(
        user.0.user_id.unwrap_or_else(|| "api_key".to_string()),
        user.0.role,
        "create_connection".to_string(),
        format!("/api/v1/connections/{}", connection_id),
        serde_json::to_value(&request).unwrap(),
        client_addr.ip().to_string(),
        true,
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateConnectionResponse {
            id: connection_id,
            message: "Connection created successfully".to_string(),
        }),
    ))
}

/// DELETE /api/v1/connections/:id - Remove connection
#[utoipa::path(
    delete,
    path = "/api/v1/connections/{id}",
    params(
        ("id" = Uuid, Path, description = "Connection ID")
    ),
    responses(
        (status = 200, description = "Connection deleted successfully", body = DeleteConnectionResponse),
        (status = 404, description = "Connection not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - requires operator role", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn delete_connection(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    RequireOperator(user): RequireOperator,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<DeleteConnectionResponse>, ApiError> {
    let id_str = id.to_string();
    info!(connection_id = %id, "Deleting connection");

    // Remove from connection pool
    state.pool.remove_connection(&id_str).await
        .map_err(|e| {
            error!(connection_id = %id, error = %e, "Failed to remove connection from pool");
            ApiError::InternalError(format!("Failed to remove connection: {}", e))
        })?;

    // Remove from state tracking
    let mut connections = state.connections.write().await;
    let initial_len = connections.len();
    connections.retain(|c| c.id != id);

    if connections.len() == initial_len {
        return Err(ApiError::NotFound(format!("Connection {} not found", id)));
    }
    drop(connections);

    // Remove filter
    state.distributor.remove_filter(&id_str);

    info!(connection_id = %id, "Connection deleted successfully");

    // Audit log with actual client IP
    state.audit_logger.log(
        user.0.user_id.unwrap_or_else(|| "api_key".to_string()),
        user.0.role,
        "delete_connection".to_string(),
        format!("/api/v1/connections/{}", id),
        serde_json::json!({"connection_id": id}),
        client_addr.ip().to_string(),
        true,
    );

    Ok(Json(DeleteConnectionResponse {
        message: "Connection deleted successfully".to_string(),
    }))
}

// ============================================================================
// Filter Management Endpoints
// ============================================================================

/// GET /api/v1/filters - List all filters
#[utoipa::path(
    get,
    path = "/api/v1/filters",
    responses(
        (status = 200, description = "Filters list retrieved successfully", body = FilterList),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn list_filters(
    State(_state): State<ApiState>,
    _user: AuthUser,
) -> Result<Json<FilterList>, ApiError> {
    // TODO: Get actual filters from filter engine
    let filters = vec![];

    Ok(Json(FilterList {
        total: filters.len(),
        filters,
    }))
}

/// GET /api/v1/filters/:id - Get specific filter
async fn get_filter(
    State(_state): State<ApiState>,
    Path(id): Path<Uuid>,
    _user: AuthUser,
) -> Result<Json<FilterRule>, ApiError> {
    // TODO: Get actual filter from filter engine
    Err(ApiError::NotFound(format!("Filter {} not found", id)))
}

/// POST /api/v1/filters - Create new filter
#[utoipa::path(
    post,
    path = "/api/v1/filters",
    request_body = CreateFilterRequest,
    responses(
        (status = 201, description = "Filter created successfully", body = CreateFilterResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - requires operator role", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn create_filter(
    State(state): State<ApiState>,
    RequireOperator(user): RequireOperator,
    Json(request): Json<CreateFilterRequest>,
) -> Result<(StatusCode, Json<CreateFilterResponse>), ApiError> {
    // Validate request
    request.validate()?;

    info!(
        name = request.name,
        action = ?request.action,
        priority = request.priority,
        "Creating filter"
    );

    // TODO: Actually create filter in filter engine
    let filter_id = Uuid::new_v4();

    // Audit log
    state.audit_logger.log(
        user.0.user_id.unwrap_or_else(|| "api_key".to_string()),
        user.0.role,
        "create_filter".to_string(),
        format!("/api/v1/filters/{}", filter_id),
        serde_json::to_value(&request).unwrap(),
        "0.0.0.0".to_string(), // TODO: Get actual IP
        true,
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateFilterResponse {
            id: filter_id,
            message: "Filter created successfully".to_string(),
        }),
    ))
}

/// DELETE /api/v1/filters/:id - Remove filter
async fn delete_filter(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    RequireOperator(user): RequireOperator,
) -> Result<Json<DeleteConnectionResponse>, ApiError> {
    info!(filter_id = %id, "Deleting filter");

    // TODO: Actually delete filter from filter engine

    // Audit log
    state.audit_logger.log(
        user.0.user_id.unwrap_or_else(|| "api_key".to_string()),
        user.0.role,
        "delete_filter".to_string(),
        format!("/api/v1/filters/{}", id),
        serde_json::json!({"filter_id": id}),
        "0.0.0.0".to_string(), // TODO: Get actual IP
        true,
    );

    Ok(Json(DeleteConnectionResponse {
        message: "Filter deleted successfully".to_string(),
    }))
}

// ============================================================================
// Metrics Endpoint
// ============================================================================

/// GET /api/v1/metrics - Prometheus metrics
#[utoipa::path(
    get,
    path = "/api/v1/metrics",
    responses(
        (status = 200, description = "Metrics retrieved successfully", content_type = "text/plain"),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn get_metrics(
    State(_state): State<ApiState>,
    _user: AuthUser,
) -> Result<String, ApiError> {
    // TODO: Export actual Prometheus metrics
    Ok(format!(
        "# HELP omnitak_connections_total Total number of connections\n\
         # TYPE omnitak_connections_total gauge\n\
         omnitak_connections_total 5\n\
         \n\
         # HELP omnitak_messages_processed_total Total messages processed\n\
         # TYPE omnitak_messages_processed_total counter\n\
         omnitak_messages_processed_total 12345\n"
    ))
}

// ============================================================================
// Authentication Endpoints
// ============================================================================

/// POST /api/v1/auth/login - User login
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    )
)]
async fn login(
    State(state): State<ApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Validate request
    request.validate()?;

    // Attempt login
    let (access_token, expires_at) = state
        .auth_service
        .login(&request.username, &request.password)
        .map_err(|_| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    // Get user role
    let user = state
        .auth_service
        .users
        .get(&request.username)
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    let role = user.role;

    info!(username = request.username, "User logged in successfully");

    Ok(Json(LoginResponse {
        access_token,
        expires_at,
        role,
    }))
}

/// POST /api/v1/auth/api-keys - Create API key (admin only)
#[utoipa::path(
    post,
    path = "/api/v1/auth/api-keys",
    request_body = ApiKeyRequest,
    responses(
        (status = 201, description = "API key created successfully", body = ApiKeyResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - requires admin role", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn create_api_key(
    State(state): State<ApiState>,
    RequireAdmin(user): RequireAdmin,
    Json(request): Json<ApiKeyRequest>,
) -> Result<(StatusCode, Json<ApiKeyResponse>), ApiError> {
    // Validate request
    request.validate()?;

    // Create API key
    let (api_key, key_id) = state
        .auth_service
        .create_api_key(request.name.clone(), request.role, request.expires_at)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    info!(
        key_name = request.name,
        key_role = ?request.role,
        "API key created"
    );

    // Audit log
    state.audit_logger.log(
        user.0.user_id.unwrap_or_else(|| "api_key".to_string()),
        user.0.role,
        "create_api_key".to_string(),
        format!("/api/v1/auth/api-keys/{}", key_id),
        serde_json::to_value(&request).unwrap(),
        "0.0.0.0".to_string(), // TODO: Get actual IP
        true,
    );

    Ok((
        StatusCode::CREATED,
        Json(ApiKeyResponse {
            api_key,
            id: key_id,
            name: request.name,
            created_at: Utc::now(),
        }),
    ))
}

// ============================================================================
// Audit Log Endpoints
// ============================================================================

/// GET /api/v1/audit - Get audit logs (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/audit",
    responses(
        (status = 200, description = "Audit logs retrieved successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - requires admin role", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn get_audit_logs(
    State(state): State<ApiState>,
    RequireAdmin(_user): RequireAdmin,
) -> Result<Json<Vec<AuditLogEntry>>, ApiError> {
    let logs = state.audit_logger.get_logs();
    Ok(Json(logs))
}

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_code, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            ApiError::InternalError(msg) => {
                error!(error = %msg, "Internal API error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred".to_string(),
                )
            }
            ApiError::ValidationError(errors) => {
                let message = format!("Validation failed: {}", errors);
                (StatusCode::BAD_REQUEST, "validation_error", message)
            }
        };

        let body = Json(ErrorResponse::new(error_code, message));
        (status, body).into_response()
    }
}
