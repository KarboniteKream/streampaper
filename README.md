# streampaper

Fetches and serves YouTube live stream images for dynamic wallpapers with a time delay.

## Prerequisites
`streampaper` requires the following binaries to be on your `PATH`:
- [`yt-dlp`](https://github.com/yt-dlp/yt-dlp)
- [`ffmpeg`](https://github.com/FFmpeg/FFmpeg)

Make sure to initialize the database before starting:
```bash
sqlite3 database.sqlite < schema.sql
```

## Running
To start `streampaper`, execute the compiled binary or use Cargo:
```bash
cargo run
```
