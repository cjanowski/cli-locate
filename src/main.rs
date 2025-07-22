use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Paragraph, canvas::Canvas},
    Frame, Terminal,
};
use reqwest;
use serde::Deserialize;
use std::{
    io,
    time::{Duration, Instant},
};


#[derive(Debug, Deserialize)]
struct LocationResponse {
    lat: f64,
    lon: f64,
    city: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Clone)]
struct Location {
    latitude: f64,
    longitude: f64,
    city: String,
    country: String,
}

struct App {
    location: Option<Location>,
    last_update: Instant,
    rotation: f64,
}

impl App {
    fn new() -> App {
        App {
            location: None,
            last_update: Instant::now(),
            rotation: 0.0,
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.rotation += elapsed * 10.0;
        self.last_update = now;
    }
}

async fn get_location() -> Result<Location> {
    let response = reqwest::get("http://ip-api.com/json/")
        .await?
        .json::<LocationResponse>()
        .await?;
    
    Ok(Location {
        latitude: response.lat,
        longitude: response.lon,
        city: response.city.unwrap_or_else(|| "Unknown".to_string()),
        country: response.country.unwrap_or_else(|| "Unknown".to_string()),
    })
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let info_text = if let Some(ref location) = app.location {
        format!(
            "Location: {}, {} | Lat: {:.4}°, Lon: {:.4}°",
            location.city, location.country, location.latitude, location.longitude
        )
    } else {
        "Fetching location...".to_string()
    };

    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("GPS Globe"))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    f.render_widget(info, chunks[0]);

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Globe"))
        .paint(|ctx| {
            ctx.draw(&ratatui::widgets::canvas::Map {
                color: Color::White,
                resolution: ratatui::widgets::canvas::MapResolution::High,
            });

            if let Some(ref location) = app.location {
                let x = location.longitude;
                let y = location.latitude;
                
                ctx.print(x, y, "●");
                
                ctx.print(x + 5.0, y + 5.0, format!("{}", location.city));
            }

            for lat in (-90..=90).step_by(30) {
                for lon in (-180..=180).step_by(30) {
                    ctx.print(lon as f64, lat as f64, "·");
                }
            }
        })
        .marker(symbols::Marker::Braille)
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0]);
    f.render_widget(canvas, chunks[1]);
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    if let Ok(location) = get_location().await {
        app.location = Some(location);
    }

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => {
                        if let Ok(location) = get_location().await {
                            app.location = Some(location);
                        }
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = Instant::now();
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
