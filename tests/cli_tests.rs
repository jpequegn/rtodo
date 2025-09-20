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

    // Remove the task
    let output = env.run_rtodo(&["remove", "1"])
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
    assert!(stdout.contains("Updated:"));
    assert!(stdout.contains("Updated title"));
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
    assert!(stdout.contains("Updated:"));
    assert!(stdout.contains("Edited task"));
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
    assert!(stdout.contains("USAGE:"));
    assert!(stdout.contains("COMMANDS:"));
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
    assert!(stdout.contains("Updated:"));
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
    assert!(stdout.contains("Updated:"));
}