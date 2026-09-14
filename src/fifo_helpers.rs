use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use crate::FIFO_PATH;

fn bytes_to_stereo_samples(buffer: &[u8]) -> Vec<(i16, i16)> {
    buffer
        .chunks_exact(4)
        .map(|frame| {
            let left = i16::from_le_bytes([frame[0], frame[1]]);
            let right = i16::from_le_bytes([frame[2], frame[3]]);
            (left, right)
        })
        .collect()
}

fn stereo_samples_to_bytes(samples: &[(i16, i16)]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(samples.len() * 4);
    for (left, right) in samples {
        buffer.extend_from_slice(&left.to_le_bytes());
        buffer.extend_from_slice(&right.to_le_bytes());
    }
    buffer
}
pub async fn start_fifo_reader(fifo_path: String, tx: mpsc::Sender<Vec<(i16, i16)>>) {
    println!("Starte Blocking-Reader für FIFO: {fifo_path}");

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
                    let samples = bytes_to_stereo_samples(&buffer[..n]);
                    if !samples.is_empty() {
                        if rt.block_on(tx.send(samples)).is_err() {
                            break; // Channel zu, Empfänger weg
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Fehler beim Lesen aus FIFO: {e}");
                    break;
                }
            }
        }
    });

    if let Err(e) = handle.await {
        eprintln!("FIFO-Reader-Task ist abgestürzt: {e}");
    }
}

pub async fn start_fifo_writer(fifo_path: String, mut rx: mpsc::Receiver<Vec<(i16, i16)>>) {
    let mut file = match OpenOptions::new().write(true).open(&fifo_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Fehler beim Öffnen der Ausgabe-FIFO {fifo_path}: {e}");
            return;
        }
    };

    while let Some(samples) = rx.recv().await {
        let bytes = stereo_samples_to_bytes(&samples);
        if let Err(e) = file.write_all(&bytes).await {
            eprintln!("Fehler beim Schreiben in die FIFO: {e}");
            break;
        }
    }
}

pub async fn start_writer(rx: mpsc::Receiver<Vec<(i16, i16)>>) {
    tokio::spawn(async move {
        start_fifo_writer(FIFO_PATH.as_str().to_string(), rx).await;
    });
}
