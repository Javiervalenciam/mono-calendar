//! Descarga HTTPS directa a los endpoints de Google (constitución IV).
//! Sin proxies ni servidores intermedios; URL ofuscada en logs/errores.

use std::time::Duration;

/// Ofusca la parte personal de la URL ICS (principio IV).
pub fn redact_url(url: &str) -> String {
    match url.split_once("/calendar/") {
        Some((base, _)) => format!("{base}/calendar/…/<redacted>"),
        None => "…/<redacted>".into(),
    }
}

pub struct FetchResult {
    pub body: String,
    pub last_modified: Option<String>,
    pub not_modified: bool,
}

pub fn redacted_client_error(url: &str, status: u16) -> String {
    format!("El calendario respondió HTTP {status} ({})", redact_url(url))
}

/// Descarga el feed ICS. `timeout` 15 s (research R-04).
pub async fn fetch_ics(
    url: &str,
    last_modified: Option<&str>,
) -> Result<FetchResult, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("mono-calendar/0.1 (+ics reader)")
        .build()
        .map_err(|e| format!("No se pudo iniciar la descarga: {e}"))?;

    let mut req = client.get(url);
    if let Some(lm) = last_modified {
        if !lm.is_empty() {
            req = req.header("If-Modified-Since", lm);
        }
    }

    let resp = req
        .send()
        .await
        .map_err(|_e| format!("No hay respuesta del calendario ({})", redact_url(url)))?;

    let status = resp.status().as_u16();
    if status == 304 {
        return Ok(FetchResult {
            body: String::new(),
            last_modified: last_modified.map(|s| s.to_string()),
            not_modified: true,
        });
    }
    if !(200..300).contains(&status) {
        return Err(redacted_client_error(url, status));
    }

    let last_modified = resp
        .headers()
        .get("last-modified")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let body = resp
        .text()
        .await
        .map_err(|_| "La respuesta no se pudo leer como texto".to_string())?;

    if !body.contains("BEGIN:VCALENDAR") {
        return Err(format!(
            "La URL no es un calendario iCal válido ({})",
            redact_url(url)
        ));
    }

    Ok(FetchResult {
        body,
        last_modified,
        not_modified: false,
    })
}

/// Validación estática de URL (FR-002): HTTPS obligatorio.
pub fn validate_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if !trimmed.starts_with("https://") {
        return Err("La URL debe empezar por https:// (tráfico directo y cifrado)"
            .to_string());
    }
    if trimmed.len() < 12 {
        return Err("La URL parece incompleta".to_string());
    }
    Ok(())
}
