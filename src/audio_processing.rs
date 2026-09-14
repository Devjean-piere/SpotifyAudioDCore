use tokio::io::{AsyncReadExt};
use tokio::sync::mpsc;
use crate::structs::{AppState};
use crate::{BUFFER};
use crate::dsp::{band_eq, change_volume};
use crate::eq::BandEq;
use crate::fifo_helpers::*;
const SPOTIFY_SOURCE_FIFO: &str = "/tmp/spotify_source_fifo";


pub async fn audio(app_state: AppState) -> tokio::task::JoinHandle<()> {
    let mut l_band_eq = BandEq::new(44100.0);
    let mut r_band_eq = BandEq::new(44100.0);

    let (spotify_tx, mut spotify_rx) = mpsc::channel::<Vec<(i16, i16)>>(BUFFER);

    tokio::spawn(async move {
        start_fifo_reader(SPOTIFY_SOURCE_FIFO.to_string(), spotify_tx).await;
    });

    let (output_tx, output_rx) = mpsc::channel::<Vec<(i16, i16)>>(BUFFER);
    start_writer(output_rx).await;

    let source_state = app_state.source.clone();

    tokio::spawn(async move {
        loop {
            match spotify_rx.recv().await {
                Some(mut samples) => {
                    let current_source = source_state.read().await;

                    let vol = *app_state.volume.read().await;
                    change_volume(&mut *samples, vol);
                    let band_settings = *app_state.band_eq.read().await;
                    band_eq(&mut *samples, band_settings, &mut l_band_eq, &mut r_band_eq);

                    if current_source.to_string() == "Spotify" {
                        if output_tx.send(samples).await.is_err() {
                            break; // Snapcast-Writer ist weg
                        }
                    }
                }
                None => break,
            }
        }
    })
}