# RTodo

A simple and efficient todo list CLI application written in Rust.

## Description

RTodo is a command-line todo list manager built with Rust, focusing on simplicity, speed, and reliability. It provides a clean interface for managing your daily tasks with support for due dates, categories, completion tracking, and colorized output.

## Features

- **Add todos** with optional due dates, categories, and priorities
- **List todos** with filtering options (completed, pending, all)
- **Search todos** by text content
- **Mark todos as complete/incomplete** by ID
- **Edit existing todos** with updated information
- **Organize todos by categories** with category management
- **View todos by due dates** (today, overdue)
- **Remove todos** by ID
- **Colorized output** for better readability
- **Fast and lightweight** - built with Rust for performance
- **Natural language date parsing** for flexible due date input

## Installation

### Prerequisites

- **Rust 1.70+ and Cargo** - [Install Rust](https://rustup.rs/)
- **Git** - For cloning the repository

### From Source

```bash
# Clone the repository
git clone https://github.com/your-username/rtodo.git
cd rtodo

# Build release version
cargo build --release

# Optionally install to PATH
cargo install --path .
```

The binary will be available at:
- `target/release/rtodo` (local build)
- `~/.cargo/bin/rtodo` (if installed to PATH)

### Quick Install via Cargo

```bash
# Install directly from local path
cargo install --path /path/to/rtodo

# Or install from Git repository
cargo install --git https://github.com/your-username/rtodo.git
```

### Verify Installation

```bash
rtodo --version
rtodo --help
```

## Usage

### Basic Task Management

#### Adding a todo

```bash
# Simple todo
rtodo add "Buy groceries"

# Todo with due date
rtodo add "Finish project" --due 2024-12-31
rtodo add "Meeting with team" --due "tomorrow"
rtodo add "Doctor appointment" --due "next Friday"

# Todo with category and priority
rtodo add "Review code" --category work --priority high
rtodo add "Call mom" --category personal --priority medium
```

#### Listing todos

```bash
# Show all todos
rtodo list

# Filter by completion status
rtodo list --pending          # Show only pending todos
rtodo list --completed        # Show only completed todos

# Filter by category
rtodo list --category work    # Show only work-related todos
rtodo list --category personal
```

#### Completing and uncompleting todos

```bash
# Mark todo as completed
rtodo complete 1              # Mark todo with ID 1 as completed

# Mark todo as incomplete (undo completion)
rtodo incomplete 1            # Mark todo with ID 1 as incomplete
```

#### Removing todos

```bash
# Remove a todo by ID
rtodo remove 1                # Remove todo with ID 1

# Remove with confirmation
rtodo remove 1 --confirm      # Prompt for confirmation before removal
```

### Advanced Task Management

#### Searching todos

```bash
# Search by text content
rtodo search "groceries"      # Find todos containing "groceries"
rtodo search "project"        # Find todos containing "project"
rtodo search "work"           # Find todos with "work" in title or category
```

#### Editing existing todos

```bash
# Edit todo title
rtodo edit 1 --title "Updated todo title"

# Edit due date
rtodo edit 1 --due "2024-12-25"
rtodo edit 1 --due "next Monday"

# Edit category and priority
rtodo edit 1 --category work --priority high

# Edit multiple fields at once
rtodo edit 1 --title "New title" --due tomorrow --priority low
```

### Category Management

#### Viewing categories

```bash
# List all categories with task counts
rtodo categories

# Example output:
# Categories:
# work (5 tasks)
# personal (3 tasks)
# shopping (2 tasks)
```

#### Renaming categories

```bash
# Rename a category across all tasks
rtodo rename-category old-name new-name

# Example: rename "work" to "professional"
rtodo rename-category work professional
```

### Date-based Views

#### View today's tasks

```bash
# Show all tasks due today
rtodo due-today

# Example output shows tasks with due date of current date
```

#### View overdue tasks

```bash
# Show all overdue tasks
rtodo overdue

# Example output shows tasks past their due date
```

### Examples and Common Workflows

#### Daily workflow example

```bash
# Morning: check what's due today
rtodo due-today

# Add a quick task
rtodo add "Respond to emails" --due today --category work

# Check all pending work
rtodo list --pending --category work

# Complete a task
rtodo complete 2

# Evening: see what's overdue
rtodo overdue
```

#### Project management example

```bash
# Set up project tasks
rtodo add "Plan project architecture" --category project-x --priority high --due "next Monday"
rtodo add "Write documentation" --category project-x --priority medium --due "next Wednesday"
rtodo add "Code review" --category project-x --priority medium --due "next Friday"

# Check project progress
rtodo list --category project-x

# Update a task
rtodo edit 3 --title "Complete code review and testing" --priority high
```

#### Category organization example

```bash
# View current categories
rtodo categories

# Rename a category
rtodo rename-category personal home

# Search within categories
rtodo search "meeting" | grep work
```

### Getting Help

```bash
# General help
rtodo --help

# Help for specific commands
rtodo add --help
rtodo list --help
rtodo edit --help

# Version information
rtodo --version
```

## Troubleshooting

### Common Issues

#### "Command not found" error
```bash
# Ensure rtodo is in your PATH
echo $PATH | grep -q "$HOME/.cargo/bin" || echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# Or use absolute path
~/rtodo/target/release/rtodo --help
```

#### "No todos found" when you expect results
```bash
# Check if todos file exists and has content
ls -la ~/.rtodo/
cat ~/.rtodo/todos.json

# Verify todos were added successfully
rtodo list --all
```

#### Date parsing issues
```bash
# Use ISO format for guaranteed parsing
rtodo add "Task" --due 2024-12-31

# Natural language dates that work:
rtodo add "Task" --due "tomorrow"
rtodo add "Task" --due "next Friday"
rtodo add "Task" --due "in 3 days"

# If date parsing fails, try ISO format: YYYY-MM-DD
```

#### Permission errors on data directory
```bash
# Ensure proper permissions on data directory
chmod 755 ~/.rtodo/
chmod 644 ~/.rtodo/todos.json
```

#### JSON parse errors
```bash
# Backup and reset if data is corrupted
cp ~/.rtodo/todos.json ~/.rtodo/todos.json.backup
echo '{"tasks": [], "next_id": 1}' > ~/.rtodo/todos.json
```

### Data Location

RTodo stores data in:
- **Linux/macOS**: `~/.rtodo/todos.json`
- **Windows**: `%APPDATA%\rtodo\todos.json`

### Logging and Debug

For debugging issues:
```bash
# Set debug logging (if implemented)
RUST_LOG=debug rtodo list

# Check version and build info
rtodo --version
```

### Getting Help

- Create an issue on GitHub for bugs
- Check existing issues for known problems
- Include your OS, Rust version, and exact command that failed

## Dependencies

- **clap** - Command line argument parsing with derive API
- **serde + serde_json** - Serialization and deserialization
- **chrono** - Date and time handling with timezone support
- **chrono-english** - Natural language date parsing ("tomorrow", "next Friday")
- **anyhow** - Error handling and context
- **colored** - Terminal color support for better UX
- **dirs** - Cross-platform directory paths
- **regex** - Pattern matching for search functionality

## Development

### Building

```bash
cargo build
```

### Running

```bash
cargo run -- <command>
```

### Testing

```bash
cargo test
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.