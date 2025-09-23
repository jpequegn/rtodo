# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RTodo is a command-line todo list manager written in Rust. It focuses on simplicity, speed, and reliability with support for due dates, categories, priorities, and natural language date parsing.

## Architecture

The codebase follows a simple two-module architecture:

1. **main.rs** - CLI interface and command handling logic
   - Defines `Cli` struct with clap parser derive macros for command-line argument parsing
   - Implements all CLI commands (add, list, search, complete, edit, etc.)
   - Handles natural language date parsing via chrono-english
   - Provides colorized output for better readability
   - Contains display logic with highlighting for search results

2. **models.rs** - Data models and persistence layer
   - `Task` struct: Individual todo items with completion status, due dates, categories, and priorities
   - `TodoList` struct: Collection manager for tasks with CRUD operations
   - `TaskUpdate` builder pattern for partial updates
   - JSON-based persistence with atomic writes and automatic backups
   - Default storage location: `~/.todo-cli/tasks.json`

## Key Commands

### Build & Development
```bash
# Build debug version
cargo build

# Build release version (recommended for testing)
cargo build --release

# Run directly with cargo
cargo run -- [commands]

# Run with custom data file
cargo run -- --file custom.json [commands]
```

### Testing
```bash
# Run all unit tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test cli_tests
```

### Common Development Tasks
```bash
# Check for compilation errors
cargo check

# Format code
cargo fmt

# Run clippy for linting
cargo clippy

# Generate documentation
cargo doc --open
```

## Important Implementation Details

### Natural Language Date Parsing
The application uses `chrono-english` for flexible date input. The `parse_date()` function in main.rs handles both natural language ("tomorrow", "next Friday") and ISO format (YYYY-MM-DD). All dates are set to end of day (23:59:59) for consistent due date handling.

### Data Persistence
The `TodoList` implements atomic file writes with automatic backups:
1. Creates backup of existing file before saving
2. Writes to temporary file first
3. Performs atomic rename to ensure data integrity
4. Handles directory creation automatically

### Search Functionality
Supports both plain text and regex searching with:
- Case-sensitive/insensitive options
- Search in title and description
- Highlighting of matched terms in output
- Filtering and sorting of results

### Task State Management
Tasks have multiple state-checking methods:
- `is_overdue()`: Past due date and not completed
- `is_due_soon()`: Due within 7 days
- `is_due_today()`: Due on current date
These methods consider completion status to avoid showing completed tasks as overdue.

## Testing Approach

The codebase includes comprehensive unit tests in `models.rs` covering:
- Task creation and state management
- TodoList CRUD operations
- File persistence with edge cases (corrupted files, missing directories)
- Category management
- Search functionality
- Due date calculations

Integration tests in `tests/cli_tests.rs` use temporary directories to test the full CLI workflow without affecting user data.

## Dependencies

- **clap (4.4)**: Command-line parsing with derive macros
- **serde/serde_json**: Serialization for data persistence
- **chrono (0.4)**: Date/time handling with timezone support
- **chrono-english (0.1)**: Natural language date parsing
- **anyhow (1.0)**: Error handling with context
- **colored (2.0)**: Terminal color output
- **dirs (5.0)**: Cross-platform directory paths
- **regex (1.10)**: Pattern matching for search