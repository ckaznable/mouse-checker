use std::{
    io,
    time::{Duration, Instant},
};

use clap::{command, Parser};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// detect sec time for mouse click
    #[arg(short, long, default_value_t = 1.0)]
    sec: f32,

    #[arg(short, long, default_value_t = 0)]
    millisecond: u64,
}

struct AppState {
    event: Vec<Duration>,
    duration: Duration,
    instant: Instant,
}

impl AppState {
    fn new(duration: Duration) -> Self {
        Self {
            event: Vec::new(),
            duration,
            instant: Instant::now(),
        }
    }

    fn reset(&mut self) {
        self.instant = Instant::now();
        self.event.clear();
    }

    fn on_click(&mut self) {
        if self.event.is_empty() {
            self.instant = Instant::now();
        }

        let mut dur = if self.event.is_empty() {
            Duration::from_secs(0)
        } else {
            self.instant.elapsed()
        };

        if dur > self.duration {
            self.reset();
            dur = Duration::from_secs(0);
        }

        self.event.push(dur);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let detect_duration = if args.millisecond == 0 {
        Duration::from_secs_f32(args.sec)
    } else {
        Duration::from_millis(args.millisecond)
    };

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = AppState::new(detect_duration);
    // run application
    run_app(&mut terminal, app)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: AppState) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        match event::read()? {
            Event::Mouse(event) => {
                if let MouseEventKind::Down(_) = event.kind {
                    app.on_click();
                }
            }
            Event::Key(event) => {
                if event.code == KeyCode::Char('q') {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut AppState) {
    let size = f.size();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(size);

    let text = if app.event.is_empty() {
        vec![Line::from("please click the mouse!")]
    } else {
        app.event
            .iter()
            .map(|dur| Line::from(format!("{} ms", dur.as_millis())))
            .collect::<Vec<_>>()
    };

    let padding_top = (size.height - (2 + app.event.len() as u16)) / 2;
    let block = Block::default()
        .padding(Padding {
            top: padding_top,
            bottom: 0,
            left: 0,
            right: 0,
        })
        .borders(Borders::ALL);

    let p = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(p, layout[0]);
}
