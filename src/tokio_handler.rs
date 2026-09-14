use std::sync::Arc;
use axum::extract::{ Query, State};
use axum::http::StatusCode;
use axum::Json;
use librespot::connect::Spirc;
use crate::{AppState, LoginQuery};
use crate::snap_cast::SnapcastClient;
use crate::structs::*;




pub async fn get_snap_cast_client(state: &AppState) -> Result<Arc<SnapcastClient>, StatusCode> {
    state
        .snapcast
        .read()
        .await
        .clone()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)
}
pub async fn get_spotify_spirc(state: &AppState) -> Result<Arc<Spirc>, StatusCode> {
    state
        .spirc
        .read()
        .await
        .clone()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)
}



pub async fn play_handler(State(state): State<AppState>) -> StatusCode {
    get_spotify_spirc(&state).await.unwrap().play();
    StatusCode::OK
}



pub async fn pause_handler(State(state): State<AppState>) -> StatusCode {
    get_spotify_spirc(&state).await.unwrap().pause();
    StatusCode::OK
}

pub async fn play_pause_handler(State(state): State<AppState>) -> StatusCode {
    get_spotify_spirc(&state).await.unwrap().play_pause();
    StatusCode::OK
}

pub async fn next_handler(State(state): State<AppState>) -> StatusCode {
    get_spotify_spirc(&state).await.unwrap().next();
    StatusCode::OK
}

pub async fn prev_handler(State(state): State<AppState>) -> StatusCode {
    let _ = get_spotify_spirc(&state).await.unwrap().prev();
    StatusCode::OK
}


//todo: make it to the dsp functionality
pub async fn set_volume_handler(
    State(state): State<AppState>,
    Query(params): Query<VolumeQuery>,
) -> StatusCode {
    let mut guard = state.volume.write().await;
    *guard = params.value;
    StatusCode::OK
}

pub async fn get_volume_handler(
    State(state): State<AppState>,
) -> Result<Json<VolumeResponse>, StatusCode> {
    let volume = *state.volume.read().await;
    Ok(Json(VolumeResponse { value: volume }))
}


pub async fn get_band_eq_handler(
    State(state): State<AppState>,
) -> Result<Json<BandEqGains>, StatusCode> {
    Ok(Json(state.band_eq.read().await.clone()))
}

pub async fn set_band_eq_handler(
    State(state): State<AppState>,
    Query(params): Query<BandEqGains>,
) -> StatusCode {
    let mut guard = state.band_eq.write().await;
    *guard = params;
    StatusCode::OK
}

pub async fn login_handler(
    State(state): State<AppState>,
    Query(params): Query<LoginQuery>,
) -> &'static str {
    match params.code {
        Some(code) => {
            if let Some(tx) = state.login_sender.lock().await.take() {
                let _ = tx.send(code);
            }
            "Login erfolgreich – du kannst dieses Tab jetzt schließen."
        }
        None => "Kein 'code'-Parameter im Redirect gefunden.",
    }
}


pub async fn set_client_volume_handler(
    State(state): State<AppState>,
    Query(parms): Query<ClientVolumeSetParams>,
) -> StatusCode {
    match get_snap_cast_client(&state).await {
        Ok(snapcast) => {
            let _ = snapcast.set_client_volume(&parms.client_id, parms.value as u8, &parms.muted);
            StatusCode::OK
        }
        Err(code) => code,
    }
}

pub async fn get_client_volume_handler(
    State(state): State<AppState>,
    Query(params): Query<ClientVolumeGetParams>,
) -> Result<Json<ClientVolumeGetResponse>, StatusCode> {
    let snapcast = get_snap_cast_client(&state)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let volume = snapcast
        .get_client_volume(&params.client_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ClientVolumeGetResponse { volume }))
}

pub async fn get_client_list_handler(
    State(state): State<AppState>,
) -> Result<Json<ClientListGetResponse>, StatusCode> {
    let snapcast = get_snap_cast_client(&state)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let clients = snapcast
        .get_client_ids()
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ClientListGetResponse { clients }))
}

pub async fn get_client_mute_handler(
    State(state): State<AppState>,
    Query(parms): Query<ClientVolumeSetParams>,
) -> Result<Json<ClientMutedGetResponse>, StatusCode> {
    let snapcast = get_snap_cast_client(&state)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let muted = snapcast.get_client_mute(&parms.client_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ClientMutedGetResponse { muted }))
}

pub async fn set_client_mute_handler(
    State(state): State<AppState>,
    Query(parms): Query<ClientVolumeSetParams>,
) -> StatusCode {
    match get_snap_cast_client(&state).await {
        Ok(snapcast) => {
            let _ = snapcast.set_client_mute(&parms.client_id, parms.muted);
            StatusCode::OK
        }
        Err(code) => code,
    }
}