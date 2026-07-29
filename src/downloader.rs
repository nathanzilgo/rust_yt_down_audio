use std::path::Path;
use std::process::Command;

/// Build the yt-dlp argument list for downloading audio as MP3.
/// `output_template` controls where the file is saved (e.g. "%(title)s.%(ext)s").
pub fn build_ytdlp_args(url: &str, output_template: &str) -> Vec<String> {
    let mut args = vec![
        "-x".to_string(),
        "--audio-format".to_string(), "mp3".to_string(),
        "--extractor-args".to_string(), "youtube:player_client=default".to_string(),
        "-o".to_string(), output_template.to_string(),
    ];

    // 1. Check for YOUTUBE_COOKIES env var (useful for Cloud Run environment variables)
    if let Ok(cookies_content) = std::env::var("YOUTUBE_COOKIES") {
        let temp_cookies_path = "/tmp/env_cookies.txt";
        let _ = std::fs::write(temp_cookies_path, cookies_content);
        println!("Using cookies from YOUTUBE_COOKIES env var");
        args.push("--cookies".to_string());
        args.push(temp_cookies_path.to_string());
    } 
    // 1.5 Check for COOKIES_PATH env var (very useful for Kubernetes mounted secrets)
    // Copy to /tmp because yt-dlp tries to write back to the cookies file on exit,
    // and secret mounts (Cloud Run, K8s) are read-only, causing OSError [Errno 22].
    else if let Ok(cookies_path) = std::env::var("COOKIES_PATH") {
        if Path::new(&cookies_path).exists() {
            let writable_path = "/tmp/cookies_rw.txt";
            match std::fs::copy(&cookies_path, writable_path) {
                Ok(_) => {
                    println!("Copied cookies from {} to {} (writable)", cookies_path, writable_path);
                    args.push("--cookies".to_string());
                    args.push(writable_path.to_string());
                }
                Err(e) => {
                    println!("Failed to copy cookies to writable path: {}, using original", e);
                    args.push("--cookies".to_string());
                    args.push(cookies_path);
                }
            }
        }
    }
    // 2. Check for cookie files in common locations (e.g. baked into Docker image)
    else if Path::new("/app/cookies.txt").exists() {
        println!("Using cookies file: /app/cookies.txt");
        args.push("--cookies".to_string());
        args.push("/app/cookies.txt".to_string());
    } 
    else if Path::new("cookies.txt").exists() {
        println!("Using cookies file: cookies.txt");
        args.push("--cookies".to_string());
        args.push("cookies.txt".to_string());
    } 
    else if Path::new("/cookies.txt").exists() {
        println!("Using cookies file: /cookies.txt");
        args.push("--cookies".to_string());
        args.push("/cookies.txt".to_string());
    } 
    // 3. Fallback to browser cookies (only works locally, not in Cloud Run)
    else if cfg!(target_os = "windows") || std::env::var("BROWSER_COOKIES").is_ok() {
        println!("Using browser cookies");
        args.push("--cookies-from-browser".to_string());
        args.push("firefox".to_string());
    }

    args.push(url.to_string());
    args
}

/// Run yt-dlp with the given arguments. Returns Ok(()) on success.
pub fn run_ytdlp(args: &[String]) -> Result<(), String> {
    let status = Command::new("yt-dlp")
        .args(args)
        .status()
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Download failed with exit code: {:?}", status.code()))
    }
}
