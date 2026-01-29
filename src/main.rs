use std::process::Command;
use std::io;

fn get_input() -> String {
    let mut input = String::new();

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

    let status = Command::new("yt-dlp")
        .args([
            "-x",                    // Extract audio
            "--audio-format", "mp3", // Convert to mp3
            "--cookies-from-browser", "firefox", // Use cookies from Firefox
            "--extractor-args", "youtube:player_client=web",
            "--remote-components", "ejs:github",
            "-o", "%(title)s.%(ext)s",
            &url,
        ])
        .status()?;

    if status.success() {
        println!("Download complete!");
    } else {
        eprintln!("Download failed with exit code: {:?}", status.code());
    }

    Ok(())
}