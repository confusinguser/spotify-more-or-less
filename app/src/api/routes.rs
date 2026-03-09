use crate::spotify::SpotifyClient;
use crate::storage::Storage;
use crate::storage::UserStorages;
use crate::types::{AlbumImageResponse, TrackInfo, TwoTracksResponse};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum_macros::debug_handler;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub(crate) fn router() -> axum::Router<(UserStorages, Arc<SpotifyClient>)> {
    let (app_router, openapi) = OpenApiRouter::new()
        .routes(routes!(get_users))
        .routes(routes!(get_random_track))
        .routes(routes!(get_two_random_tracks))
        .routes(routes!(get_album_image))
        .split_for_parts();
    app_router.route(
        "/openapi.json",
        axum::routing::get(move || async { Json(openapi) }),
    )
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct UserQuery {
    /// The user whose data to query. Omit to use the first available user.
    pub user: Option<String>,
}

/// Helper: get the storage for the given user, or the first user if none specified.
async fn resolve_user_storage(
    user_storages: &UserStorages,
    user: Option<&str>,
) -> Option<Arc<RwLock<Storage>>> {
    let map: tokio::sync::RwLockReadGuard<'_, HashMap<String, Arc<RwLock<Storage>>>> =
        user_storages.read().await;
    if let Some(name) = user {
        map.get(name).cloned()
    } else {
        map.values().next().cloned()
    }
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List of available users", body = Vec<String>),
    )
)]
pub async fn get_users(
    State((user_storages, _)): State<(UserStorages, Arc<SpotifyClient>)>,
) -> Json<Vec<String>> {
    let map: tokio::sync::RwLockReadGuard<'_, HashMap<String, Arc<RwLock<Storage>>>> =
        user_storages.read().await;
    let mut users: Vec<String> = map.keys().cloned().collect();
    users.sort();
    Json(users)
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/tracks/random",
    params(UserQuery),
    responses(
        (status = 200, description = "", body = TrackInfo),
    )
)]
pub async fn get_random_track(
    Query(params): Query<UserQuery>,
    State((user_storages, spotify_client)): State<(UserStorages, Arc<SpotifyClient>)>,
) -> Result<Json<TrackInfo>, StatusCode> {
    let storage = resolve_user_storage(&user_storages, params.user.as_deref())
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    // Try pre-fetch queue first, then fall back to a live Spotify fetch
    let (pre, personal) = {
        let s = storage.read().await;
        let pre = s.pop_prefetched().await;
        let personal = if pre.is_none() {
            s.pick_random_personal()
        } else {
            None
        };
        (pre, personal)
    };

    let track = if let Some(t) = pre {
        t
    } else if let Some(p) = personal {
        TrackInfo::from_personal_track_info(p, &spotify_client)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    // Refill in background
    let deficit = storage.read().await.prefetch_deficit().await;
    for _ in 0..deficit {
        let storage_clone = storage.clone();
        let sc = spotify_client.clone();
        tokio::spawn(async move {
            let personal = storage_clone.read().await.pick_random_personal();
            if let Some(personal) = personal {
                if let Ok(t) = TrackInfo::from_personal_track_info(personal, &sc).await {
                    storage_clone.read().await.push_prefetched(t).await;
                }
            }
        });
    }

    Ok(Json(track))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/tracks/random/two",
    params(UserQuery),
    responses(
        (status = 200, description = "Two random tracks", body = TwoTracksResponse),
    )
)]
pub async fn get_two_random_tracks(
    Query(params): Query<UserQuery>,
    State((user_storages, spotify_client)): State<(UserStorages, Arc<SpotifyClient>)>,
) -> Result<Json<TwoTracksResponse>, (StatusCode, String)> {
    let storage = resolve_user_storage(&user_storages, params.user.as_deref())
        .await
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    // Pop up to 2 pre-fetched tracks and pick personal infos for any misses —
    // all while only holding a read lock (the queue's own internal RwLock handles exclusion).
    let (pre1, pre2, personal1, personal2) = {
        let s = storage.read().await;
        let pre1 = s.pop_prefetched().await;
        let pre2 = s.pop_prefetched().await;
        let p1 = if pre1.is_none() {
            s.pick_random_personal()
        } else {
            None
        };
        let p2 = if pre2.is_none() {
            s.pick_random_personal()
        } else {
            None
        };
        (pre1, pre2, p1, p2)
    };

    // Fetch from Spotify concurrently for any pre-fetch misses
    let sc1 = spotify_client.clone();
    let sc2 = spotify_client.clone();
    let (track1, track2) = tokio::join!(
        async {
            if let Some(t) = pre1 {
                Ok(t)
            } else if let Some(p) = personal1 {
                TrackInfo::from_personal_track_info(p, &sc1).await
            } else {
                Err(crate::spotify::SpotifyError::ApiError(
                    "No tracks available".into(),
                ))
            }
        },
        async {
            if let Some(t) = pre2 {
                Ok(t)
            } else if let Some(p) = personal2 {
                TrackInfo::from_personal_track_info(p, &sc2).await
            } else {
                Err(crate::spotify::SpotifyError::ApiError(
                    "No tracks available".into(),
                ))
            }
        }
    );

    let track1 = track1.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let track2 = track2.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Refill pre-fetch queue in background — one task per deficit slot
    let deficit = storage.read().await.prefetch_deficit().await;
    for _ in 0..deficit {
        let storage_clone = storage.clone();
        let sc = spotify_client.clone();
        tokio::spawn(async move {
            let personal = storage_clone.read().await.pick_random_personal();
            if let Some(personal) = personal {
                if let Ok(track) = TrackInfo::from_personal_track_info(personal, &sc).await {
                    storage_clone.read().await.push_prefetched(track).await;
                }
            }
        });
    }

    Ok(Json(TwoTracksResponse { track1, track2 }))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/tracks/{track_id}/album-image",
    params(
        ("track_id" = String, Path, description = "Spotify track ID")
    ),
    responses(
        (status = 200, description = "Album image URL", body = AlbumImageResponse),
    )
)]
pub async fn get_album_image(
    Path(track_id): Path<String>,
    State((_, spotify_client)): State<(UserStorages, Arc<SpotifyClient>)>,
) -> Result<Json<AlbumImageResponse>, StatusCode> {
    let track = spotify_client
        .get_track(&track_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let image_url = track.album.images.first().map(|img| img.url.clone());

    Ok(Json(AlbumImageResponse {
        track_id,
        album_image_url: image_url,
    }))
}
