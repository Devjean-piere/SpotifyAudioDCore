use reqwest;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct SnapcastClient {
    client: reqwest::Client,
    base_url: String,
}

impl SnapcastClient {
    pub fn new(host_port: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: format!("http://{}/jsonrpc", host_port),
        }
    }

    async fn send_request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params.unwrap_or(json!({}))
        });

        let res = self.client
            .post(&self.base_url)
            .json(&payload)
            .send()
            .await?
            .json::<Value>()
            .await?;

        if let Some(error) = res.get("error") {
            return Err(format!("Snapcast JSON-RPC Fehler: {:?}", error).into());
        }

        Ok(res.get("result").cloned().unwrap_or(Value::Null))
    }

    // --- Status & Geräte-Abfragen ---

    pub async fn get_status(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.send_request("Server.GetStatus", None).await
    }

    /// Gibt die Gesamtzahl aller bekannten Snapcast-Clients zurück
    pub async fn get_client_count(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let status = self.get_status().await?;
        let mut count = 0;

        if let Some(server) = status.get("server") {
            if let Some(groups) = server.get("groups").and_then(|g| g.as_array()) {
                for group in groups {
                    if let Some(clients) = group.get("clients").and_then(|c| c.as_array()) {
                        count += clients.len();
                    }
                }
            }
        }
        Ok(count)
    }

    /// Gibt eine Liste aller Client-IDs (als String) zurück
    pub async fn get_client_ids(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let status = self.get_status().await?;
        let mut ids = Vec::new();

        if let Some(server) = status.get("server") {
            if let Some(groups) = server.get("groups").and_then(|g| g.as_array()) {
                for group in groups {
                    if let Some(clients) = group.get("clients").and_then(|c| c.as_array()) {
                        for client in clients {
                            if let Some(id) = client.get("id").and_then(|i| i.as_str()) {
                                ids.push(id.to_string());
                            }
                        }
                    }
                }
            }
        }
        Ok(ids)
    }

    // --- Steuerungs-Methoden ---

    pub async fn set_client_volume(
        &self,
        client_id: &str,
        percent: u8,
        muted: &Option<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let params = json!({
            "id": client_id,
            "volume": {
                "percent": percent,
                "muted": muted
            }
        });
        self.send_request("Client.SetVolume", Some(params)).await?;
        Ok(())
    }

    /// Schaltet einen spezifischen Client stumm oder hebt die Stummschaltung auf
    pub async fn set_client_mute(
        &self,
        client_id: &str,
        muted: Option<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let params = json!({
            "id": client_id,
            "volume": {
                "muted": muted
            }
        });
        self.send_request("Client.SetVolume", Some(params)).await?;
        Ok(())
    }

    /// Ändert nur die Lautstärke in Prozent, behält den Mute-Status bei (oder setzt ihn explizit)
    pub async fn set_client_volume_percent(
        &self,
        client_id: &str,
        percent: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let params = json!({
            "id": client_id,
            "volume": {
                "percent": percent
            }
        });
        self.send_request("Client.SetVolume", Some(params)).await?;
        Ok(())
    }

    /// Gibt den aktuellen Lautstärke-Prozentwert eines Clients zurück
    pub async fn get_client_volume(
        &self,
        client_id: &str,
    ) -> Result<u8, Box<dyn std::error::Error + Send + Sync>> {
        let status = self.get_status().await?;

        if let Some(server) = status.get("server") {
            if let Some(groups) = server.get("groups").and_then(|g| g.as_array()) {
                for group in groups {
                    if let Some(clients) = group.get("clients").and_then(|c| c.as_array()) {
                        for client in clients {
                            if client.get("id").and_then(|i| i.as_str()) == Some(client_id) {
                                if let Some(percent) = client
                                    .get("config")
                                    .and_then(|cfg| cfg.get("volume"))
                                    .and_then(|v| v.get("percent"))
                                    .and_then(|p| p.as_u64())
                                {
                                    return Ok(percent as u8);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(format!("Client mit ID {} nicht gefunden", client_id).into())
    }

    /// Gibt zurück, ob ein Client stummgeschaltet ist
    pub async fn get_client_mute(
        &self,
        client_id: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let status = self.get_status().await?;

        if let Some(server) = status.get("server") {
            if let Some(groups) = server.get("groups").and_then(|g| g.as_array()) {
                for group in groups {
                    if let Some(clients) = group.get("clients").and_then(|c| c.as_array()) {
                        for client in clients {
                            if client.get("id").and_then(|i| i.as_str()) == Some(client_id) {
                                if let Some(muted) = client
                                    .get("config")
                                    .and_then(|cfg| cfg.get("volume"))
                                    .and_then(|v| v.get("muted"))
                                    .and_then(|m| m.as_bool())
                                {
                                    return Ok(muted);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(format!("Client mit ID {} nicht gefunden", client_id).into())
    }
}