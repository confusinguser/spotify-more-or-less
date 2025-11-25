use crate::storage::Storage;
use crate::storage::TrackInfo;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub(crate) fn router() -> axum::Router<Arc<Storage>> {
    let (app_router, openapi) = OpenApiRouter::new()
        .routes(routes!(get_random_track))
        .split_for_parts();
    app_router.route(
        "/openapi.json",
        axum::routing::get(move || async { Json(openapi) }),
    )
}

#[utoipa::path(
    get,
    path = "/tracks/random",
    responses(
        (status = 200, description = "", body = TrackInfo),
    )
)]
pub async fn get_random_track(
    State(storage): State<Arc<Storage>>,
) -> Result<Json<TrackInfo>, StatusCode> {
    storage
        .tracks
        .random_track()
        .map(|track| Json(track.clone()))
        .ok_or(StatusCode::NOT_FOUND)
}
