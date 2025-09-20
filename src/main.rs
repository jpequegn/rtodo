use anyhow::Result;
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use std::io::{self, Write};
use std::path::PathBuf;

mod models;
use models::{Priority, TaskUpdate, TodoList};

#[derive(Parser)]
#[command(name = "rtodo")]
#[command(about = "A simple and efficient todo list CLI written in Rust")]
#[command(version, author)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Custom config file path
    #[arg(short = 'f', long = "file", global = true)]
    config_file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, ValueEnum)]
enum PriorityArg {
    Low,
    Medium,
    High,
}

impl From<PriorityArg> for Priority {
    fn from(arg: PriorityArg) -> Self {
        match arg {
            PriorityArg::Low => Priority::Low,
            PriorityArg::Medium => Priority::Medium,
            PriorityArg::High => Priority::High,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new todo item
    Add {
        /// The todo item title
        title: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
        /// Optional due date (YYYY-MM-DD format)
        #[arg(short = 'D', long)]
        due: Option<String>,
        /// Optional category
        #[arg(short, long)]
        category: Option<String>,
        /// Task priority
        #[arg(short, long, value_enum, default_value = "medium")]
        priority: PriorityArg,
    },
    /// List all todo items
    List {
        /// Show only completed items
        #[arg(short, long, conflicts_with = "pending")]
        completed: bool,
        /// Show only pending items
        #[arg(short, long, conflicts_with = "completed")]
        pending: bool,
        /// Filter by category
        #[arg(short = 'C', long)]
        category: Option<String>,
        /// Filter by priority
        #[arg(short = 'P', long, value_enum)]
        priority: Option<PriorityArg>,
        /// Show overdue tasks only
        #[arg(short, long)]
        overdue: bool,
    },
    /// Mark a todo item as completed
    Complete {
        /// The ID of the todo item to complete
        id: Option<u32>,
        /// Complete all pending tasks
        #[arg(long, conflicts_with = "id")]
        all: bool,
    },
    /// Mark a todo item as incomplete
    Incomplete {
        /// The ID of the todo item to mark as incomplete
        id: u32,
    },
    /// Remove a todo item
    Remove {
        /// The ID of the todo item to remove
        id: u32,
        /// Confirm destructive operation
        #[arg(long)]
        confirm: bool,
    },
    /// Edit an existing todo item
    Edit {
        /// The ID of the todo item to edit
        id: u32,
        /// New title
        #[arg(short, long)]
        title: Option<String>,
        /// New description (use empty string to clear)
        #[arg(short, long)]
        description: Option<String>,
        /// New due date (YYYY-MM-DD format, use 'none' to clear)
        #[arg(short = 'D', long)]
        due: Option<String>,
        /// New category (use 'none' to clear)
        #[arg(short, long)]
        category: Option<String>,
        /// New priority
        #[arg(short, long, value_enum)]
        priority: Option<PriorityArg>,
        /// Mark as incomplete
        #[arg(long)]
        incomplete: bool,
    },
}

fn parse_date(date_str: &str) -> Result<DateTime<Local>> {
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    let naive_datetime = naive_date.and_hms_opt(23, 59, 59).unwrap();
    Ok(Local.from_local_datetime(&naive_datetime).unwrap())
}

fn confirm_action(message: &str) -> bool {
    print!("{} (y/N): ", message);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn load_todo_list(config_file: Option<PathBuf>) -> Result<TodoList> {
    match config_file {
        Some(path) => TodoList::load_from_file(path),
        None => TodoList::load(),
    }
}

fn save_todo_list(todo_list: &TodoList, config_file: Option<PathBuf>) -> Result<()> {
    match config_file {
        Some(path) => todo_list.save_to_file(path),
        None => todo_list.save(),
    }
}

fn print_task(task: &models::Task, verbose: bool) {
    let status_icon = if task.completed { "✓".green() } else { "○".yellow() };
    let priority_color = match task.priority {
        Priority::High => "red",
        Priority::Medium => "yellow",
        Priority::Low => "blue",
    };

    print!("{} [{}] ", status_icon, task.id.to_string().cyan());
    print!("{}", task.title.bold());

    if let Some(category) = &task.category {
        print!(" {}", format!("#{}", category).green());
    }

    println!(" {}", format!("[{}]", format!("{:?}", task.priority).to_lowercase()).color(priority_color));

    if verbose {
        if let Some(description) = &task.description {
            println!("    {}", description.dimmed());
        }
        if let Some(due_date) = task.due_date {
            let due_str = due_date.format("%Y-%m-%d").to_string();
            if task.is_overdue() {
                println!("    {}: {}", "Due".red(), due_str.red());
            } else {
                println!("    {}: {}", "Due".blue(), due_str.blue());
            }
        }
        println!("    {}: {}", "Created".dimmed(), task.created_at.format("%Y-%m-%d %H:%M").to_string().dimmed());
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load todo list
    let mut todo_list = load_todo_list(cli.config_file.clone()).unwrap_or_else(|e| {
        if cli.verbose {
            eprintln!("{}: {}", "Warning".yellow(), e);
            eprintln!("Starting with empty todo list.");
        }
        TodoList::new()
    });

    let result = match cli.command {
        Some(Commands::Add { title, description, due, category, priority }) => {
            let due_date = if let Some(due_str) = due {
                Some(parse_date(&due_str)?)
            } else {
                None
            };

            let id = todo_list.add_task_with_details(
                title.clone(),
                description,
                due_date,
                category,
                priority.into(),
            );

            println!("{} {} {}", "Added task".green().bold(), id.to_string().cyan(), title);
            save_todo_list(&todo_list, cli.config_file)
        }

        Some(Commands::List { completed, pending, category, priority, overdue }) => {
            let tasks: Vec<&models::Task> = if completed {
                todo_list.get_completed_tasks()
            } else if pending {
                todo_list.get_pending_tasks()
            } else if overdue {
                todo_list.get_overdue_tasks()
            } else {
                todo_list.get_all_tasks().iter().collect()
            };

            let filtered_tasks: Vec<&models::Task> = tasks.into_iter()
                .filter(|task| {
                    if let Some(cat) = &category {
                        task.category.as_ref().map_or(false, |c| c == cat)
                    } else {
                        true
                    }
                })
                .filter(|task| {
                    if let Some(prio) = &priority {
                        task.priority == (*prio).clone().into()
                    } else {
                        true
                    }
                })
                .collect();

            if filtered_tasks.is_empty() {
                println!("{}", "No tasks found.".dimmed());
            } else {
                println!("{} ({} tasks):", "Todo List".cyan().bold(), filtered_tasks.len());
                for task in filtered_tasks {
                    print_task(task, cli.verbose);
                }
            }
            Ok(())
        }

        Some(Commands::Complete { id, all }) => {
            if all {
                let pending_tasks = todo_list.get_pending_tasks();
                if pending_tasks.is_empty() {
                    println!("{}", "No pending tasks to complete.".dimmed());
                    return Ok(());
                }

                let count = pending_tasks.len();
                println!("Found {} pending task(s):", count);
                for task in &pending_tasks {
                    println!("  - [{}] {}", task.id, task.title);
                }

                if confirm_action(&format!("Complete all {} task(s)?", count)) {
                    let task_ids: Vec<u32> = pending_tasks.iter().map(|task| task.id).collect();
                    let mut completed_count = 0;
                    for task_id in task_ids {
                        if todo_list.mark_complete(task_id).is_ok() {
                            completed_count += 1;
                        }
                    }
                    println!("{} {} task(s)", "Completed:".green().bold(), completed_count);
                    save_todo_list(&todo_list, cli.config_file)
                } else {
                    println!("Operation cancelled.");
                    Ok(())
                }
            } else if let Some(task_id) = id {
                match todo_list.mark_complete(task_id) {
                    Ok(_) => {
                        if let Some(task) = todo_list.get_task(task_id) {
                            println!("{} {}", "Completed:".green().bold(), task.title);
                        }
                        save_todo_list(&todo_list, cli.config_file)
                    }
                    Err(e) => {
                        eprintln!("{}: {}", "Error".red().bold(), e);
                        Ok(())
                    }
                }
            } else {
                eprintln!("{}: Must specify either a task ID or use --all flag", "Error".red().bold());
                Ok(())
            }
        }

        Some(Commands::Incomplete { id }) => {
            match todo_list.mark_incomplete(id) {
                Ok(_) => {
                    if let Some(task) = todo_list.get_task(id) {
                        println!("{} {}", "Marked as incomplete:".yellow().bold(), task.title);
                    }
                    save_todo_list(&todo_list, cli.config_file)
                }
                Err(e) => {
                    eprintln!("{}: {}", "Error".red().bold(), e);
                    Ok(())
                }
            }
        }

        Some(Commands::Remove { id, confirm }) => {
            if let Some(task) = todo_list.get_task(id) {
                let should_remove = if confirm {
                    true
                } else {
                    confirm_action(&format!("Are you sure you want to remove task [{}] '{}'?", id, task.title))
                };

                if should_remove {
                    match todo_list.remove_task(id) {
                        Some(task) => {
                            println!("{} {}", "Removed:".red().bold(), task.title);
                            save_todo_list(&todo_list, cli.config_file)
                        }
                        None => {
                            eprintln!("{}: Task with ID {} not found", "Error".red().bold(), id);
                            Ok(())
                        }
                    }
                } else {
                    println!("Remove operation cancelled.");
                    Ok(())
                }
            } else {
                eprintln!("{}: Task with ID {} not found", "Error".red().bold(), id);
                Ok(())
            }
        }

        Some(Commands::Edit { id, title, description, due, category, priority, incomplete }) => {
            let mut update = TaskUpdate::new();

            if let Some(new_title) = title {
                update = update.title(new_title);
            }

            if let Some(desc) = description {
                update = update.description(if desc == "none" || desc.is_empty() {
                    None
                } else {
                    Some(desc)
                });
            }

            if let Some(due_str) = due {
                if due_str == "none" {
                    update = update.due_date(None);
                } else {
                    let due_date = parse_date(&due_str)?;
                    update = update.due_date(Some(due_date));
                }
            }

            if let Some(cat) = category {
                update = update.category(if cat == "none" {
                    None
                } else {
                    Some(cat)
                });
            }

            if let Some(prio) = priority {
                update = update.priority(prio.into());
            }

            match todo_list.update_task(id, update) {
                Ok(_) => {
                    if incomplete {
                        todo_list.mark_incomplete(id)?;
                    }
                    if let Some(task) = todo_list.get_task(id) {
                        println!("{} {}", "Updated:".blue().bold(), task.title);
                    }
                    save_todo_list(&todo_list, cli.config_file)
                }
                Err(e) => {
                    eprintln!("{}: {}", "Error".red().bold(), e);
                    Ok(())
                }
            }
        }

        None => {
            println!("{}", "Welcome to rtodo!".cyan().bold());
            println!("Use 'rtodo --help' to see available commands.");

            let total = todo_list.len();
            let completed = todo_list.get_completed_tasks().len();
            let pending = todo_list.get_pending_tasks().len();
            let overdue = todo_list.get_overdue_tasks().len();

            if total > 0 {
                println!();
                println!("Summary: {} total, {} completed, {} pending",
                    total.to_string().cyan(),
                    completed.to_string().green(),
                    pending.to_string().yellow()
                );
                if overdue > 0 {
                    println!("  {} overdue tasks!", overdue.to_string().red().bold());
                }
            }

            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{}: {}", "Error".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}