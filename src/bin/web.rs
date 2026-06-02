use axum::{
    body::Body,
    extract::Json,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use tokio_util::io::ReaderStream;
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

async fn download(Json(payload): Json<DownloadRequest>) -> Response {
    let url = payload.url.trim().to_string();

    if url.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing URL").into_response();
    }

    if !url.contains("youtube.com") && !url.contains("youtu.be") {
        return (StatusCode::BAD_REQUEST, "Invalid YouTube URL").into_response();
    }

    let download_id = Uuid::new_v4().to_string();
    let download_dir = format!("/tmp/yt_down_{}", download_id);
    std::fs::create_dir_all(&download_dir).unwrap_or_default();

    let output_template = format!("{}/%(title)s.%(ext)s", download_dir);

    let result = tokio::task::spawn_blocking(move || {
        let args = downloader::build_ytdlp_args(&url, &output_template);

        let output = std::process::Command::new("yt-dlp").args(&args).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    if let Ok(entries) = std::fs::read_dir(&download_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.extension().is_some_and(|ext| ext == "mp3") {
                                let filename = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let file_size = std::fs::metadata(&path)
                                    .map(|m| m.len())
                                    .unwrap_or(0);
                                let file_path = path.to_string_lossy().into_owned();
                                return Ok((filename, file_path, file_size, download_dir));
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
        Ok(Ok((filename, file_path, file_size, download_dir))) => {
            match tokio::fs::File::open(&file_path).await {
                Ok(file) => {
                    // On Linux, unlinking after open keeps the data readable until the fd closes
                    let _ = tokio::fs::remove_dir_all(&download_dir).await;

                    let mut headers = HeaderMap::new();
                    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("audio/mpeg"));
                    let disposition = format!("attachment; filename=\"{}\"", filename);
                    headers.insert(
                        header::CONTENT_DISPOSITION,
                        HeaderValue::from_str(&disposition).unwrap_or(
                            HeaderValue::from_static("attachment; filename=\"download.mp3\""),
                        ),
                    );
                    if file_size > 0 {
                        headers.insert(
                            header::CONTENT_LENGTH,
                            HeaderValue::from_str(&file_size.to_string())
                                .unwrap_or(HeaderValue::from_static("0")),
                        );
                    }

                    let body = Body::from_stream(ReaderStream::new(file));
                    (StatusCode::OK, headers, body).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to open file: {}", e),
                )
                    .into_response(),
            }
        }
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task error: {}", e),
        )
            .into_response(),
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
