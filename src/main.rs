use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod models;

#[derive(Parser)]
#[command(name = "rtodo")]
#[command(about = "A simple and efficient todo list CLI written in Rust")]
#[command(version, author)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new todo item
    Add {
        /// The todo item description
        description: String,
        /// Optional due date (YYYY-MM-DD format)
        #[arg(short, long)]
        due: Option<String>,
    },
    /// List all todo items
    List {
        /// Show only completed items
        #[arg(short, long)]
        completed: bool,
        /// Show only pending items
        #[arg(short, long)]
        pending: bool,
    },
    /// Mark a todo item as completed
    Complete {
        /// The ID of the todo item to complete
        id: usize,
    },
    /// Remove a todo item
    Remove {
        /// The ID of the todo item to remove
        id: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { description, due }) => {
            println!("{} {}", "Adding todo:".green().bold(), description);
            if let Some(due_date) = due {
                println!("{} {}", "Due date:".yellow(), due_date);
            }
            // TODO: Implement todo addition logic
        }
        Some(Commands::List { completed, pending }) => {
            println!("{}", "Listing todos:".blue().bold());
            if completed {
                println!("Showing only completed items");
            } else if pending {
                println!("Showing only pending items");
            } else {
                println!("Showing all items");
            }
            // TODO: Implement todo listing logic
        }
        Some(Commands::Complete { id }) => {
            println!("{} {}", "Completing todo with ID:".green().bold(), id);
            // TODO: Implement todo completion logic
        }
        Some(Commands::Remove { id }) => {
            println!("{} {}", "Removing todo with ID:".red().bold(), id);
            // TODO: Implement todo removal logic
        }
        None => {
            println!("{}", "Welcome to rtodo!".cyan().bold());
            println!("Use 'rtodo --help' to see available commands.");
        }
    }

    Ok(())
}