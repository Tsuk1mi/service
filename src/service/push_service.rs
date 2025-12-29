use serde::Serialize;

#[derive(Clone)]
pub struct PushService {
    pub fcm_server_key: Option<String>,
}

impl PushService {
    pub fn new(fcm_server_key: Option<String>) -> Self {
        Self { fcm_server_key }
    }

    pub async fn send_fcm(
        &self,
        token: &str,
        title: &str,
        body: &str,
        data: serde_json::Value,
    ) -> Result<(), String> {
        let key = match &self.fcm_server_key {
            Some(k) if !k.is_empty() => k,
            _ => return Ok(()), // Нет ключа — тихо выходим
        };

        #[derive(Serialize)]
        struct FcmMessage<'a> {
            to: &'a str,
            notification: Notification<'a>,
            data: serde_json::Value,
        }
        #[derive(Serialize)]
        struct Notification<'a> {
            title: &'a str,
            body: &'a str,
        }

        let payload = FcmMessage {
            to: token,
            notification: Notification { title, body },
            data,
        };

        let client = reqwest::Client::new();
        let res = client
            .post("https://fcm.googleapis.com/fcm/send")
            .header("Authorization", format!("key={}", key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(format!("FCM error: status {}", res.status()))
        }
    }
}

