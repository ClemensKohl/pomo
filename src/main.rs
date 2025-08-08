use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use rodio::{OutputStream, Sink, Source};
use std::{
    io,
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Focus time in minutes
    #[arg(short, long, default_value_t = 25)]
    focus: u64,

    /// Break time in minutes  
    #[arg(short, long, default_value_t = 5)]
    break_time: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TimerState {
    Focus,
    Break,
    Paused,
}

struct PomodoroTimer {
    focus_remaining: u64,
    break_remaining: u64,
    focus_duration: u64,
    break_duration: u64,
    state: TimerState,
    last_update: Instant,
    total_cycles: u32,
    notification_flash: bool,
    flash_timer: Instant,
}

impl PomodoroTimer {
    fn new(focus_minutes: u64, break_minutes: u64) -> Self {
        let focus_duration = focus_minutes * 60;
        let break_duration = break_minutes * 60;
        Self {
            focus_remaining: focus_duration,
            break_remaining: break_duration,
            focus_duration,
            break_duration,
            state: TimerState::Focus,
            last_update: Instant::now(),
            total_cycles: 0,
            notification_flash: false,
            flash_timer: Instant::now(),
        }
    }

    fn update(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs();
        self.last_update = now;

        let mut sound_needed = false;

        match self.state {
            TimerState::Focus => {
                if self.focus_remaining > elapsed {
                    self.focus_remaining -= elapsed;
                } else {
                    self.focus_remaining = 0;
                    self.state = TimerState::Break;
                    self.total_cycles += 1;
                    sound_needed = true;
                    self.notification_flash = true;
                    self.flash_timer = Instant::now();
                }
            }
            TimerState::Break => {
                if self.break_remaining > elapsed {
                    self.break_remaining -= elapsed;
                } else {
                    self.break_remaining = self.break_duration;
                    self.focus_remaining = self.focus_duration;
                    self.state = TimerState::Focus;
                    sound_needed = true;
                    self.notification_flash = true;
                    self.flash_timer = Instant::now();
                }
            }
            TimerState::Paused => {}
        }

        // Update flash notification
        if self.notification_flash && self.flash_timer.elapsed() > Duration::from_secs(2) {
            self.notification_flash = false;
        }

        sound_needed
    }

    fn toggle_pause(&mut self) {
        self.state = match self.state {
            TimerState::Focus => TimerState::Paused,
            TimerState::Break => TimerState::Paused,
            TimerState::Paused => TimerState::Focus,
        };
        self.last_update = Instant::now();
    }

    fn reset(&mut self) {
        self.focus_remaining = self.focus_duration;
        self.break_remaining = self.break_duration;
        self.state = TimerState::Focus;
        self.last_update = Instant::now();
        self.notification_flash = false;
    }

    fn adjust_focus_time(&mut self, minutes: u64) {
        self.focus_duration = minutes * 60;
        if self.state == TimerState::Focus {
            self.focus_remaining = self.focus_duration;
        }
    }

    fn adjust_break_time(&mut self, minutes: u64) {
        self.break_duration = minutes * 60;
        if self.state == TimerState::Break {
            self.break_remaining = self.break_duration;
        }
    }

    fn format_time(seconds: u64) -> String {
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    fn get_ascii_digits(time_str: &str) -> Vec<String> {
        let digits = [
            [
                " ██████  ",
                "██    ██ ",
                "██    ██ ",
                "██    ██ ",
                " ██████  ",
            ], // 0
            [
                "   ██    ",
                " ████    ",
                "   ██    ",
                "   ██    ",
                " ██████  ",
            ], // 1
            [
                " ██████  ",
                "      ██ ",
                " ██████  ",
                "██       ",
                "████████ ",
            ], // 2
            [
                " ██████  ",
                "      ██ ",
                " ██████  ",
                "      ██ ",
                " ██████  ",
            ], // 3
            [
                "██    ██ ",
                "██    ██ ",
                "████████ ",
                "      ██ ",
                "      ██ ",
            ], // 4
            [
                "████████ ",
                "██       ",
                "███████  ",
                "      ██ ",
                "███████  ",
            ], // 5
            [
                " ██████  ",
                "██       ",
                "███████  ",
                "██    ██ ",
                " ██████  ",
            ], // 6
            [
                "████████ ",
                "      ██ ",
                "    ██   ",
                "  ██     ",
                "██       ",
            ], // 7
            [
                " ██████  ",
                "██    ██ ",
                " ██████  ",
                "██    ██ ",
                " ██████  ",
            ], // 8
            [
                " ██████  ",
                "██    ██ ",
                " ███████ ",
                "      ██ ",
                " ██████  ",
            ], // 9
            [
                "         ",
                "   ██    ",
                "         ",
                "   ██    ",
                "         ",
            ], // : (colon)
        ];

        let char_to_index = |c: char| match c {
            '0'..='9' => (c as usize) - ('0' as usize),
            ':' => 10,
            _ => 10, // Default to colon for unknown chars
        };

        let mut result = vec![String::new(); 5];
        
        for ch in time_str.chars() {
            let digit_lines = &digits[char_to_index(ch)];
            for (i, line) in digit_lines.iter().enumerate() {
                result[i].push_str(line);
            }
        }

        result
    }
}

fn play_notification_sound() {
    tokio::spawn(async {
        // Try to play sound, but don't crash if audio device is unavailable
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                // Generate 3 beeps with pauses between them
                for i in 0..3 {
                    // Generate a sine wave beep
                    let beep = rodio::source::SineWave::new(800.0) // 800 Hz frequency
                        .take_duration(Duration::from_millis(200)) // 0.2 seconds
                        .amplify(0.20); // 20% volume
                    
                    sink.append(beep);
                    
                    // Add a pause between beeps (except after the last one)
                    if i < 2 {
                        let silence = rodio::source::SineWave::new(0.0) // Silent "beep"
                            .take_duration(Duration::from_millis(150)) // 0.15 seconds pause
                            .amplify(0.0); // 0% volume (silence)
                        sink.append(silence);
                    }
                }
                
                let _ = sink.sleep_until_end(); // Ignore errors if audio playback fails
            }
        }
        // If audio fails, we simply continue without sound notification
    });
}

fn draw_ui(f: &mut Frame, timer: &PomodoroTimer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(8),     // Focus timer
            Constraint::Min(8),     // Break timer
            Constraint::Length(3),  // Controls
        ])
        .split(f.area());

    // Header
    let header_text = if timer.notification_flash {
        "🔔 NOTIFICATION! 🔔"
    } else {
        "🍅 POMODORO TIMER 🍅"
    };
    let header_color = if timer.notification_flash {
        Color::Yellow
    } else {
        Color::Red
    };
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(header_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Cyan)));
    f.render_widget(header, chunks[0]);

    // Focus Timer
    let focus_active = timer.state == TimerState::Focus;
    let focus_style = if focus_active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    
    let focus_time = PomodoroTimer::format_time(timer.focus_remaining);
    let focus_ascii = PomodoroTimer::get_ascii_digits(&focus_time);
    
    let focus_lines: Vec<Line> = focus_ascii
        .iter()
        .map(|line| Line::from(Span::styled(line.clone(), focus_style)))
        .collect();
    
    let focus_title = if focus_active { "FOCUS TIME ⚡" } else { "FOCUS TIME" };
    let focus_block = Block::default()
        .title(focus_title)
        .borders(Borders::ALL)
        .style(if focus_active {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    
    let focus_paragraph = Paragraph::new(focus_lines)
        .block(focus_block)
        .alignment(Alignment::Center);
    f.render_widget(focus_paragraph, chunks[1]);

    // Break Timer
    let break_active = timer.state == TimerState::Break;
    let break_style = if break_active {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    
    let break_time = PomodoroTimer::format_time(timer.break_remaining);
    let break_ascii = PomodoroTimer::get_ascii_digits(&break_time);
    
    let break_lines: Vec<Line> = break_ascii
        .iter()
        .map(|line| Line::from(Span::styled(line.clone(), break_style)))
        .collect();
    
    let break_title = if break_active { "BREAK TIME ☕" } else { "BREAK TIME" };
    let break_block = Block::default()
        .title(break_title)
        .borders(Borders::ALL)
        .style(if break_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    
    let break_paragraph = Paragraph::new(break_lines)
        .block(break_block)
        .alignment(Alignment::Center);
    f.render_widget(break_paragraph, chunks[2]);

    // Controls
    let controls = match timer.state {
        TimerState::Paused => "SPACE: Resume | R: Reset | Q: Quit",
        _ => "SPACE: Pause | R: Reset | Q: Quit",
    };
    
    let focus_min = timer.focus_duration / 60;
    let break_min = timer.break_duration / 60;
    let settings_text = format!("Focus: {}min | Break: {}min", focus_min, break_min);
    let controls_text = format!("Cycles: {} | {} | f/F: focus +/- | b/B: break +/- | {}", 
                               timer.total_cycles, settings_text, controls);
    let controls_paragraph = Paragraph::new(controls_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls_paragraph, chunks[3]);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut timer = PomodoroTimer::new(args.focus, args.break_time);
    let mut last_tick = Instant::now();

    loop {
        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(' ') => timer.toggle_pause(),
                        KeyCode::Char('r') => timer.reset(),
                        KeyCode::Char('f') => {
                            let current_focus = timer.focus_duration / 60;
                            timer.adjust_focus_time((current_focus + 1).max(1));
                        },
                        KeyCode::Char('F') => {
                            let current_focus = timer.focus_duration / 60;
                            timer.adjust_focus_time((current_focus.saturating_sub(1)).max(1));
                        },
                        KeyCode::Char('b') => {
                            let current_break = timer.break_duration / 60;
                            timer.adjust_break_time((current_break + 1).max(1));
                        },
                        KeyCode::Char('B') => {
                            let current_break = timer.break_duration / 60;
                            timer.adjust_break_time((current_break.saturating_sub(1)).max(1));
                        },
                        _ => {}
                    }
                }
            }
        }

        // Update timer
        if timer.state != TimerState::Paused {
            let now = Instant::now();
            if now.duration_since(last_tick) >= Duration::from_secs(1) {
                if timer.update() {
                    play_notification_sound();
                }
                last_tick = now;
            }
        }

        // Draw UI
        terminal.draw(|f| draw_ui(f, &timer))?;
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
