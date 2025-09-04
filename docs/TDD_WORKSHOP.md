# TDD Workshop: Hands-On Implementation

## 🎯 Workshop Overview

This hands-on workshop guides you through implementing Test-Driven Development in a Rust project. You'll build a simple but realistic application using TDD principles, experiencing the Red-Green-Refactor cycle firsthand.

## 🏗️ Workshop Setup

### Prerequisites
- Rust installed (1.70+)
- Basic Rust knowledge
- Text editor or IDE
- Terminal/command line

### Project Setup
```bash
# Create new project
cargo new tdd-workshop
cd tdd-workshop

# Add dependencies
cargo add tokio --features full
cargo add serde --features derive
cargo add serde_json
cargo add uuid --features v4

# Add dev dependencies
cargo add --dev proptest
cargo add --dev criterion
cargo add --dev tempfile
cargo add --dev rand
```

## 🚀 Workshop Project: Task Manager

We'll build a simple task management system with the following features:
- Create, read, update, delete tasks
- Task categories and priorities
- Due dates and completion status
- Search and filtering
- Data persistence

## 📋 Workshop Structure

### Phase 1: Foundation (30 minutes)
- Project setup and basic structure
- First TDD cycle with simple task creation

### Phase 2: Core Features (45 minutes)
- Task CRUD operations
- Validation and error handling
- Property-based testing

### Phase 3: Advanced Features (45 minutes)
- Search and filtering
- Performance optimization
- Integration testing

### Phase 4: Polish and Reflection (30 minutes)
- Code review and refactoring
- Lessons learned discussion

## 🧪 Phase 1: Foundation

### Step 1: Project Structure

Create the following directory structure:
```
tdd-workshop/
├── src/
│   ├── lib.rs
│   ├── task.rs
│   ├── task_manager.rs
│   └── error.rs
├── tests/
│   ├── unit/
│   ├── integration/
│   └── property/
└── Cargo.toml
```

### Step 2: First TDD Cycle

#### Red: Write Failing Test

Create `tests/unit/task_tests.rs`:

```rust
use tdd_workshop::task::{Task, TaskStatus, Priority};

#[test]
fn test_task_creation() {
    // This test will fail because Task doesn't exist yet
    let task = Task::new(
        "Learn TDD",
        "Study test-driven development principles",
        Priority::High,
    );
    
    assert_eq!(task.title(), "Learn TDD");
    assert_eq!(task.description(), "Study test-driven development principles");
    assert_eq!(task.priority(), Priority::High);
    assert_eq!(task.status(), TaskStatus::Pending);
}
```

#### Green: Minimal Implementation

Create `src/task.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    title: String,
    description: String,
    priority: Priority,
    status: TaskStatus,
    created_at: SystemTime,
    updated_at: SystemTime,
}

impl Task {
    pub fn new(title: &str, description: &str, priority: Priority) -> Self {
        let now = SystemTime::now();
        Self {
            title: title.to_string(),
            description: description.to_string(),
            priority,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn title(&self) -> &str {
        &self.title
    }
    
    pub fn description(&self) -> &str {
        &self.description
    }
    
    pub fn priority(&self) -> &Priority {
        &self.priority
    }
    
    pub fn status(&self) -> &TaskStatus {
        &self.status
    }
}
```

#### Refactor: Improve Implementation

Add more functionality while keeping tests passing:

```rust
impl Task {
    // ... existing methods ...
    
    pub fn update_title(&mut self, new_title: &str) {
        self.title = new_title.to_string();
        self.updated_at = SystemTime::now();
    }
    
    pub fn update_description(&mut self, new_description: &str) {
        self.description = new_description.to_string();
        self.updated_at = SystemTime::now();
    }
    
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = SystemTime::now();
    }
    
    pub fn is_completed(&self) -> bool {
        matches!(self.status, TaskStatus::Completed)
    }
    
    pub fn is_overdue(&self) -> bool {
        // Add due date logic later
        false
    }
}
```

### Step 3: Run Tests

```bash
cargo test
```

You should see all tests passing! 🎉

## 🔧 Phase 2: Core Features

### Step 1: Task Manager Implementation

#### Red: Write Failing Test

Create `tests/unit/task_manager_tests.rs`:

```rust
use tdd_workshop::task_manager::TaskManager;
use tdd_workshop::task::{Task, Priority};

#[test]
fn test_task_manager_creation() {
    let manager = TaskManager::new();
    assert_eq!(manager.task_count(), 0);
    assert!(manager.is_empty());
}

#[test]
fn test_add_task() {
    let mut manager = TaskManager::new();
    let task = Task::new("Test Task", "Test Description", Priority::Medium);
    
    let task_id = manager.add_task(task).unwrap();
    assert_eq!(manager.task_count(), 1);
    assert!(!manager.is_empty());
    
    let retrieved_task = manager.get_task(task_id).unwrap();
    assert_eq!(retrieved_task.title(), "Test Task");
}

#[test]
fn test_remove_task() {
    let mut manager = TaskManager::new();
    let task = Task::new("Test Task", "Test Description", Priority::Medium);
    
    let task_id = manager.add_task(task).unwrap();
    assert_eq!(manager.task_count(), 1);
    
    let removed_task = manager.remove_task(task_id).unwrap();
    assert_eq!(removed_task.title(), "Test Task");
    assert_eq!(manager.task_count(), 0);
    assert!(manager.is_empty());
}
```

#### Green: Implement TaskManager

Create `src/task_manager.rs`:

```rust
use crate::task::Task;
use std::collections::HashMap;
use uuid::Uuid;

pub type TaskId = Uuid;

#[derive(Debug)]
pub struct TaskManager {
    tasks: HashMap<TaskId, Task>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }
    
    pub fn add_task(&mut self, task: Task) -> Result<TaskId, TaskManagerError> {
        let task_id = Uuid::new_v4();
        self.tasks.insert(task_id, task);
        Ok(task_id)
    }
    
    pub fn get_task(&self, task_id: TaskId) -> Option<&Task> {
        self.tasks.get(&task_id)
    }
    
    pub fn remove_task(&mut self, task_id: TaskId) -> Option<Task> {
        self.tasks.remove(&task_id)
    }
    
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaskManagerError {
    #[error("Task not found")]
    TaskNotFound,
    #[error("Invalid task data")]
    InvalidTaskData,
}
```

### Step 2: Property-Based Testing

#### Red: Write Property Tests

Create `tests/property/task_properties.rs`:

```rust
use proptest::prelude::*;
use tdd_workshop::task::{Task, Priority};
use tdd_workshop::task_manager::TaskManager;

proptest! {
    #[test]
    fn test_task_manager_invariants(
        tasks in prop::collection::vec(
            (any::<String>(), any::<String>(), any::<Priority>()),
            0..100
        )
    ) {
        let mut manager = TaskManager::new();
        let mut task_ids = Vec::new();
        
        // Add tasks
        for (title, description, priority) in tasks {
            let task = Task::new(&title, &description, priority);
            let task_id = manager.add_task(task).unwrap();
            task_ids.push(task_id);
        }
        
        // Property: Task count should match added tasks
        prop_assert_eq!(manager.task_count(), tasks.len());
        
        // Property: All added tasks should be retrievable
        for task_id in task_ids {
            prop_assert!(manager.get_task(task_id).is_some());
        }
        
        // Property: Removing all tasks should empty the manager
        for task_id in task_ids {
            manager.remove_task(task_id);
        }
        prop_assert!(manager.is_empty());
    }
    
    #[test]
    fn test_task_immutability(
        title in any::<String>(),
        description in any::<String>(),
        priority in any::<Priority>()
    ) {
        let task = Task::new(&title, &description, priority);
        
        // Property: Task creation should not modify input strings
        prop_assert_eq!(task.title(), title);
        prop_assert_eq!(task.description(), description);
        prop_assert_eq!(task.priority(), &priority);
    }
}
```

#### Green: Ensure Tests Pass

Run the property tests:

```bash
cargo test --test property_tests
```

### Step 3: Error Handling

#### Red: Write Error Test

```rust
#[test]
fn test_get_nonexistent_task() {
    let manager = TaskManager::new();
    let random_id = Uuid::new_v4();
    
    let result = manager.get_task(random_id);
    assert!(result.is_none());
}

#[test]
fn test_remove_nonexistent_task() {
    let mut manager = TaskManager::new();
    let random_id = Uuid::new_v4();
    
    let result = manager.remove_task(random_id);
    assert!(result.is_none());
}
```

## 🚀 Phase 3: Advanced Features

### Step 1: Search and Filtering

#### Red: Write Search Test

```rust
#[test]
fn test_search_tasks_by_title() {
    let mut manager = TaskManager::new();
    
    let task1 = Task::new("Learn Rust", "Study Rust programming", Priority::High);
    let task2 = Task::new("Learn TDD", "Study test-driven development", Priority::Medium);
    let task3 = Task::new("Write Code", "Practice coding", Priority::Low);
    
    manager.add_task(task1).unwrap();
    manager.add_task(task2).unwrap();
    manager.add_task(task3).unwrap();
    
    let rust_tasks = manager.search_by_title("Rust");
    assert_eq!(rust_tasks.len(), 1);
    assert_eq!(rust_tasks[0].title(), "Learn Rust");
    
    let learn_tasks = manager.search_by_title("Learn");
    assert_eq!(learn_tasks.len(), 2);
}
```

#### Green: Implement Search

```rust
impl TaskManager {
    // ... existing methods ...
    
    pub fn search_by_title(&self, query: &str) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.title().to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }
    
    pub fn filter_by_priority(&self, priority: Priority) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.priority() == &priority)
            .collect()
    }
    
    pub fn filter_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.status() == &status)
            .collect()
    }
}
```

### Step 2: Performance Testing

#### Red: Write Performance Test

Create `benches/task_manager_benchmarks.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tdd_workshop::task_manager::TaskManager;
use tdd_workshop::task::{Task, Priority};

fn benchmark_add_tasks(c: &mut Criterion) {
    c.bench_function("add_100_tasks", |b| {
        b.iter(|| {
            let mut manager = TaskManager::new();
            for i in 0..100 {
                let task = Task::new(
                    &format!("Task {}", i),
                    &format!("Description {}", i),
                    Priority::Medium,
                );
                black_box(manager.add_task(task));
            }
        })
    });
}

fn benchmark_search_tasks(c: &mut Criterion) {
    let mut manager = TaskManager::new();
    
    // Setup: Add 1000 tasks
    for i in 0..1000 {
        let task = Task::new(
            &format!("Task {}", i),
            &format!("Description {}", i),
            Priority::Medium,
        );
        manager.add_task(task).unwrap();
    }
    
    c.bench_function("search_tasks", |b| {
        b.iter(|| {
            black_box(manager.search_by_title("Task"));
        })
    });
}

criterion_group!(benches, benchmark_add_tasks, benchmark_search_tasks);
criterion_main!(benches);
```

#### Green: Run Benchmarks

```bash
cargo bench
```

### Step 3: Integration Testing

#### Red: Write Integration Test

Create `tests/integration/task_workflow_tests.rs`:

```rust
use tdd_workshop::{Task, TaskManager, Priority, TaskStatus};

#[test]
fn test_complete_task_workflow() {
    let mut manager = TaskManager::new();
    
    // 1. Create task
    let task = Task::new("Complete Project", "Finish the TDD workshop", Priority::High);
    let task_id = manager.add_task(task).unwrap();
    
    // 2. Verify task creation
    let retrieved_task = manager.get_task(task_id).unwrap();
    assert_eq!(retrieved_task.title(), "Complete Project");
    assert_eq!(retrieved_task.status(), &TaskStatus::Pending);
    
    // 3. Update task status
    let mut task_to_update = retrieved_task.clone();
    task_to_update.set_status(TaskStatus::InProgress);
    manager.update_task(task_id, task_to_update).unwrap();
    
    // 4. Verify status update
    let updated_task = manager.get_task(task_id).unwrap();
    assert_eq!(updated_task.status(), &TaskStatus::InProgress);
    
    // 5. Complete task
    let mut task_to_complete = updated_task.clone();
    task_to_complete.set_status(TaskStatus::Completed);
    manager.update_task(task_id, task_to_complete).unwrap();
    
    // 6. Verify completion
    let completed_task = manager.get_task(task_id).unwrap();
    assert!(completed_task.is_completed());
}
```

#### Green: Implement Missing Methods

Add to `TaskManager`:

```rust
impl TaskManager {
    // ... existing methods ...
    
    pub fn update_task(&mut self, task_id: TaskId, updated_task: Task) -> Result<(), TaskManagerError> {
        if self.tasks.contains_key(&task_id) {
            self.tasks.insert(task_id, updated_task);
            Ok(())
        } else {
            Err(TaskManagerError::TaskNotFound)
        }
    }
}
```

## 🎨 Phase 4: Polish and Reflection

### Step 1: Code Review

Review your implementation for:

1. **Code Quality**: Is the code clean and readable?
2. **Error Handling**: Are all error cases handled properly?
3. **Performance**: Are there any obvious performance issues?
4. **Test Coverage**: Are all code paths tested?

### Step 2: Refactoring

Identify areas for improvement:

```rust
// Before: Simple implementation
impl TaskManager {
    pub fn search_by_title(&self, query: &str) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.title().to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }
}

// After: More efficient implementation
impl TaskManager {
    pub fn search_by_title(&self, query: &str) -> Vec<&Task> {
        let query_lower = query.to_lowercase();
        self.tasks
            .values()
            .filter(|task| task.title().to_lowercase().contains(&query_lower))
            .collect()
    }
}
```

### Step 3: Lessons Learned Discussion

Discuss with your team:

1. **What went well?**
   - TDD cycle effectiveness
   - Test coverage
   - Code quality

2. **What was challenging?**
   - Writing tests first
   - Minimal implementation
   - Refactoring decisions

3. **What would you do differently?**
   - Test organization
   - Implementation approach
   - Tooling choices

## 📊 Workshop Metrics

### Success Criteria
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] Benchmarks run successfully
- [ ] Property tests validate invariants

### Quality Metrics
- **Test Coverage**: Aim for >90%
- **Performance**: No significant regressions
- **Code Quality**: Clean, readable code
- **Documentation**: Clear API documentation

## 🔧 Advanced Challenges

### Challenge 1: Persistence Layer
Implement file-based persistence for tasks:

```rust
pub trait TaskStorage {
    async fn save_tasks(&self, tasks: &HashMap<TaskId, Task>) -> Result<(), StorageError>;
    async fn load_tasks(&self) -> Result<HashMap<TaskId, Task>, StorageError>;
}
```

### Challenge 2: Concurrent Access
Add thread-safe operations:

```rust
use std::sync::RwLock;

pub struct ConcurrentTaskManager {
    tasks: RwLock<HashMap<TaskId, Task>>,
}
```

### Challenge 3: Event System
Implement task lifecycle events:

```rust
pub enum TaskEvent {
    TaskCreated { task_id: TaskId, task: Task },
    TaskUpdated { task_id: TaskId, old_task: Task, new_task: Task },
    TaskDeleted { task_id: TaskId, task: Task },
}
```

## 📚 Additional Resources

### Rust Testing
- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion.rs](https://bheisler.github.io/criterion.rs/)
- [Proptest](https://altsysrq.github.io/proptest-book/)

### TDD Resources
- "Test-Driven Development: By Example" by Kent Beck
- "Growing Object-Oriented Software, Guided by Tests" by Steve Freeman and Nat Pryce

### Workshop Extensions
- Add a web API using `axum` or `warp`
- Implement a CLI interface using `clap`
- Add a simple web UI using `leptos` or `yew`

---

**This workshop provides hands-on experience with TDD in Rust. The key is to practice the Red-Green-Refactor cycle and build confidence in writing tests first. Remember: the goal is not just working code, but well-tested, maintainable code.**
