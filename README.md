# GPS Globe Terminal App

A Rust terminal application that displays your current GPS location on an ASCII globe.

## Features

- Fetches your location using IP geolocation
- Displays your location on a world map in the terminal
- Shows latitude, longitude, city, and country
- Interactive terminal UI with real-time updates

## Usage

```bash
# Build and run the application
cargo run

# Controls:
# - 'q' or 'Esc': Quit the application
# - 'r': Refresh location
```

## Dependencies

- `tokio`: Async runtime
- `reqwest`: HTTP client for location API
- `ratatui`: Terminal UI framework
- `crossterm`: Cross-platform terminal manipulation
- `serde`: JSON serialization
- `anyhow`: Error handling

## How it works

1. Uses the ip-api.com service to get your approximate location based on your IP address
2. Renders a world map using ASCII characters in the terminal
3. Marks your location with a red dot (●) on the globe
4. Displays coordinate grid points for reference

Note: Location accuracy depends on your IP address(local machine) and may not be precise for mobile networks or VPNs.
