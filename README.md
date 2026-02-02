# yt_down

A simple CLI tool to download YouTube audio as MP3, written in Rust. Uses [yt-dlp](https://github.com/yt-dlp/yt-dlp) under the hood.

Rust here is solely for learning purposes!

## Features

- Downloads YouTube videos as MP3 audio
- Supports browser cookies for bot detection bypass
- Docker support for easy reproducibility
- Cross-platform (Windows, Linux, macOS)

## Prerequisites

- [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- [ffmpeg](https://ffmpeg.org/)
- [deno](https://deno.land/) (for YouTube challenge solving)

### Windows

```powershell
winget install yt-dlp
```

### Linux/macOS

```bash
pip install yt-dlp
# or
brew install yt-dlp ffmpeg
```

## Installation

### From Source

```bash
cd yt_down
cargo build --release
```

The binary will be at `target/release/yt_down`.

### Docker

```bash
cd yt_down
docker build -t yt_down .
```

## Usage

### Native

```bash
./yt_down
# Then paste a YouTube URL when prompted
```

### Docker

```bash
docker run -it -v "$(pwd)/downloads:/downloads" yt_down
```

### With Cookies (for bot detection)

If YouTube blocks downloads, you can provide cookies:

1. Export cookies from your browser using the "Get cookies.txt LOCALLY" extension
2. Run with cookies mounted:

```bash
docker run -it -v "$(pwd)/downloads:/downloads" -v "$(pwd)/cookies.txt:/cookiesftxt" yt_down
```.

## How It Works

1. Prompts for a YouTube URL
2. Extracts audio using yt-dlp
3. Converts to MP3 using ffmpeg
4. Saves with the video title as filename

