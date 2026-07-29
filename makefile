default: install build run

install:
	@echo "Checking dependencies..."
	@which ffmpeg >/dev/null 2>&1 || (echo "Installing ffmpeg..." && sudo apt-get update && sudo apt-get install -y ffmpeg)
	@which yt-dlp >/dev/null 2>&1 || (echo "Installing yt-dlp..." && pip install yt-dlp || (curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && chmod a+rx /usr/local/bin/yt-dlp))
	@which deno >/dev/null 2>&1 || (echo "Installing deno..." && curl -fsSL https://deno.land/install.sh | sh)
	@echo "Dependencies installed successfully!"

run:
	cargo run --release --bin yt_down

web:
	@echo "Starting local web server on http://localhost:8080"
	cargo run --release --bin yt_down_web

build:
	cargo build --release

build-docker:
	docker build -t yt_down .

run-docker:
	docker run -it --entrypoint yt_down --rm yt_down

run-docker-web:
	docker run -p 8080:8080 --rm yt_down
