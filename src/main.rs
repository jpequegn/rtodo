//! RTodo - A simple and efficient todo list CLI application written in Rust
//!
//! This module contains the main CLI interface and command handling logic for RTodo.
//! RTodo provides a clean command-line interface for managing daily tasks with support
//! for due dates, categories, priorities, completion tracking, and colorized output.
//!
//! # Features
//!
//! - Add todos with optional due dates, categories, and priorities
//! - List todos with various filtering options
//! - Search todos by text content with regex support
//! - Mark todos as complete or incomplete
//! - Edit existing todos
//! - Organize todos by categories
//! - View todos by due dates (today, overdue)
//! - Natural language date parsing ("tomorrow", "next Friday")
//! - Colorized terminal output for better readability
//!
//! # Usage
//!
//! ```bash
//! rtodo add "Buy groceries" --due tomorrow --category personal
//! rtodo list --pending --category work
//! rtodo complete 1
//! rtodo search "project" --regex
//! ```

use anyhow::Result;
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use chrono_english;
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use std::io::{self, Write};
use std::path::PathBuf;

mod models;
use models::{Priority, TaskUpdate, TodoList};

/// Main CLI structure for parsing command line arguments
///
/// This struct defines the top-level command-line interface for RTodo.
/// It includes global options like verbose output and custom config file paths,
/// as well as the subcommand to execute.
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

/// Priority levels for command line argument parsing
///
/// This enum represents the priority levels that can be specified via command line
/// arguments. It maps to the internal `Priority` enum used in the task model.
#[derive(Clone, ValueEnum)]
enum PriorityArg {
    /// Low priority task
    Low,
    /// Medium priority task (default)
    Medium,
    /// High priority task
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

/// Fields available for sorting tasks
///
/// This enum defines the different fields by which tasks can be sorted
/// when displaying lists of tasks.
#[derive(Clone, ValueEnum)]
enum SortField {
    /// Sort by task creation date
    Created,
    /// Sort by due date (tasks without due dates appear last)
    Due,
    /// Sort by priority (High -> Medium -> Low)
    Priority,
    /// Sort by task title (alphabetical)
    Title,
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
        /// Show tasks due within a week
        #[arg(short = 'd', long)]
        due_soon: bool,
        /// Sort tasks by field
        #[arg(short = 's', long, value_enum)]
        sort_by: Option<SortField>,
        /// Reverse sort order (descending)
        #[arg(short = 'r', long)]
        reverse: bool,
    },
    /// Search for todo items by text
    Search {
        /// Search query text
        query: String,
        /// Case-insensitive search
        #[arg(short = 'i', long)]
        case_insensitive: bool,
        /// Use regular expression
        #[arg(short = 'x', long)]
        regex: bool,
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
        /// Show tasks due within a week
        #[arg(short = 'd', long)]
        due_soon: bool,
        /// Sort tasks by field
        #[arg(short = 's', long, value_enum)]
        sort_by: Option<SortField>,
        /// Reverse sort order (descending)
        #[arg(short = 'r', long)]
        reverse: bool,
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
    /// List all categories with task counts
    Categories,
    /// Rename a category across all tasks
    RenameCategory {
        /// Current category name
        old_name: String,
        /// New category name
        new_name: String,
    },
    /// Show tasks due today
    DueToday {
        /// Sort tasks by field
        #[arg(short = 's', long, value_enum)]
        sort_by: Option<SortField>,
        /// Reverse sort order (descending)
        #[arg(short = 'r', long)]
        reverse: bool,
    },
    /// Show overdue tasks
    Overdue {
        /// Sort tasks by field
        #[arg(short = 's', long, value_enum)]
        sort_by: Option<SortField>,
        /// Reverse sort order (descending)
        #[arg(short = 'r', long)]
        reverse: bool,
    },
}

/// Parse a date string using natural language or ISO format
///
/// This function attempts to parse date strings in two ways:
/// 1. Natural language parsing using chrono-english (e.g., "tomorrow", "next Friday")
/// 2. ISO format parsing (YYYY-MM-DD)
///
/// All parsed dates are set to end of day (23:59:59) for consistency in due date handling.
///
/// # Arguments
///
/// * `date_str` - The date string to parse
///
/// # Returns
///
/// * `Ok(DateTime<Local>)` - Successfully parsed date set to end of day
/// * `Err(anyhow::Error)` - Parsing failed for both natural language and ISO format
///
/// # Examples
///
/// ```
/// let tomorrow = parse_date("tomorrow")?;
/// let specific = parse_date("2024-12-31")?;
/// let natural = parse_date("next Friday")?;
/// ```
fn parse_date(date_str: &str) -> Result<DateTime<Local>> {
    // First try natural language parsing
    if let Ok(parsed) = chrono_english::parse_date_string(date_str, Local::now(), chrono_english::Dialect::Us) {
        // Set time to end of day (23:59:59) for consistency
        let end_of_day = parsed.date_naive().and_hms_opt(23, 59, 59).unwrap();
        return Ok(Local.from_local_datetime(&end_of_day).unwrap());
    }

    // Fallback to the original YYYY-MM-DD format
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    let naive_datetime = naive_date.and_hms_opt(23, 59, 59).unwrap();
    Ok(Local.from_local_datetime(&naive_datetime).unwrap())
}

/// Highlight search query matches in text with colored output
///
/// This function searches for matches of a query string within text and highlights
/// them using terminal colors (bright yellow background with black text).
/// Supports both regular text search and regex pattern matching.
///
/// # Arguments
///
/// * `text` - The text to search within
/// * `query` - The search query or regex pattern
/// * `case_insensitive` - Whether to perform case-insensitive matching
/// * `use_regex` - Whether to treat the query as a regex pattern
///
/// # Returns
///
/// * `String` - The text with highlighted matches, or original text if no matches
///
/// # Examples
///
/// ```
/// let highlighted = highlight_text("Buy groceries", "Buy", false, false);
/// // Returns "Buy" with yellow background + "groceries" in normal colors
/// ```
fn highlight_text(text: &str, query: &str, case_insensitive: bool, use_regex: bool) -> String {
    use regex::Regex;
    use colored::*;

    if use_regex {
        // For regex, try to find and highlight matches
        let pattern = if case_insensitive {
            format!("(?i){}", query)
        } else {
            query.to_string()
        };

        if let Ok(re) = Regex::new(&pattern) {
            let mut result = String::new();
            let mut last_end = 0;

            for mat in re.find_iter(text) {
                result.push_str(&text[last_end..mat.start()]);
                result.push_str(&text[mat.start()..mat.end()].on_bright_yellow().black().to_string());
                last_end = mat.end();
            }
            result.push_str(&text[last_end..]);
            return result;
        }
    } else {
        // For normal text search, highlight all occurrences
        if case_insensitive {
            let lower_text = text.to_lowercase();
            let lower_query = query.to_lowercase();
            let mut result = String::new();
            let mut last_end = 0;

            for (idx, _) in lower_text.match_indices(&lower_query) {
                result.push_str(&text[last_end..idx]);
                result.push_str(&text[idx..idx + query.len()].on_bright_yellow().black().to_string());
                last_end = idx + query.len();
            }
            result.push_str(&text[last_end..]);
            return result;
        } else {
            let mut result = String::new();
            let mut last_end = 0;

            for (idx, _) in text.match_indices(query) {
                result.push_str(&text[last_end..idx]);
                result.push_str(&text[idx..idx + query.len()].on_bright_yellow().black().to_string());
                last_end = idx + query.len();
            }
            result.push_str(&text[last_end..]);
            return result;
        }
    }

    text.to_string()
}

fn print_task_with_highlight(task: &models::Task, verbose: bool, query: &str, case_insensitive: bool, use_regex: bool) {
    let status_icon = if task.completed { "✓".green() } else { "○".yellow() };
    let priority_color = match task.priority {
        Priority::High => "red",
        Priority::Medium => "yellow",
        Priority::Low => "blue",
    };

    print!("{} [{}] ", status_icon, task.id.to_string().cyan());

    // Highlight the title
    let highlighted_title = highlight_text(&task.title, query, case_insensitive, use_regex);
    print!("{}", highlighted_title.bold());

    if let Some(category) = &task.category {
        print!(" {}", format!("#{}", category).green());
    }

    // Show time until due in the main line
    if let Some(due_date) = task.due_date {
        let time_until = format_time_until_due(due_date);
        if task.is_overdue() {
            print!(" ({})", time_until.red());
        } else if task.is_due_today() {
            print!(" ({})", time_until.yellow().bold());
        } else if task.is_due_soon() {
            print!(" ({})", time_until.yellow());
        } else {
            print!(" ({})", time_until.blue());
        }
    }

    println!(" {}", format!("[{}]", format!("{:?}", task.priority).to_lowercase()).color(priority_color));

    if verbose {
        if let Some(description) = &task.description {
            // Highlight the description
            let highlighted_desc = highlight_text(description, query, case_insensitive, use_regex);
            println!("    {}", highlighted_desc.dimmed());
        }
        if let Some(due_date) = task.due_date {
            let due_str = due_date.format("%Y-%m-%d").to_string();
            let time_until = format_time_until_due(due_date);
            if task.is_overdue() {
                println!("    {}: {} ({})", "Due".red(), due_str.red(), time_until.red());
            } else if task.is_due_today() {
                println!("    {}: {} ({})", "Due".yellow().bold(), due_str.yellow().bold(), time_until.yellow().bold());
            } else if task.is_due_soon() {
                println!("    {}: {} ({})", "Due".yellow(), due_str.yellow(), time_until.yellow());
            } else {
                println!("    {}: {} ({})", "Due".blue(), due_str.blue(), time_until.blue());
            }
        }
    }
}

/// Sort a vector of task references by the specified field
///
/// This function sorts tasks by different criteria (creation date, due date, priority, or title).
/// For due date sorting, tasks without due dates appear after tasks with due dates.
/// For priority sorting, the order is High -> Medium -> Low when not reversed.
///
/// # Arguments
///
/// * `tasks` - Vector of task references to sort
/// * `sort_by` - Optional field to sort by (None means no sorting)
/// * `reverse` - Whether to reverse the sort order (descending instead of ascending)
///
/// # Returns
///
/// * `Vec<&models::Task>` - Sorted vector of task references
///
/// # Examples
///
/// ```
/// let sorted = sort_tasks(tasks, Some(SortField::Priority), false);
/// // Returns tasks sorted High -> Medium -> Low priority
///
/// let reverse_sorted = sort_tasks(tasks, Some(SortField::Due), true);
/// // Returns tasks sorted by due date, latest first
/// ```
fn sort_tasks(mut tasks: Vec<&models::Task>, sort_by: Option<SortField>, reverse: bool) -> Vec<&models::Task> {
    if let Some(field) = sort_by {
        tasks.sort_by(|a, b| {
            let ordering = match field {
                SortField::Created => a.created_at.cmp(&b.created_at),
                SortField::Due => {
                    match (a.due_date, b.due_date) {
                        (Some(a_due), Some(b_due)) => a_due.cmp(&b_due),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
                SortField::Priority => {
                    // High = 0, Medium = 1, Low = 2 for ascending priority order
                    let a_priority = match a.priority {
                        Priority::High => 0,
                        Priority::Medium => 1,
                        Priority::Low => 2,
                    };
                    let b_priority = match b.priority {
                        Priority::High => 0,
                        Priority::Medium => 1,
                        Priority::Low => 2,
                    };
                    a_priority.cmp(&b_priority)
                }
                SortField::Title => a.title.cmp(&b.title),
            };

            if reverse {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }
    tasks
}

/// Prompt the user for confirmation of a potentially destructive action
///
/// This function displays a message and waits for user input to confirm or deny
/// an action. Only explicit "y" or "yes" responses are treated as confirmation.
/// All other responses (including empty input) are treated as denial.
///
/// # Arguments
///
/// * `message` - The confirmation message to display to the user
///
/// # Returns
///
/// * `bool` - `true` if user confirmed (y/yes), `false` otherwise
///
/// # Examples
///
/// ```
/// if confirm_action("Delete all completed tasks?") {
///     // User confirmed, proceed with deletion
/// } else {
///     // User denied or gave unclear response, cancel operation
/// }
/// ```
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

/// Format a human-readable string describing time until or since due date
///
/// This function calculates the time difference between now and a due date,
/// returning a formatted string that describes the relationship in natural language.
///
/// # Arguments
///
/// * `due_date` - The due date to compare against current time
///
/// # Returns
///
/// * `String` - Human-readable description of time until/since due date
///
/// # Examples
///
/// ```
/// // If today is 2024-09-21:
/// let due_today = Local::now().date_naive().and_hms_opt(23, 59, 59).unwrap();
/// assert_eq!(format_time_until_due(Local.from_local_datetime(&due_today).unwrap()), "due today");
///
/// let due_tomorrow = due_today + Duration::days(1);
/// assert_eq!(format_time_until_due(Local.from_local_datetime(&due_tomorrow).unwrap()), "due tomorrow");
/// ```
fn format_time_until_due(due_date: DateTime<Local>) -> String {
    let now = Local::now();
    let duration = due_date.signed_duration_since(now);

    if duration.num_days() == 0 {
        "due today".to_string()
    } else if duration.num_days() == 1 {
        "due tomorrow".to_string()
    } else if duration.num_days() > 0 {
        format!("due in {} days", duration.num_days())
    } else if duration.num_days() == -1 {
        "1 day overdue".to_string()
    } else {
        format!("{} days overdue", -duration.num_days())
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

    // Show time until due in the main line
    if let Some(due_date) = task.due_date {
        let time_until = format_time_until_due(due_date);
        if task.is_overdue() {
            print!(" ({})", time_until.red());
        } else if task.is_due_today() {
            print!(" ({})", time_until.yellow().bold());
        } else if task.is_due_soon() {
            print!(" ({})", time_until.yellow());
        } else {
            print!(" ({})", time_until.blue());
        }
    }

    println!(" {}", format!("[{}]", format!("{:?}", task.priority).to_lowercase()).color(priority_color));

    if verbose {
        if let Some(description) = &task.description {
            println!("    {}", description.dimmed());
        }
        if let Some(due_date) = task.due_date {
            let due_str = due_date.format("%Y-%m-%d").to_string();
            let time_until = format_time_until_due(due_date);
            if task.is_overdue() {
                println!("    {}: {} ({})", "Due".red(), due_str.red(), time_until.red());
            } else if task.is_due_today() {
                println!("    {}: {} ({})", "Due".yellow().bold(), due_str.yellow().bold(), time_until.yellow().bold());
            } else if task.is_due_soon() {
                println!("    {}: {} ({})", "Due".yellow(), due_str.yellow(), time_until.yellow());
            } else {
                println!("    {}: {} ({})", "Due".blue(), due_str.blue(), time_until.blue());
            }
        }
        println!("    {}: {}", "Created".dimmed(), task.created_at.format("%Y-%m-%d %H:%M").to_string().dimmed());
    }
}

fn show_task_comparison(before: &models::Task, after: &models::Task) {
    let mut changes = Vec::new();

    // Compare title
    if before.title != after.title {
        changes.push(format!("  {}: {} {} {}",
            "Title".bold(),
            before.title.red(),
            "→".dimmed(),
            after.title.green()
        ));
    }

    // Compare description
    if before.description != after.description {
        let before_desc = before.description.as_deref().unwrap_or("(none)");
        let after_desc = after.description.as_deref().unwrap_or("(none)");
        changes.push(format!("  {}: {} {} {}",
            "Description".bold(),
            before_desc.red(),
            "→".dimmed(),
            after_desc.green()
        ));
    }

    // Compare due date
    if before.due_date != after.due_date {
        let before_due = before.due_date.map_or("(none)".to_string(), |d| d.format("%Y-%m-%d").to_string());
        let after_due = after.due_date.map_or("(none)".to_string(), |d| d.format("%Y-%m-%d").to_string());
        changes.push(format!("  {}: {} {} {}",
            "Due date".bold(),
            before_due.red(),
            "→".dimmed(),
            after_due.green()
        ));
    }

    // Compare category
    if before.category != after.category {
        let before_cat = before.category.as_deref().unwrap_or("(none)");
        let after_cat = after.category.as_deref().unwrap_or("(none)");
        changes.push(format!("  {}: {} {} {}",
            "Category".bold(),
            before_cat.red(),
            "→".dimmed(),
            after_cat.green()
        ));
    }

    // Compare priority
    if before.priority != after.priority {
        changes.push(format!("  {}: {} {} {}",
            "Priority".bold(),
            format!("{:?}", before.priority).to_lowercase().red(),
            "→".dimmed(),
            format!("{:?}", after.priority).to_lowercase().green()
        ));
    }

    // Compare completion status
    if before.completed != after.completed {
        let before_status = if before.completed { "completed" } else { "incomplete" };
        let after_status = if after.completed { "completed" } else { "incomplete" };
        changes.push(format!("  {}: {} {} {}",
            "Status".bold(),
            before_status.red(),
            "→".dimmed(),
            after_status.green()
        ));
    }

    if changes.is_empty() {
        println!("  {}", "No changes made".dimmed());
    } else {
        println!("{}", "Changes:".yellow().bold());
        for change in changes {
            println!("{}", change);
        }
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

        Some(Commands::List { completed, pending, category, priority, overdue, due_soon, sort_by, reverse }) => {
            let tasks: Vec<&models::Task> = if completed {
                todo_list.get_completed_tasks()
            } else if pending {
                todo_list.get_pending_tasks()
            } else if overdue {
                todo_list.get_overdue_tasks()
            } else if due_soon {
                todo_list.get_due_soon_tasks()
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

            let sorted_tasks = sort_tasks(filtered_tasks, sort_by, reverse);

            if sorted_tasks.is_empty() {
                println!("{}", "No tasks found.".dimmed());
            } else {
                println!("{} ({} tasks):", "Todo List".cyan().bold(), sorted_tasks.len());
                for task in sorted_tasks {
                    print_task(task, cli.verbose);
                }
            }
            Ok(())
        }

        Some(Commands::Search {
            query,
            case_insensitive,
            regex,
            completed,
            pending,
            category,
            priority,
            overdue,
            due_soon,
            sort_by,
            reverse
        }) => {
            // First, perform the search
            let search_results = todo_list.search_tasks(&query, case_insensitive, regex)?;

            // Then apply filters
            let filtered_tasks: Vec<&models::Task> = search_results.into_iter()
                .filter(|task| {
                    // Filter by completion status
                    if completed {
                        task.completed
                    } else if pending {
                        !task.completed
                    } else {
                        true
                    }
                })
                .filter(|task| {
                    // Filter by category
                    if let Some(cat) = &category {
                        task.category.as_ref().map_or(false, |c| c == cat)
                    } else {
                        true
                    }
                })
                .filter(|task| {
                    // Filter by priority
                    if let Some(prio) = &priority {
                        task.priority == (*prio).clone().into()
                    } else {
                        true
                    }
                })
                .filter(|task| {
                    // Filter by overdue
                    if overdue {
                        task.is_overdue()
                    } else {
                        true
                    }
                })
                .filter(|task| {
                    // Filter by due soon
                    if due_soon {
                        task.is_due_soon()
                    } else {
                        true
                    }
                })
                .collect();

            // Sort the results
            let sorted_tasks = sort_tasks(filtered_tasks, sort_by, reverse);

            // Display results
            if sorted_tasks.is_empty() {
                println!("{}", "No tasks found matching the search criteria.".dimmed());
            } else {
                println!("{} ({} matching tasks):", "Search Results".cyan().bold(), sorted_tasks.len());
                for task in sorted_tasks {
                    print_task_with_highlight(task, cli.verbose, &query, case_insensitive, regex);
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
            // Get the task before making changes for comparison
            let task_before = match todo_list.get_task(id) {
                Some(task) => task.clone(),
                None => {
                    eprintln!("{}: Task with ID {} not found", "Error".red().bold(), id);
                    return Ok(());
                }
            };

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
                    if let Some(task_after) = todo_list.get_task(id) {
                        println!("{} [{}]", "Updated task".blue().bold(), id.to_string().cyan());
                        show_task_comparison(&task_before, task_after);
                    }
                    save_todo_list(&todo_list, cli.config_file)
                }
                Err(e) => {
                    eprintln!("{}: {}", "Error".red().bold(), e);
                    Ok(())
                }
            }
        }

        Some(Commands::Categories) => {
            let categories = todo_list.get_all_categories();

            if categories.is_empty() {
                println!("{}", "No categories found.".dimmed());
            } else {
                println!("{}", "Categories:".cyan().bold());

                // Sort categories alphabetically
                let mut sorted_categories: Vec<(&String, &usize)> = categories.iter().collect();
                sorted_categories.sort_by_key(|(name, _)| name.as_str());

                for (category, count) in sorted_categories {
                    let task_word = if *count == 1 { "task" } else { "tasks" };
                    println!("  {} {} ({} {})",
                        format!("#{}", category).green(),
                        category.bold(),
                        count.to_string().cyan(),
                        task_word.dimmed()
                    );
                }

                let total_categories = categories.len();
                let total_tasks: usize = categories.values().sum();
                println!();
                println!("{} {} categories with {} {} total",
                    "Summary:".bold(),
                    total_categories.to_string().cyan(),
                    total_tasks.to_string().cyan(),
                    if total_tasks == 1 { "task" } else { "tasks" }
                );
            }
            Ok(())
        }

        Some(Commands::RenameCategory { old_name, new_name }) => {
            if old_name == new_name {
                eprintln!("{}: Old and new category names are the same", "Error".red().bold());
                return Ok(());
            }

            match todo_list.rename_category(&old_name, &new_name) {
                Ok(count) => {
                    let task_word = if count == 1 { "task" } else { "tasks" };
                    println!("{} Renamed category '{}' to '{}' for {} {}",
                        "Success:".green().bold(),
                        old_name.yellow(),
                        new_name.green(),
                        count.to_string().cyan(),
                        task_word
                    );
                    save_todo_list(&todo_list, cli.config_file)
                }
                Err(e) => {
                    eprintln!("{}: {}", "Error".red().bold(), e);
                    Ok(())
                }
            }
        }

        Some(Commands::DueToday { sort_by, reverse }) => {
            let tasks = todo_list.get_due_today_tasks();
            let sorted_tasks = sort_tasks(tasks, sort_by, reverse);

            if sorted_tasks.is_empty() {
                println!("{}", "No tasks due today.".dimmed());
            } else {
                println!("{} ({} tasks):", "Tasks Due Today".cyan().bold(), sorted_tasks.len());
                for task in sorted_tasks {
                    print_task(task, cli.verbose);
                }
            }
            Ok(())
        }

        Some(Commands::Overdue { sort_by, reverse }) => {
            let tasks = todo_list.get_overdue_tasks();
            let sorted_tasks = sort_tasks(tasks, sort_by, reverse);

            if sorted_tasks.is_empty() {
                println!("{}", "No overdue tasks.".dimmed());
            } else {
                println!("{} ({} tasks):", "Overdue Tasks".red().bold(), sorted_tasks.len());
                for task in sorted_tasks {
                    print_task(task, cli.verbose);
                }
            }
            Ok(())
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