use std::process::Command;
use std::io;
use std::path::Path;

fn get_input() -> String {
    let mut input = String::new();
    
    println!("Please enter the youtube link you want to download:");

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let trimmed_input = input.trim();
    println!("You entered: {}", trimmed_input);

    trimmed_input.to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let url = get_input();
    
    println!("Downloading audio from: {}", url);

    let mut args = vec![
        "-x".to_string(),
        "--audio-format".to_string(), "mp3".to_string(),
        "--extractor-args".to_string(), "youtube:player_client=web".to_string(),
        "--remote-components".to_string(), "ejs:github".to_string(),
        "-o".to_string(), "%(title)s.%(ext)s".to_string(),
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

    args.push(url);

    let status = Command::new("yt-dlp")
        .args(&args)
        .status()?;

    if status.success() {
        println!("Download complete!");
    } else {
        eprintln!("Download failed with exit code: {:?}", status.code());
    }

    Ok(())
}