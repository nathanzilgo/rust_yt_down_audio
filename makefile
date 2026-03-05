default: build run

run:
	cargo run --release --bin yt_down

run-web:
	cargo run --release --bin yt_down_web

build:
	cargo build --release

build-docker:
	docker build -t yt_down .

run-docker:
	docker run -it --entrypoint yt_down --rm yt_down

run-docker-web:
	docker run -p 8080:8080 --rm yt_down
