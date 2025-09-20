# RTodo

A simple and efficient todo list CLI application written in Rust.

## Description

RTodo is a command-line todo list manager built with Rust, focusing on simplicity, speed, and reliability. It provides a clean interface for managing your daily tasks with support for due dates, completion tracking, and colorized output.

## Features

- **Add todos** with optional due dates
- **List todos** with filtering options (completed, pending, all)
- **Mark todos as complete** by ID
- **Remove todos** by ID
- **Colorized output** for better readability
- **Fast and lightweight** - built with Rust for performance

## Installation

### Prerequisites

- Rust 1.70+ and Cargo

### From Source

```bash
git clone https://github.com/your-username/rtodo.git
cd rtodo
cargo build --release
```

The binary will be available at `target/release/rtodo`.

## Usage

### Adding a todo

```bash
rtodo add "Buy groceries"
rtodo add "Finish project" --due 2024-12-31
```

### Listing todos

```bash
rtodo list                    # Show all todos
rtodo list --pending          # Show only pending todos
rtodo list --completed        # Show only completed todos
```

### Completing a todo

```bash
rtodo complete 1              # Mark todo with ID 1 as completed
```

### Removing a todo

```bash
rtodo remove 1                # Remove todo with ID 1
```

### Help

```bash
rtodo --help                  # Show help information
rtodo <subcommand> --help     # Show help for specific subcommand
```

## Dependencies

- **clap** - Command line argument parsing with derive API
- **serde + serde_json** - Serialization and deserialization
- **chrono** - Date and time handling
- **anyhow** - Error handling
- **colored** - Terminal color support

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