use axum::{
    Router,
    extract::Json,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use yt_down::downloader;

#[derive(Deserialize)]
struct DownloadRequest {
    url: String,
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

async fn health() -> &'static str {
    "ok"
}

async fn download(Json(payload): Json<DownloadRequest>) -> impl IntoResponse {
    let url = payload.url.trim().to_string();

    // Basic validation
    if url.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            HeaderMap::new(),
            b"Missing URL".to_vec(),
        );
    }

    if !url.contains("youtube.com") && !url.contains("youtu.be") {
        return (
            StatusCode::BAD_REQUEST,
            HeaderMap::new(),
            b"Invalid YouTube URL".to_vec(),
        );
    }

    // Create a unique temp directory for this download
    let download_id = Uuid::new_v4().to_string();
    let download_dir = format!("/tmp/yt_down_{}", download_id);
    std::fs::create_dir_all(&download_dir).unwrap_or_default();

    let output_template = format!("{}/%(title)s.%(ext)s", download_dir);

    let result = tokio::task::spawn_blocking(move || {
        let args = downloader::build_ytdlp_args(&url, &output_template);

        let output = std::process::Command::new("yt-dlp")
            .args(&args)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    // Find the downloaded MP3 file
                    if let Ok(entries) = std::fs::read_dir(&download_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.extension().is_some_and(|ext| ext == "mp3") {
                                let filename = path.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let data = std::fs::read(&path).unwrap_or_default();
                                let _ = std::fs::remove_dir_all(&download_dir);
                                return Ok((filename, data));
                            }
                        }
                    }
                    let _ = std::fs::remove_dir_all(&download_dir);
                    Err("Download completed but MP3 file not found".to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let _ = std::fs::remove_dir_all(&download_dir);
                    Err(format!("yt-dlp failed: {}", stderr))
                }
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(&download_dir);
                Err(format!("Failed to run yt-dlp: {}", e))
            }
        }
    })
    .await;

    match result {
        Ok(Ok((filename, data))) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("audio/mpeg"),
            );
            let disposition = format!("attachment; filename=\"{}\"", filename);
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&disposition).unwrap_or(
                    HeaderValue::from_static("attachment; filename=\"download.mp3\""),
                ),
            );
            (StatusCode::OK, headers, data)
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            HeaderMap::new(),
            e.into_bytes(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            HeaderMap::new(),
            format!("Task error: {}", e).into_bytes(),
        ),
    }
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🎵 yt_down web server starting on http://{}", addr);

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/download", post(download))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
