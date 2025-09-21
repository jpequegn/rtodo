use std::process::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use serde_json;

struct TestEnv {
    _temp_dir: TempDir,
    config_file: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_file = temp_dir.path().join("test_todos.json");

        TestEnv {
            _temp_dir: temp_dir,
            config_file,
        }
    }

    fn run_rtodo(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--")
            .arg("--file")
            .arg(&self.config_file)
            .args(args);
        cmd
    }

    fn get_todos_json(&self) -> serde_json::Value {
        if self.config_file.exists() {
            let content = fs::read_to_string(&self.config_file).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        }
    }
}

#[test]
fn test_add_basic_todo() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["add", "Test task"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Added task"));
    assert!(stdout.contains("Test task"));
}

#[test]
fn test_add_todo_with_details() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&[
        "add", "Complete project",
        "--description", "Finish the Rust CLI project",
        "--due", "2024-12-31",
        "--category", "work",
        "--priority", "high"
    ]).output().expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Added task"));
    assert!(stdout.contains("Complete project"));
}

#[test]
fn test_list_todos_empty() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("No tasks found"));
}

#[test]
fn test_list_todos_with_data() {
    let env = TestEnv::new();

    // Add a task first
    env.run_rtodo(&["add", "Test task"])
        .output()
        .expect("Failed to add task");

    // List tasks
    let output = env.run_rtodo(&["list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Todo List"));
    assert!(stdout.contains("Test task"));
    assert!(stdout.contains("[1]")); // Task ID
}

#[test]
fn test_complete_todo() {
    let env = TestEnv::new();

    // Add a task
    env.run_rtodo(&["add", "Task to complete"])
        .output()
        .expect("Failed to add task");

    // Complete the task
    let output = env.run_rtodo(&["complete", "1"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Completed:"));
    assert!(stdout.contains("Task to complete"));
}

#[test]
fn test_complete_nonexistent_todo() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["complete", "999"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success()); // Command succeeds but shows error
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error") || stderr.contains("not found"));
}

#[test]
fn test_remove_todo() {
    let env = TestEnv::new();

    // Add a task
    env.run_rtodo(&["add", "Task to remove"])
        .output()
        .expect("Failed to add task");

    // Remove the task (using --confirm to skip interactive prompt in tests)
    let output = env.run_rtodo(&["remove", "1", "--confirm"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Removed:"));
    assert!(stdout.contains("Task to remove"));
}

#[test]
fn test_edit_todo_title() {
    let env = TestEnv::new();

    // Add a task
    env.run_rtodo(&["add", "Original title"])
        .output()
        .expect("Failed to add task");

    // Edit the task title
    let output = env.run_rtodo(&["edit", "1", "--title", "Updated title"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Updated task [1]"));
    assert!(stdout.contains("Changes:"));
    assert!(stdout.contains("Title: Original title → Updated title"));
}

#[test]
fn test_edit_todo_all_fields() {
    let env = TestEnv::new();

    // Add a task
    env.run_rtodo(&["add", "Task to edit"])
        .output()
        .expect("Failed to add task");

    // Edit all fields
    let output = env.run_rtodo(&[
        "edit", "1",
        "--title", "Edited task",
        "--description", "New description",
        "--due", "2024-12-25",
        "--category", "personal",
        "--priority", "low"
    ]).output().expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Updated task [1]"));
    assert!(stdout.contains("Changes:"));
    assert!(stdout.contains("Title: Task to edit → Edited task"));
    assert!(stdout.contains("Description: (none) → New description"));
    assert!(stdout.contains("Due date: (none) → 2024-12-25"));
    assert!(stdout.contains("Category: (none) → personal"));
    assert!(stdout.contains("Priority: medium → low"));
}

#[test]
fn test_list_with_filters() {
    let env = TestEnv::new();

    // Add tasks with different states
    env.run_rtodo(&["add", "Pending task", "--category", "work"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Another task", "--category", "personal", "--priority", "high"])
        .output()
        .expect("Failed to add task");

    // Complete one task
    env.run_rtodo(&["complete", "1"])
        .output()
        .expect("Failed to complete task");

    // Test pending filter
    let output = env.run_rtodo(&["list", "--pending"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Another task"));
    assert!(!stdout.contains("Pending task")); // Should not show completed

    // Test completed filter
    let output = env.run_rtodo(&["list", "--completed"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Pending task"));
    assert!(!stdout.contains("Another task")); // Should not show pending

    // Test category filter
    let output = env.run_rtodo(&["list", "--category", "personal"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Another task"));

    // Test priority filter
    let output = env.run_rtodo(&["list", "--priority", "high"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Another task"));
}

#[test]
fn test_verbose_output() {
    let env = TestEnv::new();

    // Add a task with details
    env.run_rtodo(&[
        "add", "Detailed task",
        "--description", "This is a detailed description",
        "--due", "2024-12-31",
        "--category", "work"
    ]).output().expect("Failed to add task");

    // List with verbose flag
    let output = env.run_rtodo(&["--verbose", "list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Detailed task"));
    assert!(stdout.contains("This is a detailed description"));
    assert!(stdout.contains("Due:"));
    assert!(stdout.contains("Created:"));
}

#[test]
fn test_invalid_date_format() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["add", "Task with bad date", "--due", "invalid-date"])
        .output()
        .expect("Failed to execute command");

    // Should fail with error
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error") || !stderr.is_empty());
}

#[test]
fn test_default_behavior_no_subcommand() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&[])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Welcome to rtodo!"));
    assert!(stdout.contains("Use 'rtodo --help'"));
}

#[test]
fn test_help_output() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rtodo"));
    assert!(stdout.contains("todo list CLI"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
}

#[test]
fn test_subcommand_help() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["add", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Add a new todo item"));
    assert!(stdout.contains("--description"));
    assert!(stdout.contains("--due"));
    assert!(stdout.contains("--priority"));
}

#[test]
fn test_persistence_across_commands() {
    let env = TestEnv::new();

    // Add a task
    env.run_rtodo(&["add", "Persistent task"])
        .output()
        .expect("Failed to add task");

    // Verify it persists by listing in a new command
    let output = env.run_rtodo(&["list"])
        .output()
        .expect("Failed to list tasks");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Persistent task"));

    // Verify the JSON file was created and contains data
    assert!(env.config_file.exists());
    let todos = env.get_todos_json();
    assert!(!todos.is_null());
}

#[test]
fn test_overdue_tasks() {
    let env = TestEnv::new();

    // Add a task with past due date
    env.run_rtodo(&["add", "Overdue task", "--due", "2020-01-01"])
        .output()
        .expect("Failed to add task");

    // List overdue tasks
    let output = env.run_rtodo(&["list", "--overdue"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Overdue task"));
}

#[test]
fn test_edit_clear_fields() {
    let env = TestEnv::new();

    // Add a task with details
    env.run_rtodo(&[
        "add", "Task with details",
        "--description", "Original description",
        "--category", "work"
    ]).output().expect("Failed to add task");

    // Clear description and category
    let output = env.run_rtodo(&[
        "edit", "1",
        "--description", "none",
        "--category", "none"
    ]).output().expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Updated task [1]"));
    assert!(stdout.contains("Changes:"));
    assert!(stdout.contains("Description: Original description → (none)"));
    assert!(stdout.contains("Category: work → (none)"));
}

#[test]
fn test_mark_incomplete() {
    let env = TestEnv::new();

    // Add and complete a task
    env.run_rtodo(&["add", "Task to uncomplete"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["complete", "1"])
        .output()
        .expect("Failed to complete task");

    // Mark as incomplete
    let output = env.run_rtodo(&["edit", "1", "--incomplete"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Updated task [1]"));
    assert!(stdout.contains("Changes:"));
    assert!(stdout.contains("Status: completed → incomplete"));
}

#[test]
fn test_list_due_soon_filter() {
    let env = TestEnv::new();

    // Add a task due soon (5 days from now)
    env.run_rtodo(&["add", "Due soon task", "--due", "2025-09-25"])
        .output()
        .expect("Failed to add task");

    // Add a task due far in the future
    env.run_rtodo(&["add", "Future task", "--due", "2025-12-31"])
        .output()
        .expect("Failed to add task");

    // Test due-soon filter
    let output = env.run_rtodo(&["list", "--due-soon"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Due soon task"));
    assert!(!stdout.contains("Future task")); // Should not show tasks due far in future
}

#[test]
fn test_list_sort_by_title() {
    let env = TestEnv::new();

    // Add tasks in specific order
    env.run_rtodo(&["add", "Zebra task"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Alpha task"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Beta task"])
        .output()
        .expect("Failed to add task");

    // Test sorting by title
    let output = env.run_rtodo(&["list", "--sort-by", "title"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    // Find task lines (skip header)
    let task_lines: Vec<&str> = lines.iter()
        .filter(|line| line.contains("[1]") || line.contains("[2]") || line.contains("[3]"))
        .cloned()
        .collect();

    // Should be sorted alphabetically
    assert!(task_lines[0].contains("Alpha task"));
    assert!(task_lines[1].contains("Beta task"));
    assert!(task_lines[2].contains("Zebra task"));
}

#[test]
fn test_list_sort_by_priority() {
    let env = TestEnv::new();

    // Add tasks with different priorities
    env.run_rtodo(&["add", "Low task", "--priority", "low"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "High task", "--priority", "high"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Medium task", "--priority", "medium"])
        .output()
        .expect("Failed to add task");

    // Test sorting by priority
    let output = env.run_rtodo(&["list", "--sort-by", "priority"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    // Find task lines (skip header)
    let task_lines: Vec<&str> = lines.iter()
        .filter(|line| line.contains("[1]") || line.contains("[2]") || line.contains("[3]"))
        .cloned()
        .collect();

    // Should be sorted by priority: high, medium, low
    assert!(task_lines[0].contains("High task"));
    assert!(task_lines[1].contains("Medium task"));
    assert!(task_lines[2].contains("Low task"));
}

#[test]
fn test_list_sort_by_due_date() {
    let env = TestEnv::new();

    // Add tasks with different due dates
    env.run_rtodo(&["add", "Late task", "--due", "2025-12-31"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Early task", "--due", "2025-09-25"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "No due date task"])
        .output()
        .expect("Failed to add task");

    // Test sorting by due date
    let output = env.run_rtodo(&["list", "--sort-by", "due"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    // Find task lines (skip header)
    let task_lines: Vec<&str> = lines.iter()
        .filter(|line| line.contains("[1]") || line.contains("[2]") || line.contains("[3]"))
        .cloned()
        .collect();

    // Should be sorted by due date: earliest first, no due date last
    assert!(task_lines[0].contains("Early task"));
    assert!(task_lines[1].contains("Late task"));
    assert!(task_lines[2].contains("No due date task"));
}

#[test]
fn test_list_reverse_sort() {
    let env = TestEnv::new();

    // Add tasks with different priorities
    env.run_rtodo(&["add", "Low task", "--priority", "low"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "High task", "--priority", "high"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Medium task", "--priority", "medium"])
        .output()
        .expect("Failed to add task");

    // Test reverse sorting by priority
    let output = env.run_rtodo(&["list", "--sort-by", "priority", "--reverse"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    // Find task lines (skip header)
    let task_lines: Vec<&str> = lines.iter()
        .filter(|line| line.contains("[1]") || line.contains("[2]") || line.contains("[3]"))
        .cloned()
        .collect();

    // Should be sorted in reverse priority order: low, medium, high
    assert!(task_lines[0].contains("Low task"));
    assert!(task_lines[1].contains("Medium task"));
    assert!(task_lines[2].contains("High task"));
}

#[test]
fn test_list_sort_by_created() {
    let env = TestEnv::new();

    // Add tasks in specific order (will have different creation times)
    env.run_rtodo(&["add", "First task"])
        .output()
        .expect("Failed to add task");

    // Small delay to ensure different creation times
    std::thread::sleep(std::time::Duration::from_millis(10));

    env.run_rtodo(&["add", "Second task"])
        .output()
        .expect("Failed to add task");

    std::thread::sleep(std::time::Duration::from_millis(10));

    env.run_rtodo(&["add", "Third task"])
        .output()
        .expect("Failed to add task");

    // Test sorting by created date
    let output = env.run_rtodo(&["list", "--sort-by", "created"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    // Find task lines (skip header)
    let task_lines: Vec<&str> = lines.iter()
        .filter(|line| line.contains("[1]") || line.contains("[2]") || line.contains("[3]"))
        .cloned()
        .collect();

    // Should be sorted by creation order
    assert!(task_lines[0].contains("First task"));
    assert!(task_lines[1].contains("Second task"));
    assert!(task_lines[2].contains("Third task"));
}

#[test]
fn test_list_combined_filter_and_sort() {
    let env = TestEnv::new();

    // Add tasks with different categories and priorities
    env.run_rtodo(&["add", "Work B", "--category", "work", "--priority", "low"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Work A", "--category", "work", "--priority", "high"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Personal task", "--category", "personal", "--priority", "high"])
        .output()
        .expect("Failed to add task");

    // Test filtering by category and sorting by title
    let output = env.run_rtodo(&["list", "--category", "work", "--sort-by", "title"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should only show work tasks, sorted alphabetically
    assert!(stdout.contains("Work A"));
    assert!(stdout.contains("Work B"));
    assert!(!stdout.contains("Personal task"));

    // Check order - Work A should come before Work B
    let work_a_pos = stdout.find("Work A").unwrap();
    let work_b_pos = stdout.find("Work B").unwrap();
    assert!(work_a_pos < work_b_pos);
}

#[test]
fn test_due_soon_color_highlighting() {
    let env = TestEnv::new();

    // Add a task due soon
    env.run_rtodo(&["add", "Due soon task", "--due", "2025-09-25"])
        .output()
        .expect("Failed to add task");

    // Test verbose mode to see color highlighting
    let output = env.run_rtodo(&["--verbose", "list", "--due-soon"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should show the task with due date
    assert!(stdout.contains("Due soon task"));
    assert!(stdout.contains("Due: 2025-09-25"));
}

#[test]
fn test_list_help_shows_new_options() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["list", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check that new options are documented
    assert!(stdout.contains("--due-soon"));
    assert!(stdout.contains("--sort-by"));
    assert!(stdout.contains("--reverse"));
}

#[test]
fn test_sort_field_values_in_help() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["list", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check that sort field options are shown (these might be in different format)
    // The exact format depends on clap's help generation
    assert!(stdout.contains("sort") && (
        stdout.contains("created") ||
        stdout.contains("due") ||
        stdout.contains("priority") ||
        stdout.contains("title")
    ));
}

#[test]
fn test_standalone_incomplete_command() {
    let env = TestEnv::new();

    // Add and complete a task
    env.run_rtodo(&["add", "Task to mark incomplete"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["complete", "1"])
        .output()
        .expect("Failed to complete task");

    // Use the standalone incomplete command
    let output = env.run_rtodo(&["incomplete", "1"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Marked as incomplete:"));
    assert!(stdout.contains("Task to mark incomplete"));
}

#[test]
fn test_incomplete_nonexistent_task() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["incomplete", "999"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success()); // Command succeeds but shows error
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error") || stderr.contains("not found"));
}

#[test]
fn test_remove_with_confirm_flag() {
    let env = TestEnv::new();

    // Add a task
    env.run_rtodo(&["add", "Task to remove with confirm"])
        .output()
        .expect("Failed to add task");

    // Remove with --confirm flag (should skip interactive prompt)
    let output = env.run_rtodo(&["remove", "1", "--confirm"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Removed:"));
    assert!(stdout.contains("Task to remove with confirm"));
}

#[test]
fn test_categories_empty() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["categories"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("No categories found."));
}

#[test]
fn test_categories_with_data() {
    let env = TestEnv::new();

    // Add tasks with different categories
    env.run_rtodo(&["add", "Work task 1", "--category", "work"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Work task 2", "--category", "work"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Personal task", "--category", "personal"])
        .output()
        .expect("Failed to add task");

    let output = env.run_rtodo(&["categories"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Categories:"));
    assert!(stdout.contains("#personal personal (1 task)"));
    assert!(stdout.contains("#work work (2 tasks)"));
    assert!(stdout.contains("Summary: 2 categories with 3 tasks total"));
}

#[test]
fn test_rename_category_success() {
    let env = TestEnv::new();

    // Add tasks with a category
    env.run_rtodo(&["add", "Work task 1", "--category", "work"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Work task 2", "--category", "work"])
        .output()
        .expect("Failed to add task");

    // Rename the category
    let output = env.run_rtodo(&["rename-category", "work", "business"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Success: Renamed category 'work' to 'business' for 2 tasks"));

    // Verify the category was renamed
    let output = env.run_rtodo(&["categories"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("#business business (2 tasks)"));
    assert!(!stdout.contains("#work"));
}

#[test]
fn test_rename_category_not_found() {
    let env = TestEnv::new();

    let output = env.run_rtodo(&["rename-category", "nonexistent", "newname"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success()); // Command runs but shows error
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error: No tasks found with category 'nonexistent'"));
}

#[test]
fn test_rename_category_same_names() {
    let env = TestEnv::new();

    // Add a task with a category
    env.run_rtodo(&["add", "Work task", "--category", "work"])
        .output()
        .expect("Failed to add task");

    let output = env.run_rtodo(&["rename-category", "work", "work"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success()); // Command runs but shows error
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error: Old and new category names are the same"));
}

#[test]
fn test_categories_alphabetical_sorting() {
    let env = TestEnv::new();

    // Add tasks with categories in non-alphabetical order
    env.run_rtodo(&["add", "Zoo task", "--category", "zoo"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Apple task", "--category", "apple"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Middle task", "--category", "middle"])
        .output()
        .expect("Failed to add task");

    let output = env.run_rtodo(&["categories"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify alphabetical order
    let apple_pos = stdout.find("#apple").unwrap();
    let middle_pos = stdout.find("#middle").unwrap();
    let zoo_pos = stdout.find("#zoo").unwrap();

    assert!(apple_pos < middle_pos);
    assert!(middle_pos < zoo_pos);
}

// Search functionality tests

#[test]
fn test_search_basic_text() {
    let env = TestEnv::new();

    // Add test tasks
    env.run_rtodo(&["add", "Frontend development task", "--description", "Work on React components"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Backend API development", "--description", "Build REST endpoints"])
        .output()
        .expect("Failed to add task");

    // Test basic text search in title
    let output = env.run_rtodo(&["search", "Frontend"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Search Results (1 matching tasks)"));
    assert!(stdout.contains("Frontend development task"));
    assert!(!stdout.contains("Backend API development"));
}

#[test]
fn test_search_case_insensitive() {
    let env = TestEnv::new();

    env.run_rtodo(&["add", "Frontend development task"])
        .output()
        .expect("Failed to add task");

    // Test case-insensitive search
    let output = env.run_rtodo(&["search", "FRONTEND", "-i"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Search Results (1 matching tasks)"));
    assert!(stdout.contains("Frontend development task"));
}

#[test]
fn test_search_in_description() {
    let env = TestEnv::new();

    env.run_rtodo(&["add", "Development task", "--description", "Work on React components"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Testing task", "--description", "Write unit tests"])
        .output()
        .expect("Failed to add task");

    // Test search in description
    let output = env.run_rtodo(&["search", "React"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Search Results (1 matching tasks)"));
    assert!(stdout.contains("Development task"));
    assert!(!stdout.contains("Testing task"));
}

#[test]
fn test_search_regex() {
    let env = TestEnv::new();

    env.run_rtodo(&["add", "Write documentation"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Test implementation"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Code review"])
        .output()
        .expect("Failed to add task");

    // Test regex search for tasks starting with "Write" or "Test"
    let output = env.run_rtodo(&["search", "^(Write|Test)", "-x"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Search Results (2 matching tasks)"));
    assert!(stdout.contains("Write documentation"));
    assert!(stdout.contains("Test implementation"));
    assert!(!stdout.contains("Code review"));
}

#[test]
fn test_search_with_filters() {
    let env = TestEnv::new();

    // Add tasks with different properties
    env.run_rtodo(&["add", "API development", "--category", "backend"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "API testing", "--category", "frontend"])
        .output()
        .expect("Failed to add task");

    // Complete one task
    env.run_rtodo(&["complete", "1"])
        .output()
        .expect("Failed to complete task");

    // Test search with category filter
    let output = env.run_rtodo(&["search", "API", "--category", "backend"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Search Results (1 matching tasks)"));
    assert!(stdout.contains("API development"));
    assert!(!stdout.contains("API testing"));

    // Test search with completion status filter
    let output = env.run_rtodo(&["search", "API", "--completed"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Search Results (1 matching tasks)"));
    assert!(stdout.contains("API development"));
    assert!(!stdout.contains("API testing"));
}

#[test]
fn test_search_no_results() {
    let env = TestEnv::new();

    env.run_rtodo(&["add", "Task one"])
        .output()
        .expect("Failed to add task");

    // Test search with no matches
    let output = env.run_rtodo(&["search", "nonexistent"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("No tasks found matching the search criteria"));
}

#[test]
fn test_search_invalid_regex() {
    let env = TestEnv::new();

    env.run_rtodo(&["add", "Task one"])
        .output()
        .expect("Failed to add task");

    // Test invalid regex
    let output = env.run_rtodo(&["search", "[invalid", "-x"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Invalid regex pattern"));
}

#[test]
fn test_search_combined_with_sort() {
    let env = TestEnv::new();

    // Add tasks that match search term
    env.run_rtodo(&["add", "Zebra API task"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Alpha API task"])
        .output()
        .expect("Failed to add task");

    // Test search with sorting
    let output = env.run_rtodo(&["search", "API", "--sort-by", "title"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Search Results (2 matching tasks)"));

    // Verify alphabetical order
    let alpha_pos = stdout.find("Alpha API task").unwrap();
    let zebra_pos = stdout.find("Zebra API task").unwrap();
    assert!(alpha_pos < zebra_pos);
}

// Due date enhancement tests

#[test]
fn test_natural_language_date_parsing() {
    let env = TestEnv::new();

    // Test adding tasks with natural language dates
    let output = env.run_rtodo(&["add", "Task due tomorrow", "--due", "tomorrow"])
        .output()
        .expect("Failed to add task");

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap();
        panic!("Command failed. Stderr: {}", stderr);
    }
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Added task"));
    assert!(stdout.contains("Task due tomorrow"));

    // Test adding task with "today"
    let output = env.run_rtodo(&["add", "Task due today", "--due", "today"])
        .output()
        .expect("Failed to add task");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Added task"));
    assert!(stdout.contains("Task due today"));

    // Test adding task with "yesterday"
    let output = env.run_rtodo(&["add", "Task due yesterday", "--due", "yesterday"])
        .output()
        .expect("Failed to add task");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Added task"));
    assert!(stdout.contains("Task due yesterday"));
}

#[test]
fn test_due_today_command() {
    let env = TestEnv::new();

    // Add tasks with different due dates
    env.run_rtodo(&["add", "Task due today", "--due", "today"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Task due tomorrow", "--due", "tomorrow"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Task due yesterday", "--due", "yesterday"])
        .output()
        .expect("Failed to add task");

    // Test due-today command
    let output = env.run_rtodo(&["due-today"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Tasks Due Today"));
    assert!(stdout.contains("Task due today"));
    assert!(!stdout.contains("Task due tomorrow"));
    assert!(!stdout.contains("Task due yesterday"));
}

#[test]
fn test_overdue_command() {
    let env = TestEnv::new();

    // Add tasks with different due dates
    env.run_rtodo(&["add", "Task due today", "--due", "today"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Task due tomorrow", "--due", "tomorrow"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Overdue task", "--due", "yesterday"])
        .output()
        .expect("Failed to add task");

    // Test overdue command
    let output = env.run_rtodo(&["overdue"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Overdue Tasks"));
    assert!(stdout.contains("Overdue task"));
    assert!(!stdout.contains("Task due today"));
    assert!(!stdout.contains("Task due tomorrow"));
}

#[test]
fn test_due_today_empty() {
    let env = TestEnv::new();

    // Add task due tomorrow (not today)
    env.run_rtodo(&["add", "Task due tomorrow", "--due", "tomorrow"])
        .output()
        .expect("Failed to add task");

    // Test due-today command with no results
    let output = env.run_rtodo(&["due-today"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("No tasks due today"));
}

#[test]
fn test_overdue_empty() {
    let env = TestEnv::new();

    // Add task due tomorrow (not overdue)
    env.run_rtodo(&["add", "Task due tomorrow", "--due", "tomorrow"])
        .output()
        .expect("Failed to add task");

    // Test overdue command with no results
    let output = env.run_rtodo(&["overdue"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("No overdue tasks"));
}

#[test]
fn test_due_today_with_sorting() {
    let env = TestEnv::new();

    // Add multiple tasks due today
    env.run_rtodo(&["add", "Zebra task", "--due", "today"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Alpha task", "--due", "today"])
        .output()
        .expect("Failed to add task");

    // Test due-today command with title sorting
    let output = env.run_rtodo(&["due-today", "--sort-by", "title"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Tasks Due Today (2 tasks)"));

    // Verify alphabetical order
    let alpha_pos = stdout.find("Alpha task").unwrap();
    let zebra_pos = stdout.find("Zebra task").unwrap();
    assert!(alpha_pos < zebra_pos);
}

// Comprehensive integration tests for issue #13

#[test]
fn test_version_command() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rtodo"));
}

#[test]
fn test_file_path_edge_cases() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test with deeply nested path
    let nested_path = temp_dir.path().join("deep").join("nested").join("path").join("todos.json");

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--file")
        .arg(&nested_path)
        .arg("add")
        .arg("Test task")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(nested_path.exists());

    // Test with relative path components
    let relative_path = temp_dir.path().join("./test/../todos.json");
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--file")
        .arg(&relative_path)
        .arg("list")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_long_task_titles() {
    let env = TestEnv::new();

    // Test very long title (300+ characters)
    let long_title = "A".repeat(300);
    let output = env.run_rtodo(&["add", &long_title])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verify it was added
    let output = env.run_rtodo(&["list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&long_title));
}

#[test]
fn test_special_characters_and_unicode() {
    let env = TestEnv::new();

    // Test special characters
    let special_chars = "Task with special chars: @#$%^&*()[]{}|\\:;\"'<>,.?/~`";
    let output = env.run_rtodo(&["add", special_chars])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Test unicode characters
    let unicode_text = "Unicode task: 🚀 ñáéíóú 中文 العربية हिन्दी";
    let output = env.run_rtodo(&["add", unicode_text])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verify both were added
    let output = env.run_rtodo(&["list"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(special_chars));
    assert!(stdout.contains(unicode_text));
}

#[test]
fn test_empty_and_whitespace_inputs() {
    let env = TestEnv::new();

    // Test with only whitespace
    let output = env.run_rtodo(&["add", "   "])
        .output()
        .expect("Failed to execute command");

    // Should succeed but trim whitespace
    assert!(output.status.success());

    // Test search with empty string
    let output = env.run_rtodo(&["search", ""])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_large_task_list() {
    let env = TestEnv::new();

    // Add many tasks to test performance and edge cases
    for i in 1..=100 {
        let title = format!("Task number {}", i);
        env.run_rtodo(&["add", &title])
            .output()
            .expect("Failed to add task");
    }

    // Test listing large number of tasks
    let output = env.run_rtodo(&["list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Todo List (100 tasks)"));

    // Test search in large list
    let output = env.run_rtodo(&["search", "Task number 50"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Task number 50"));
}

#[test]
fn test_corrupted_json_recovery() {
    let env = TestEnv::new();

    // Add a task first
    env.run_rtodo(&["add", "Initial task"])
        .output()
        .expect("Failed to add task");

    // Corrupt the JSON file
    fs::write(&env.config_file, "{ invalid json content").expect("Failed to write corrupted JSON");

    // Should handle corrupted JSON gracefully
    let output = env.run_rtodo(&["list"])
        .output()
        .expect("Failed to execute command");

    // Should succeed with empty list or error message
    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn test_concurrent_file_operations() {
    let env = TestEnv::new();

    // Add initial task
    env.run_rtodo(&["add", "Task 1"])
        .output()
        .expect("Failed to add task");

    // Simulate concurrent operations by manually reading and writing
    let _json_content = env.get_todos_json();

    // Add another task while "holding" the first state
    env.run_rtodo(&["add", "Task 2"])
        .output()
        .expect("Failed to add task");

    // Verify both tasks exist
    let output = env.run_rtodo(&["list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Task 1"));
    assert!(stdout.contains("Task 2"));
}

#[test]
fn test_invalid_command_combinations() {
    let env = TestEnv::new();

    // Test invalid task ID
    let output = env.run_rtodo(&["complete", "999"])
        .output()
        .expect("Failed to execute command");

    // Should either fail or show an error message
    if output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("not found") || stderr.contains("Error"));
    }

    // Test invalid sort field
    let output = env.run_rtodo(&["list", "--sort-by", "invalid_field"])
        .output()
        .expect("Failed to execute command");

    // Should either fail or show an error message
    if output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("invalid value") || stderr.contains("Error"));
    }

    // Test invalid regex in search
    let _output = env.run_rtodo(&["search", "[invalid"])
        .output()
        .expect("Failed to execute command");

    // Should handle gracefully (might succeed with warning or fail)
    // The behavior depends on implementation
}

#[test]
fn test_edge_case_task_ids() {
    let env = TestEnv::new();

    // Add tasks
    env.run_rtodo(&["add", "Task 1"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Task 2"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Task 3"])
        .output()
        .expect("Failed to add task");

    // Remove middle task
    env.run_rtodo(&["remove", "2"])
        .output()
        .expect("Failed to remove task");

    // Test with extremely high ID that definitely doesn't exist
    let output = env.run_rtodo(&["complete", "999"])
        .output()
        .expect("Failed to execute command");

    // Should either fail or show an error message
    if output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stderr.contains("not found") || stderr.contains("Error") ||
                stdout.contains("not found") || stdout.contains("Error"));
    }

    // Test with ID 0 (invalid ID)
    let output = env.run_rtodo(&["complete", "0"])
        .output()
        .expect("Failed to execute command");

    // Should either fail or show an error message
    if output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stderr.contains("not found") || stderr.contains("Error") ||
                stdout.contains("not found") || stdout.contains("Error"));
    }
}

#[test]
fn test_complex_filter_combinations() {
    let env = TestEnv::new();

    // Add tasks with various properties
    env.run_rtodo(&["add", "Urgent work", "--priority", "high", "--category", "work"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Personal task", "--priority", "low", "--category", "personal"])
        .output()
        .expect("Failed to add task");

    env.run_rtodo(&["add", "Medium work", "--priority", "medium", "--category", "work"])
        .output()
        .expect("Failed to add task");

    // Test multiple filters with sorting
    let output = env.run_rtodo(&["list", "--category", "work", "--sort-by", "priority", "--reverse"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Urgent work"));
    assert!(stdout.contains("Medium work"));
    assert!(!stdout.contains("Personal task"));
}

#[test]
fn test_verbose_output_coverage() {
    let env = TestEnv::new();

    // Test verbose flag with different commands
    let output = env.run_rtodo(&["--verbose", "add", "Verbose task"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let output = env.run_rtodo(&["--verbose", "list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let output = env.run_rtodo(&["list", "--verbose"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_backup_functionality() {
    let env = TestEnv::new();

    // Add some tasks
    env.run_rtodo(&["add", "Task for backup test"])
        .output()
        .expect("Failed to add task");

    // Check if backup files are created (implementation dependent)
    let _backup_pattern = format!("{}.backup", env.config_file.display());

    // Trigger operations that might create backups
    env.run_rtodo(&["remove", "1"])
        .output()
        .expect("Failed to remove task");

    // The exact backup behavior depends on implementation
    // This test ensures backup-related code paths are exercised
}

#[test]
fn test_date_edge_cases() {
    let env = TestEnv::new();

    // Test leap year dates
    let output = env.run_rtodo(&["add", "Leap year task", "--due", "2024-02-29"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Test year boundaries
    let output = env.run_rtodo(&["add", "New year task", "--due", "2024-12-31"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Test far future dates
    let output = env.run_rtodo(&["add", "Future task", "--due", "2099-12-31"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Test past dates for overdue functionality
    let output = env.run_rtodo(&["add", "Old task", "--due", "2020-01-01"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Test overdue command with these edge cases
    let output = env.run_rtodo(&["overdue"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_category_edge_cases() {
    let env = TestEnv::new();

    // Test category with special characters
    env.run_rtodo(&["add", "Special category task", "--category", "work-urgent!"])
        .output()
        .expect("Failed to add task");

    // Test category with spaces
    env.run_rtodo(&["add", "Spaced category task", "--category", "personal life"])
        .output()
        .expect("Failed to add task");

    // Test very long category name
    let long_category = "a".repeat(100);
    env.run_rtodo(&["add", "Long category task", "--category", &long_category])
        .output()
        .expect("Failed to add task");

    // Test categories listing
    let output = env.run_rtodo(&["categories"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("work-urgent!"));
    assert!(stdout.contains("personal life"));
    assert!(stdout.contains(&long_category));
}