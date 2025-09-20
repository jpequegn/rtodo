use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Priority levels for tasks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

/// Builder struct for updating task fields
#[derive(Debug, Default, Clone)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub due_date: Option<Option<DateTime<Local>>>,
    pub category: Option<Option<String>>,
    pub priority: Option<Priority>,
}

impl TaskUpdate {
    /// Create a new empty TaskUpdate
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the title
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the description
    pub fn description<S: Into<String>>(mut self, description: Option<S>) -> Self {
        self.description = Some(description.map(|s| s.into()));
        self
    }

    /// Set the due date
    pub fn due_date(mut self, due_date: Option<DateTime<Local>>) -> Self {
        self.due_date = Some(due_date);
        self
    }

    /// Set the category
    pub fn category<S: Into<String>>(mut self, category: Option<S>) -> Self {
        self.category = Some(category.map(|s| s.into()));
        self
    }

    /// Set the priority
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }
}

/// A single todo task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime<Local>,
    pub due_date: Option<DateTime<Local>>,
    pub category: Option<String>,
    pub priority: Priority,
}

impl Task {
    /// Create a new task with the given title
    pub fn new(id: u32, title: String) -> Self {
        Self {
            id,
            title,
            description: None,
            completed: false,
            created_at: Local::now(),
            due_date: None,
            category: None,
            priority: Priority::default(),
        }
    }

    /// Create a new task with additional options
    pub fn with_details(
        id: u32,
        title: String,
        description: Option<String>,
        due_date: Option<DateTime<Local>>,
        category: Option<String>,
        priority: Priority,
    ) -> Self {
        Self {
            id,
            title,
            description,
            completed: false,
            created_at: Local::now(),
            due_date,
            category,
            priority,
        }
    }

    /// Mark the task as completed
    pub fn complete(&mut self) {
        self.completed = true;
    }

    /// Mark the task as incomplete
    pub fn uncomplete(&mut self) {
        self.completed = false;
    }

    /// Check if the task is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            !self.completed && Local::now() > due_date
        } else {
            false
        }
    }
}

/// Collection of tasks with management operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    tasks: Vec<Task>,
    next_id: u32,
}

impl TodoList {
    /// Create a new empty todo list
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a new task to the list
    pub fn add_task(&mut self, title: String) -> u32 {
        let id = self.next_id;
        let task = Task::new(id, title);
        self.tasks.push(task);
        self.next_id += 1;
        id
    }

    /// Add a new task with detailed information
    pub fn add_task_with_details(
        &mut self,
        title: String,
        description: Option<String>,
        due_date: Option<DateTime<Local>>,
        category: Option<String>,
        priority: Priority,
    ) -> u32 {
        let id = self.next_id;
        let task = Task::with_details(id, title, description, due_date, category, priority);
        self.tasks.push(task);
        self.next_id += 1;
        id
    }

    /// Get a task by ID
    pub fn get_task(&self, id: u32) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == id)
    }

    /// Get a mutable reference to a task by ID
    pub fn get_task_mut(&mut self, id: u32) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.id == id)
    }

    /// Remove a task by ID
    pub fn remove_task(&mut self, id: u32) -> Option<Task> {
        if let Some(pos) = self.tasks.iter().position(|task| task.id == id) {
            Some(self.tasks.remove(pos))
        } else {
            None
        }
    }

    /// Complete a task by ID
    pub fn complete_task(&mut self, id: u32) -> bool {
        if let Some(task) = self.get_task_mut(id) {
            task.complete();
            true
        } else {
            false
        }
    }

    /// Mark a task as complete by ID with error handling
    pub fn mark_complete(&mut self, id: u32) -> Result<()> {
        if let Some(task) = self.get_task_mut(id) {
            task.complete();
            Ok(())
        } else {
            Err(anyhow!("Task with ID {} not found", id))
        }
    }

    /// Mark a task as incomplete by ID with error handling
    pub fn mark_incomplete(&mut self, id: u32) -> Result<()> {
        if let Some(task) = self.get_task_mut(id) {
            task.uncomplete();
            Ok(())
        } else {
            Err(anyhow!("Task with ID {} not found", id))
        }
    }

    /// Update a task by ID using the builder pattern
    pub fn update_task(&mut self, id: u32, updates: TaskUpdate) -> Result<()> {
        if let Some(task) = self.get_task_mut(id) {
            if let Some(title) = updates.title {
                task.title = title;
            }
            if let Some(description) = updates.description {
                task.description = description;
            }
            if let Some(due_date) = updates.due_date {
                task.due_date = due_date;
            }
            if let Some(category) = updates.category {
                task.category = category;
            }
            if let Some(priority) = updates.priority {
                task.priority = priority;
            }
            Ok(())
        } else {
            Err(anyhow!("Task with ID {} not found", id))
        }
    }

    /// Get all tasks
    pub fn get_all_tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Get completed tasks
    pub fn get_completed_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|task| task.completed).collect()
    }

    /// Get pending (incomplete) tasks
    pub fn get_pending_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|task| !task.completed).collect()
    }

    /// Get tasks by category
    pub fn get_tasks_by_category(&self, category: &str) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| {
                task.category
                    .as_ref()
                    .map_or(false, |cat| cat == category)
            })
            .collect()
    }

    /// Get tasks by priority
    pub fn get_tasks_by_priority(&self, priority: Priority) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| task.priority == priority)
            .collect()
    }

    /// Get overdue tasks
    pub fn get_overdue_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|task| task.is_overdue()).collect()
    }

    /// Get the total number of tasks
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Check if the todo list is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

impl Default for TodoList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Local};

    #[test]
    fn test_priority_default() {
        assert_eq!(Priority::default(), Priority::Medium);
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new(1, "Test task".to_string());
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Test task");
        assert!(!task.completed);
        assert_eq!(task.priority, Priority::Medium);
        assert!(task.description.is_none());
        assert!(task.due_date.is_none());
        assert!(task.category.is_none());
    }

    #[test]
    fn test_task_with_details() {
        let due_date = Local::now() + Duration::days(7);
        let task = Task::with_details(
            1,
            "Detailed task".to_string(),
            Some("This is a description".to_string()),
            Some(due_date),
            Some("work".to_string()),
            Priority::High,
        );

        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Detailed task");
        assert_eq!(task.description, Some("This is a description".to_string()));
        assert_eq!(task.due_date, Some(due_date));
        assert_eq!(task.category, Some("work".to_string()));
        assert_eq!(task.priority, Priority::High);
    }

    #[test]
    fn test_task_completion() {
        let mut task = Task::new(1, "Test task".to_string());
        assert!(!task.completed);

        task.complete();
        assert!(task.completed);

        task.uncomplete();
        assert!(!task.completed);
    }

    #[test]
    fn test_task_overdue() {
        let past_date = Local::now() - Duration::days(1);
        let future_date = Local::now() + Duration::days(1);

        let mut overdue_task = Task::with_details(
            1,
            "Overdue task".to_string(),
            None,
            Some(past_date),
            None,
            Priority::Medium,
        );
        assert!(overdue_task.is_overdue());

        // Completed tasks are not overdue
        overdue_task.complete();
        assert!(!overdue_task.is_overdue());

        let future_task = Task::with_details(
            2,
            "Future task".to_string(),
            None,
            Some(future_date),
            None,
            Priority::Medium,
        );
        assert!(!future_task.is_overdue());

        let no_due_date_task = Task::new(3, "No due date".to_string());
        assert!(!no_due_date_task.is_overdue());
    }

    #[test]
    fn test_todolist_creation() {
        let todo_list = TodoList::new();
        assert!(todo_list.is_empty());
        assert_eq!(todo_list.len(), 0);
    }

    #[test]
    fn test_todolist_add_task() {
        let mut todo_list = TodoList::new();
        let id = todo_list.add_task("First task".to_string());

        assert_eq!(id, 1);
        assert_eq!(todo_list.len(), 1);
        assert!(!todo_list.is_empty());

        let task = todo_list.get_task(id).unwrap();
        assert_eq!(task.title, "First task");
    }

    #[test]
    fn test_todolist_add_multiple_tasks() {
        let mut todo_list = TodoList::new();
        let id1 = todo_list.add_task("Task 1".to_string());
        let id2 = todo_list.add_task("Task 2".to_string());

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(todo_list.len(), 2);
    }

    #[test]
    fn test_todolist_remove_task() {
        let mut todo_list = TodoList::new();
        let id = todo_list.add_task("Task to remove".to_string());

        let removed_task = todo_list.remove_task(id);
        assert!(removed_task.is_some());
        assert_eq!(removed_task.unwrap().title, "Task to remove");
        assert_eq!(todo_list.len(), 0);

        let not_found = todo_list.remove_task(999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_todolist_complete_task() {
        let mut todo_list = TodoList::new();
        let id = todo_list.add_task("Task to complete".to_string());

        let success = todo_list.complete_task(id);
        assert!(success);

        let task = todo_list.get_task(id).unwrap();
        assert!(task.completed);

        let not_found = todo_list.complete_task(999);
        assert!(!not_found);
    }

    #[test]
    fn test_todolist_filtering() {
        let mut todo_list = TodoList::new();
        let id1 = todo_list.add_task_with_details(
            "High priority task".to_string(),
            None,
            None,
            Some("work".to_string()),
            Priority::High,
        );
        let id2 = todo_list.add_task_with_details(
            "Low priority task".to_string(),
            None,
            None,
            Some("personal".to_string()),
            Priority::Low,
        );

        todo_list.complete_task(id1);

        let completed = todo_list.get_completed_tasks();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].id, id1);

        let pending = todo_list.get_pending_tasks();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, id2);

        let work_tasks = todo_list.get_tasks_by_category("work");
        assert_eq!(work_tasks.len(), 1);
        assert_eq!(work_tasks[0].id, id1);

        let high_priority = todo_list.get_tasks_by_priority(Priority::High);
        assert_eq!(high_priority.len(), 1);
        assert_eq!(high_priority[0].id, id1);
    }

    #[test]
    fn test_task_update_builder() {
        let update = TaskUpdate::new()
            .title("Updated title")
            .description(Some("Updated description"))
            .priority(Priority::High)
            .category(Some("updated_category"));

        assert_eq!(update.title, Some("Updated title".to_string()));
        assert_eq!(update.description, Some(Some("Updated description".to_string())));
        assert_eq!(update.priority, Some(Priority::High));
        assert_eq!(update.category, Some(Some("updated_category".to_string())));
    }

    #[test]
    fn test_task_update_builder_with_none_values() {
        let update = TaskUpdate::new()
            .description(None)
            .category(None)
            .due_date(None);

        assert_eq!(update.description, Some(None));
        assert_eq!(update.category, Some(None));
        assert_eq!(update.due_date, Some(None));
    }

    #[test]
    fn test_mark_complete_success() {
        let mut todo_list = TodoList::new();
        let id = todo_list.add_task("Task to mark complete".to_string());

        let result = todo_list.mark_complete(id);
        assert!(result.is_ok());

        let task = todo_list.get_task(id).unwrap();
        assert!(task.completed);
    }

    #[test]
    fn test_mark_complete_not_found() {
        let mut todo_list = TodoList::new();
        let result = todo_list.mark_complete(999);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Task with ID 999 not found");
    }

    #[test]
    fn test_mark_incomplete_success() {
        let mut todo_list = TodoList::new();
        let id = todo_list.add_task("Task to mark incomplete".to_string());

        // First mark it complete
        todo_list.mark_complete(id).unwrap();
        assert!(todo_list.get_task(id).unwrap().completed);

        // Then mark it incomplete
        let result = todo_list.mark_incomplete(id);
        assert!(result.is_ok());

        let task = todo_list.get_task(id).unwrap();
        assert!(!task.completed);
    }

    #[test]
    fn test_mark_incomplete_not_found() {
        let mut todo_list = TodoList::new();
        let result = todo_list.mark_incomplete(999);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Task with ID 999 not found");
    }

    #[test]
    fn test_update_task_all_fields() {
        let mut todo_list = TodoList::new();
        let id = todo_list.add_task("Original task".to_string());

        let future_date = Local::now() + Duration::days(5);
        let update = TaskUpdate::new()
            .title("Updated task")
            .description(Some("Updated description"))
            .due_date(Some(future_date))
            .category(Some("updated_category"))
            .priority(Priority::High);

        let result = todo_list.update_task(id, update);
        assert!(result.is_ok());

        let task = todo_list.get_task(id).unwrap();
        assert_eq!(task.title, "Updated task");
        assert_eq!(task.description, Some("Updated description".to_string()));
        assert_eq!(task.due_date, Some(future_date));
        assert_eq!(task.category, Some("updated_category".to_string()));
        assert_eq!(task.priority, Priority::High);
    }

    #[test]
    fn test_update_task_partial_fields() {
        let mut todo_list = TodoList::new();
        let id = todo_list.add_task_with_details(
            "Original task".to_string(),
            Some("Original description".to_string()),
            None,
            Some("original_category".to_string()),
            Priority::Medium,
        );

        let update = TaskUpdate::new()
            .title("Updated task")
            .priority(Priority::Low);

        let result = todo_list.update_task(id, update);
        assert!(result.is_ok());

        let task = todo_list.get_task(id).unwrap();
        assert_eq!(task.title, "Updated task");
        assert_eq!(task.description, Some("Original description".to_string())); // Unchanged
        assert_eq!(task.category, Some("original_category".to_string())); // Unchanged
        assert_eq!(task.priority, Priority::Low); // Updated
    }

    #[test]
    fn test_update_task_clear_optional_fields() {
        let mut todo_list = TodoList::new();
        let id = todo_list.add_task_with_details(
            "Original task".to_string(),
            Some("Original description".to_string()),
            None,
            Some("original_category".to_string()),
            Priority::Medium,
        );

        let update = TaskUpdate::new()
            .description(None)
            .category(None);

        let result = todo_list.update_task(id, update);
        assert!(result.is_ok());

        let task = todo_list.get_task(id).unwrap();
        assert_eq!(task.title, "Original task"); // Unchanged
        assert_eq!(task.description, None); // Cleared
        assert_eq!(task.category, None); // Cleared
        assert_eq!(task.priority, Priority::Medium); // Unchanged
    }

    #[test]
    fn test_update_task_not_found() {
        let mut todo_list = TodoList::new();
        let update = TaskUpdate::new().title("Updated task");

        let result = todo_list.update_task(999, update);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Task with ID 999 not found");
    }

    #[test]
    fn test_update_task_with_due_date() {
        let mut todo_list = TodoList::new();
        let id = todo_list.add_task("Task with due date".to_string());

        let due_date = Local::now() + Duration::days(3);
        let update = TaskUpdate::new().due_date(Some(due_date));

        let result = todo_list.update_task(id, update);
        assert!(result.is_ok());

        let task = todo_list.get_task(id).unwrap();
        assert_eq!(task.due_date, Some(due_date));

        // Clear the due date
        let clear_update = TaskUpdate::new().due_date(None);
        let result = todo_list.update_task(id, clear_update);
        assert!(result.is_ok());

        let task = todo_list.get_task(id).unwrap();
        assert_eq!(task.due_date, None);
    }
}