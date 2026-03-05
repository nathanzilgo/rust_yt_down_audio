use std::path::Path;
use std::process::Command;

/// Build the yt-dlp argument list for downloading audio as MP3.
/// `output_template` controls where the file is saved (e.g. "%(title)s.%(ext)s").
pub fn build_ytdlp_args(url: &str, output_template: &str) -> Vec<String> {
    let mut args = vec![
        "-x".to_string(),
        "--audio-format".to_string(), "mp3".to_string(),
        "--extractor-args".to_string(), "youtube:player_client=web".to_string(),
        "--remote-components".to_string(), "ejs:github".to_string(),
        "-o".to_string(), output_template.to_string(),
    ];

    // Use cookies file if available (for Docker), otherwise try browser
    if Path::new("/cookies.txt").exists() {
        println!("Using cookies file: /cookies.txt");
        args.push("--cookies".to_string());
        args.push("/cookies.txt".to_string());
    } else if cfg!(target_os = "windows") || std::env::var("BROWSER_COOKIES").is_ok() {
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
