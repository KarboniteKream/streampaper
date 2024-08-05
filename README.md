# streampaper

Fetches and serves YouTube live stream images for dynamic wallpapers with a time delay.

## Prerequisites
`streampaper` requires the following binaries to be on your `PATH`:
- [`yt-dlp`](https://github.com/yt-dlp/yt-dlp)
- [`ffmpeg`](https://github.com/FFmpeg/FFmpeg)

Make sure to run the database migrations before starting:
```bash
diesel migration run
```

## Running
To start `streampaper`, execute the compiled binary or use Cargo:
```bash
cargo run
```
