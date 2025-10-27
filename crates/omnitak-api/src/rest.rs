//! REST API endpoints using Axum

use crate::auth::{AuthService, AuthUser, RequireAdmin, RequireOperator};
use crate::middleware::AuditLogger;
use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
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
    // TODO: Add actual service connections when implemented
    // pub pool_manager: Arc<PoolManager>,
    // pub filter_engine: Arc<FilterEngine>,
}

impl ApiState {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self {
            auth_service,
            audit_logger: Arc::new(AuditLogger::new()),
        }
    }
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
    State(_state): State<ApiState>,
    Query(_query): Query<ListQuery>,
    _user: AuthUser,
) -> Result<Json<ConnectionList>, ApiError> {
    // TODO: Get actual connections from pool manager
    let connections = vec![];

    Ok(Json(ConnectionList {
        total: connections.len(),
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
    State(_state): State<ApiState>,
    Path(id): Path<Uuid>,
    _user: AuthUser,
) -> Result<Json<ConnectionInfo>, ApiError> {
    // TODO: Get actual connection from pool manager
    Err(ApiError::NotFound(format!("Connection {} not found", id)))
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
    Json(request): Json<CreateConnectionRequest>,
) -> Result<(StatusCode, Json<CreateConnectionResponse>), ApiError> {
    // Validate request
    request.validate()?;

    info!(
        name = request.name,
        connection_type = ?request.connection_type,
        address = request.address,
        port = request.port,
        "Creating connection"
    );

    // TODO: Actually create connection in pool manager
    let connection_id = Uuid::new_v4();

    // Audit log
    state.audit_logger.log(
        user.0.user_id.unwrap_or_else(|| "api_key".to_string()),
        user.0.role,
        "create_connection".to_string(),
        format!("/api/v1/connections/{}", connection_id),
        serde_json::to_value(&request).unwrap(),
        "0.0.0.0".to_string(), // TODO: Get actual IP
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
) -> Result<Json<DeleteConnectionResponse>, ApiError> {
    info!(connection_id = %id, "Deleting connection");

    // TODO: Actually delete connection from pool manager

    // Audit log
    state.audit_logger.log(
        user.0.user_id.unwrap_or_else(|| "api_key".to_string()),
        user.0.role,
        "delete_connection".to_string(),
        format!("/api/v1/connections/{}", id),
        serde_json::json!({"connection_id": id}),
        "0.0.0.0".to_string(), // TODO: Get actual IP
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
