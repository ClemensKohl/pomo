# Pomo - Terminal Pomodoro Timer

A retro-styled terminal-based Pomodoro timer written in Rust with a beautiful TUI interface and customizable timer durations.

## Features

- 🍅 Classic Pomodoro technique (25 min focus, 5 min break by default)
- ⏰ **Customizable timer durations** via command line or runtime controls
- 🎨 Retro terminal UI with big ASCII art numbers
- 🔔 Audio notifications (3 beeps) when timers complete
- ⌨️  Keyboard-only controls
- 📊 Cycle tracking
- ⏸️  Pause/resume functionality
- 🎛️  Real-time timer adjustment

## Controls

### Basic Controls
- `SPACE` - Pause/Resume timer
- `R` - Reset current timer
- `Q` - Quit application

### Timer Adjustment (Real-time)
- `f` - Increase focus time by 1 minute
- `F` - Decrease focus time by 1 minute
- `b` - Increase break time by 1 minute
- `B` - Decrease break time by 1 minute

## Installation

### Building from Source

1. **Clone or download the source code** to your local machine

2. **Build the release binary:**
```bash
cargo build --release
```

3. **The binary will be created at:**
```bash
target/release/pomo
```

4. **Optional: Install system-wide (Unix/Linux/macOS):**
```bash
# Copy to a directory in your PATH
sudo cp target/release/pomo /usr/local/bin/
# Or add to your user bin
mkdir -p ~/.local/bin
cp target/release/pomo ~/.local/bin/
```

5. **Optional: Create a portable binary (Windows):**
```bash
# The binary will be at target/release/pomo.exe
# Copy it wherever you want and run directly
```

## Usage

### Running with Cargo (Development)
```bash
# Default usage (25 min focus, 5 min break)
cargo run

# Custom timer durations
cargo run -- --focus 45 --break-time 10
cargo run -- -f 2 -b 1
```

### Running the Binary (After Building)
```bash
# Default usage
./target/release/pomo

# Custom timer durations  
./target/release/pomo --focus 45 --break-time 10
./target/release/pomo -f 2 -b 1

# If installed system-wide
pomo --focus 30 --break-time 8
```

### Command Line Options
- `-f, --focus <MINUTES>` - Set focus time in minutes (default: 25)
- `-b, --break-time <MINUTES>` - Set break time in minutes (default: 5)
- `-h, --help` - Show help message
- `-V, --version` - Show version

The timer starts in focus mode with your specified duration. When it completes, it automatically switches to break mode, and the cycle repeats. The active timer is highlighted in green (focus) or yellow (break), while the inactive timer is shown in gray.

The current timer settings are displayed at the bottom of the screen, and you can adjust them in real-time using the keyboard shortcuts without losing your current progress.

## Requirements

- Rust 1.70+
- Terminal with Unicode support
- Audio device (optional - timer works without sound)

## Examples

```bash
# Pomodoro for deep work (50 min focus, 10 min break)
cargo run -- --focus 50 --break-time 10

# Short bursts (15 min focus, 3 min break)
cargo run -- --focus 15 --break-time 3

# Ultradian rhythm (90 min focus, 20 min break)
cargo run -- --focus 90 --break-time 20
```

Enjoy your productive coding sessions! 🚀