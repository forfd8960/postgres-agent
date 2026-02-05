# Week 8: TUI Basics & CLI

## Overview

This week implements the user interface layer for the PostgreSQL Agent, providing both a command-line interface (CLI) for scriptable operations and a terminal UI (TUI) for interactive use.

## Implementation Summary

### CLI Module (`postgres-agent-cli`)

The CLI module provides scriptable access to the PostgreSQL Agent using `clap` for argument parsing.

**Files:**
- `crates/cli/src/args.rs` - Command-line argument definitions
- `crates/cli/src/commands.rs` - CLI command implementations
- `crates/cli/src/lib.rs` - Module exports

**Commands:**
| Command | Description |
|---------|-------------|
| `query` | Execute a natural language query |
| `interactive` | Start interactive TUI mode |
| `execute` | Execute SQL from a file |
| `profiles` | List configured database profiles |
| `config` | Show current configuration |
| `schema` | Display database schema |
| `version` | Show version information |

**Output Formats:**
- `table` - ASCII table format (default)
- `json` - JSON output
- `csv` - CSV format
- `raw` - Raw SQL and results

### TUI Module (`postgres-agent-tui`)

The TUI module provides an interactive terminal interface. The implementation is UI-agnostic, focusing on state management and event handling that can be integrated with any TUI library (ratatui, cursive, etc.).

**Components:**

1. **Input (`components/input.rs`)**
   - Multi-mode text input (Normal, Insert, Command modes)
   - Full cursor navigation (forward/backward, start/end)
   - Text insertion and deletion
   - Placeholder support

2. **Status Bar (`components/status_bar.rs`)**
   - Connection status display (Disconnected, Connecting, Connected, Error)
   - Safety level indicator (ReadOnly, Balanced, Permissive)
   - Profile name, execution time, row count
   - View mode indicator

3. **Command Palette (`components/command_palette.rs`)**
   - Searchable command list
   - Keyboard navigation (up/down)
   - Category filtering
   - Default commands:
     - Navigation: Chat, Results, Schema views
     - Query: Execute, Clear
     - Database: Refresh Schema
     - Application: Quit, Help

4. **Chat View (`views/chat.rs`)**
   - Message history with roles (User, Assistant, System, Tool)
   - Reasoning/thinking message support
   - Loading indicators
   - Scroll offset management
   - Auto-scroll toggle

5. **Main Application (`app.rs`)**
   - Central state management
   - Event handling (input, special keys, control keys)
   - View mode switching (Chat, Results, Schema, Settings)
   - Command dispatch

**Key Types:**
```rust
// Application state
enum AppState { Running, Waiting, Processing, Error }

// View modes
enum ViewMode { Chat, Results, Schema, Settings }

// TUI errors
enum TuiError { InitError, EventError { message: String } }
```

## Architecture

### Design Decisions

1. **UI-Agnostic Core**: The TUI crate focuses on state management and event handling without binding to a specific rendering library. This allows flexibility in choosing or swapping TUI libraries.

2. **Component Composition**: Each component is self-contained with clear responsibilities:
   - `Input` handles text editing
   - `StatusBar` displays runtime information
   - `CommandPalette` manages quick actions
   - `ChatView` handles conversation display

3. **Event-Driven Design**: The main application processes events generically:
   - Character input
   - Special keys (Enter, Esc, Arrow keys, etc.)
   - Control key combinations

### Module Structure

```
crates/tui/
├── src/
│   ├── lib.rs           # Module exports
│   ├── app.rs           # Main TUI application
│   ├── components/
│   │   ├── mod.rs
│   │   ├── input.rs     # Text input component
│   │   ├── status_bar.rs # Status display
│   │   └── command_palette.rs
│   └── views/
│       ├── mod.rs
│       └── chat.rs      # Chat conversation view
└── Cargo.toml

crates/cli/
├── src/
│   ├── lib.rs           # Module exports
│   ├── args.rs          # CLI argument definitions
│   └── commands.rs      # Command implementations
└── Cargo.toml
```

## Tests

### CLI Tests (9 passing)
- `test_query_command_parsing`
- `test_interactive_command`
- `test_schema_command`
- `test_profiles_command`
- `test_version_command`
- `test_default_values`
- `test_output_format_parsing`
- `test_query_result_success`
- `test_query_result_error`

### TUI Tests (22 passing)
- App state transitions and command handling
- Input component: creation, insertion, deletion, cursor movement
- Command palette: visibility, filtering, selection movement
- Status bar: connection status, safety level, display
- Chat view: message creation, scroll operations

## Future Enhancements

1. **Actual TUI Rendering**: Integrate with ratatui or crossterm for visual rendering
2. **Syntax Highlighting**: Add SQL syntax highlighting in input
3. **Result Grid**: Display query results in a scrollable table
4. **Theme Support**: Multiple color themes for the TUI
5. **Key Bindings**: Customizable key bindings configuration

## Usage Examples

### CLI

```bash
# Query a database
pg-agent query "Show me all users"

# Execute SQL from file
pg-agent execute queries.sql --profile production

# Start interactive mode
pg-agent interactive

# List configured profiles
pg-agent profiles
```

### TUI Key Bindings

| Key | Action |
|-----|--------|
| `Ctrl+C` | Switch to Chat view |
| `Ctrl+R` | Switch to Results view |
| `Ctrl+S` | Switch to Schema view |
| `Ctrl+P` | Show command palette |
| `Ctrl+Q` | Quit application |
| `i` | Enter insert mode |
| `Esc` | Return to normal mode |
| `Enter` | Submit query |
| `Arrow keys` | Navigate/move cursor |
| `Tab` | Insert 4 spaces |

## Dependencies

### CLI
- `clap` - Command-line argument parsing
- `tokio` - Async runtime
- `anyhow` - Error handling

### TUI
- `tokio` - Async runtime
- `serde` - Serialization
- `tracing` - Logging
- Internal: `postgres-agent-core`, `postgres-agent-util`
