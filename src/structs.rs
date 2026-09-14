use std::collections::HashMap;
use crate::{LoginSender, snap_cast};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use librespot::connect::Spirc;
use tokio::sync::RwLock;

pub struct Arg {
    pub long: String,
    pub short: String,
}

#[derive(serde::Deserialize)]
pub struct LoginQuery {
    pub code: Option<String>,
}


#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BandEqGains {
    pub high: f32,
    pub low: f32,
    pub mid: f32,
}

#[derive(Clone)]
pub struct AppState {
    pub login_sender: LoginSender,
    pub snapcast: Arc<RwLock<Option<Arc<snap_cast::SnapcastClient>>>>,
    pub source: Arc<RwLock<String>>,
    pub volume: Arc<RwLock<f32>>,
    pub spirc: Arc<RwLock<Option<Arc<Spirc>>>>,
    pub band_eq: Arc<RwLock<BandEqGains>>,
}
#[derive(Deserialize)]
pub struct VolumeQuery {
    pub value: f32,
}
#[derive(Serialize)]
pub struct VolumeResponse {
    pub value: f32,
}

#[derive(Deserialize)]
pub struct ClientVolumeSetParams {
    pub client_id: String,
    pub value: u8,
    pub muted: Option<bool>,
}

#[derive(Deserialize)]
pub struct ClientMuteSetParams {
    pub client_id: String,
    pub muted: bool,
}

#[derive(Deserialize)]
pub struct ClientVolumeGetParams {
    pub client_id: String,
}

#[derive(Serialize)]
pub struct ClientVolumeGetResponse {
    pub volume: u8,
}

#[derive(Serialize)]
pub struct ClientMutedGetResponse {
    pub muted: bool,
}

#[derive(Serialize)]
pub struct ClientListGetResponse {
    pub clients: Vec<String>,
}