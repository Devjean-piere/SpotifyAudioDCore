mod snap_cast;
mod structs;
mod tokio_handler;
mod spotify;
mod audio_processing;
mod eq;
mod compressor;
mod dsp;
mod fifo_helpers;

use crate::tokio_handler::*;
use crate::structs::*;
use crate::audio_processing::audio;
use crate::spotify::spotify;

use std::env;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use axum::{routing::{get, post}, Router};
use tokio::sync::{oneshot, Mutex, RwLock};
use crate::snap_cast::SnapcastClient;

fn find_arg(arg: &Arg) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        if (args[i] == arg.long || args[i] == arg.short) && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

pub const SERVER_PORT: u16 = 8080;
pub const BUFFER: usize = 1;
pub static FIFO_PATH: LazyLock<String> = LazyLock::new(|| {
    let config = Arg { long: "--fifo".to_string(), short: "-f".to_string() };
    find_arg(&config)
        .or_else(|| env::var("FIFO_PATH").ok())
        .unwrap_or_else(|| "/tmp/spotify_fifo".to_string())
});

pub static CONNECT_NAME: LazyLock<String> = LazyLock::new(|| {
    let config = Arg { long: "--connect-name".to_string(), short: "-n".to_string() };
    find_arg(&config)
        .or_else(|| env::var("CONNECT_NAME").ok())
        .unwrap_or_else(|| "spotify_connect".to_string())
});

type LoginSender = Arc<Mutex<Option<oneshot::Sender<String>>>>;

pub fn get_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("spotifyAudioD")
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = env_logger::Builder::new();
    logger.parse_filters("librespot=info,librespot_core=info,librespot_connect=info");
    logger.init();

    let _ = std::process::Command::new("mkfifo").arg(&*FIFO_PATH).status();

    std::fs::create_dir_all(&get_cache_path())?;
    println!("Cache: {}", get_cache_path().display());

    let app_state = AppState {
        login_sender: Arc::new(Mutex::new(None)),
        snapcast: Arc::new(RwLock::new(None)),
        source: Arc::new(RwLock::new("Spotify".parse()?)),
        volume: Arc::new(RwLock::new(1.0)),
        spirc: Arc::new(RwLock::new(None)),
        band_eq: Arc::new(RwLock::new(BandEqGains {low: 1.0, mid: 1.0, high: 1.0}))
    };



    let playback_routes = Router::new()
        .route("/play", post(play_handler))
        .route("/pause", post(pause_handler))
        .route("/playpause", post(play_pause_handler))
        .route("/next", post(next_handler))
        .route("/prev", post(prev_handler))
        .route("/volume/set", post(set_volume_handler))
        .route("/volume/get", get(get_volume_handler))
        .route("/band_eq/get", get(get_band_eq_handler))
        .route("/band_eq/set", post(set_band_eq_handler));

    let client_routes = Router::new()
        .route("/volume/set", post(set_client_volume_handler))
        .route("/volume/get", get(get_client_volume_handler))
        .route("/list", get(get_client_list_handler))
        .route("/mute/set", post(set_client_mute_handler))
        .route("/mute/get", post(get_client_mute_handler));

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/login", get(login_handler))
        .nest("/playback", playback_routes)
        .nest("/client", client_routes)
        .with_state(app_state.clone());

    // Webserver im Hintergrund starten
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", SERVER_PORT)).await?;
    let axum_thread = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    println!("Axum Webserver läuft auf Port {SERVER_PORT}.");

    let spotify_state = app_state.clone();
    let spotify_thread = tokio::spawn(async move {
        if let Err(e) = spotify(SERVER_PORT, spotify_state).await {
            eprintln!("Fehler im Spotify-Task: {:?}", e);
        }
    });

    let snapcast_client = SnapcastClient::new("127.0.0.1:1780");
    *app_state.snapcast.write().await = Some(Arc::from(snapcast_client));

    let audio_thread = audio(app_state).await;

    tokio::select! {
    _ = tokio::signal::ctrl_c() => {
        println!("Shutdown-Signal empfangen, räume auf...");
    }
    res = axum_thread => {
        eprintln!("Axum-Task ist unerwartet beendet: {res:?}");
    }
    res = spotify_thread => {
        eprintln!("Spotify-Task ist unerwartet beendet: {res:?}");
    }
    res = audio_thread => {
        eprintln!("Audio-Task ist unerwartet beendet: {res:?}");
    }
    }

    let _ = std::process::Command::new("rm").arg("-rf").arg(&*FIFO_PATH).status();

    Ok(())
}