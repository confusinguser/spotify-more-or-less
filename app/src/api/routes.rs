use crate::storage::Storage;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;
use axum_macros::debug_handler;
use tokio::sync::RwLock;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use crate::spotify::SpotifyClient;
use crate::types::TrackInfo;

pub(crate) fn router() -> axum::Router<(Arc<RwLock<Storage>>, Arc<SpotifyClient>)> {
    let (app_router, openapi) = OpenApiRouter::new()
        .routes(routes!(get_random_track))
        .split_for_parts();
    app_router.route(
        "/openapi.json",
        axum::routing::get(move || async { Json(openapi) }),
    )
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/tracks/random",
    responses(
        (status = 200, description = "", body = TrackInfo),
    )
)]
pub async fn get_random_track(
    State((storage, spotify_client)): State<(Arc<RwLock<Storage>>, Arc<SpotifyClient>)>,
) -> Result<Json<TrackInfo>, StatusCode> {
    storage.write().await
        .random_track(&spotify_client).await
        .map(|track| Json(track.clone()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
