use std::io;
use yt_down::downloader;

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

    let args = downloader::build_ytdlp_args(&url, "%(title)s.%(ext)s");

    match downloader::run_ytdlp(&args) {
        Ok(()) => println!("Download complete!"),
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}