use std::env;
use std::sync::Arc;
use librespot::connect::{ConnectConfig, Spirc};
use librespot::core::{Session, SessionConfig};
use librespot::core::cache::Cache;
use librespot::discovery::Credentials;
use librespot::playback::{audio_backend, mixer};
use librespot::playback::config::{AudioFormat, PlayerConfig};
use librespot::playback::mixer::MixerConfig;
use librespot::playback::player::Player;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl, TokenResponse};
use tokio::sync::oneshot;
use crate::{get_cache_path, CONNECT_NAME};
use crate::structs::{AppState};

pub async fn spotify(server_port: u16, app_state: AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cache_path = get_cache_path();
    let session_config = SessionConfig::default();
    let cache = Cache::new(Some(&cache_path), Some(&cache_path), Some(&cache_path), None)?;

    let credentials = if let Some(creds) = cache.credentials() {
        println!("Saved Spotify-Credentials Found.");
        creds
    } else if let Ok(token) = env::var("SPOTIFY_TOKEN") {
        println!("Using Spotify Token from environment variable.");
        Credentials::with_access_token(token)
    } else {
        println!("No Spotify-Credentials found. Run OAuth Login...");
        let redirect_uri = format!("http://127.0.0.1:{server_port}/login");

        let client = BasicClient::new(ClientId::new(session_config.client_id.clone()))
            .set_auth_uri(AuthUrl::new(
                "https://accounts.spotify.com/authorize".to_string(),
            )?)
            .set_token_uri(TokenUrl::new(
                "https://accounts.spotify.com/api/token".to_string(),
            )?)
            .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, _csrf) = client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(vec![Scope::new("streaming".to_string())])
            .set_pkce_challenge(pkce_challenge)
            .url();

        println!("Bitte im Browser einloggen: {auth_url}");

        let (tx, rx) = oneshot::channel();
        *app_state.login_sender.lock().await = Some(tx);

        let code = rx.await?;

        let http_client = reqwest::Client::new();
        let token_response = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(&http_client)
            .await?;

        println!("Spotify OAuth Successful.");
        Credentials::with_access_token(token_response.access_token().secret().to_string())
    };

    let session = Session::new(session_config, Some(cache));

    let audio_backend = audio_backend::find(Some("pipe".parse()?)).ok_or("No audio-backend found")?;
    let mixer_builder = mixer::find(None).ok_or("No Mixer found")?;
    let mixer = mixer_builder(MixerConfig::default())?;


    let player_config = PlayerConfig::default();
    let audio_format = AudioFormat::default();
    let player = Player::new(player_config, session.clone(), mixer.get_soft_volume(), move || {
        audio_backend(Some("/tmp/spotify_source_fifo".parse().unwrap()), audio_format)
    });

    let connect_config = ConnectConfig {
        name: CONNECT_NAME.clone(),
        ..Default::default()
    };


    let (spirc, spirc_task) = Spirc::new(connect_config, session, credentials, player, mixer).await?;

    let spirc = Arc::new(spirc);
    *app_state.spirc.write().await = Some(spirc.clone());
    

    println!("Spotify Connect Device Online! API bereit auf Port {server_port}.");
    

    spirc_task.await;



    Ok(())
}