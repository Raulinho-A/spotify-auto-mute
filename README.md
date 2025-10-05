# spotify-auto-mute

Automatically mutes Spotify when ads are detected on Windows using Rust and Win32 APIs.

## Installation and Usage

This project automatically mutes Spotify when ads are detected, using Rust and Win32 APIs on Windows.

### Requirements

Make sure you have the following installed:

- Rust and Cargo
- Windows 10 or higher
- Spotify Desktop installed and running

### Clone the repository

```bash
git clone https://github.com/Raulinho-A/spotify-auto-mute.git
cd spotify-auto-mute
```

### Run the program

```
cargo run --release
```

1. You can test how the ad detection works:

2. Open Spotify Desktop.

3. Play a playlist.

Observe how the volume automatically mutes during commercials and returns to normal once the ad ends.

### Build the executable

If you want to generate a standalone executable:

```
cargo build --release
```

The binary will be located at:

target/release/spotify-auto-mute.exe
