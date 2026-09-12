use bytes::Bytes;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

use crate::structs::{AppState, AudioSource};
use crate::FIFO_PATH;

const SPOTIFY_SOURCE_FIFO: &str = "/tmp/spotify_source_fifo";

async fn start_fifo_reader(fifo_path: String, tx: mpsc::Sender<Bytes>) {
    println!("Starte Blocking-Reader für FIFO: {fifo_path}");

    // FIFO muss existieren, bevor wir sie öffnen - librespot legt sie nicht selbst an.
    let _ = std::process::Command::new("mkfifo").arg(&fifo_path).status();

    let handle = tokio::task::spawn_blocking(move || {
        use std::fs::OpenOptions;
        use std::io::Read;

        let mut file = match OpenOptions::new().read(true).open(&fifo_path) {
            Ok(f) => {
                println!("FIFO erfolgreich synchron geöffnet!");
                f
            }
            Err(e) => {
                eprintln!("Fehler beim synchronen Öffnen der FIFO: {e}");
                return;
            }
        };

        let mut buffer = vec![0u8; 4096];
        let rt = tokio::runtime::Handle::current();

        loop {
            match file.read(&mut buffer) {
                Ok(0) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(n) => {
                    let chunk = Bytes::copy_from_slice(&buffer[..n]);
                    if rt.block_on(tx.send(chunk)).is_err() {
                        break; // Channel zu, Empfänger weg
                    }
                }
                Err(e) => {
                    eprintln!("Fehler beim Lesen aus FIFO: {e}");
                    break;
                }
            }
        }
    });

    // Panics/Fehler aus dem Blocking-Thread nicht mehr verschlucken
    if let Err(e) = handle.await {
        eprintln!("FIFO-Reader-Task ist abgestürzt: {e}");
    }
}

async fn start_fifo_writer(fifo_path: String, mut rx: mpsc::Receiver<Bytes>) {
    // Kein create(true)! Existiert die FIFO nicht, wollen wir einen klaren
    // Fehler statt einer versehentlich angelegten Regel-Datei.
    let mut file = match OpenOptions::new().write(true).open(&fifo_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Fehler beim Öffnen der Ausgabe-FIFO {fifo_path}: {e}");
            return;
        }
    };

    while let Some(chunk) = rx.recv().await {
        if let Err(e) = file.write_all(&chunk).await {
            eprintln!("Fehler beim Schreiben in die FIFO: {e}");
            break;
        }
    }
}

pub async fn start_writer(rx: mpsc::Receiver<Bytes>) {
    tokio::spawn(async move {
        start_fifo_writer(FIFO_PATH.as_str().to_string(), rx).await;
    });
}

pub async fn audio(app_state: AppState) {
    let (spotify_tx, mut spotify_rx) = mpsc::channel::<Bytes>(1);

    tokio::spawn(async move {
        start_fifo_reader(SPOTIFY_SOURCE_FIFO.to_string(), spotify_tx).await;
    });

    let (output_tx, output_rx) = mpsc::channel::<Bytes>(1);
    start_writer(output_rx).await;

    let source_state = app_state.source.clone();

    tokio::spawn(async move {
        loop {
            match spotify_rx.recv().await {
                Some(chunk) => {
                    let current_source = *source_state.read().await;

                    if current_source == AudioSource::Spotify {
                        if output_tx.send(chunk).await.is_err() {
                            break; // Snapcast-Writer ist weg
                        }
                    }
                }
                None => break,
            }
        }
    });
}