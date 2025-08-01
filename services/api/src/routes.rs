//! API service routes

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    middleware,
    response::IntoResponse,
    routing::{delete, get, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    AppState,
    error::ApiError,
    middleware::auth_middleware,
    models::{
        CreateUserRequest, SessionResponse, UserResponse,
        media::{MediaItem, MediaListResponse, MediaQuery, MediaRefreshRequest},
    },
};

/// Create the router for the API service
pub fn create_router(state: AppState) -> Router {
    let protected_routes = Router::new()
        .route("/protected", get(protected_route))
        .route("/media", get(get_media_items))
        .route("/media/:id", get(get_media_item))
        .route("/media/refresh", post(refresh_media))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .route("/health", get(health_check))
        .route("/users", post(create_user))
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
        .route("/sessions", get(get_sessions))
        .route("/sessions/:id", delete(delete_session))
        .merge(protected_routes)
        .with_state(state)
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "api-service"
    }))
}

/// Create a new user
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user = state.user_repository.create(&payload).await.map_err(|e| {
        tracing::error!("Failed to create user: {}", e);
        ApiError::InternalServerError
    })?;

    Ok((axum::http::StatusCode::CREATED, Json(user)))
}

/// Get all users
pub async fn get_users(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let users = state.user_repository.get_all().await.map_err(|e| {
        tracing::error!("Failed to get users: {}", e);
        ApiError::InternalServerError
    })?;

    Ok(Json(users))
}

/// Get a user by ID
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let user = state
        .user_repository
        .find_by_id(id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user: {}", e);
            ApiError::InternalServerError
        })?
        .ok_or(ApiError::BadRequest("User not found".to_string()))?;

    Ok(Json(user))
}

/// Get all sessions for the current user
pub async fn get_sessions(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    // In a real implementation, we would get the user ID from the JWT token
    // For now, we'll return an empty list
    let sessions: Vec<SessionResponse> = vec![];

    Ok(Json(sessions))
}

/// Get a media item by ID
pub async fn get_media_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let media_item = state
        .media_repository
        .get_by_id(id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get media item: {}", e);
            ApiError::InternalServerError
        })?
        .ok_or(ApiError::BadRequest("Media item not found".to_string()))?;

    Ok(Json(media_item))
}

/// Get media items with pagination, sorting, and filtering
pub async fn get_media_items(
    State(state): State<AppState>,
    Query(query): Query<MediaQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let (items, total) = state
        .media_repository
        .get_media_items(&query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get media items: {}", e);
            ApiError::InternalServerError
        })?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).min(100).max(1);

    let response = MediaListResponse {
        items,
        page,
        limit,
        total,
    };

    Ok(Json(response))
}

/// Refresh media library or specific media item
pub async fn refresh_media(
    State(_state): State<AppState>,
    Json(payload): Json<MediaRefreshRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // For now, we'll return a simple success response
    // In a real implementation, this would trigger a background process
    // to refresh the media library or specific media item

    let message = if payload.media_id.is_some() || payload.s3_key.is_some() {
        "Media refresh initiated for specific item"
    } else {
        "Media library refresh initiated"
    };

    Ok(Json(serde_json::json!({
        "message": message,
        "status": "success"
    })))
}

/// Protected route that requires authentication
pub async fn protected_route(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(json!({
        "message": "This is a protected route",
        "status": "success"
    })))
}

/// Delete a session by ID
pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state
        .session_repository
        .delete_session(id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete session: {}", e);
            ApiError::InternalServerError
        })?;

    if deleted {
        Ok(Json(json!({"message": "Session deleted successfully"})))
    } else {
        Err(ApiError::BadRequest("Session not found".to_string()))
    }
}
