use std::sync::Arc;
use librespot::connect::Spirc;
use librespot::playback::mixer::Mixer;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use crate::{snap_cast, LoginSender};


pub struct Arg {
    pub long: String,
    pub short: String,
}

#[derive(serde::Deserialize)]
pub struct LoginQuery {
    pub code: Option<String>,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSource {
    Spotify
}

#[derive(Clone)]
pub struct AppState {
    pub login_sender: LoginSender,
    pub spirc: Arc<RwLock<Option<Arc<Spirc>>>>,
    pub mixer: Arc<RwLock<Option<Arc<dyn Mixer>>>>,
    pub snapcast: Arc<RwLock<Option<Arc<snap_cast::SnapcastClient>>>>,
    pub source: Arc<RwLock<AudioSource>>,
}
#[derive(Deserialize)]
pub struct VolumeQuery {
    pub value: u16,
}
#[derive(Serialize)]
pub struct VolumeResponse {
    pub value: u16,
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