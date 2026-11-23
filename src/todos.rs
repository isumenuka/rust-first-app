//! To-do list domain logic, free of any HTTP or async concerns.

use serde::Serialize;

/// Longest title accepted, counted in characters (not bytes).
pub const MAX_TITLE_CHARS: usize = 200;

/// A single to-do item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Todo {
    /// Unique identifier. Ids are never reused, even after a delete.
    pub id: u64,
    /// What needs doing.
    pub title: String,
    /// Whether the item has been completed.
    pub done: bool,
}

/// Everything that can go wrong when changing a [`TodoList`].
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TodoError {
    /// The title was empty or only whitespace.
    #[error("title must not be empty")]
    EmptyTitle,
    /// The title was longer than [`MAX_TITLE_CHARS`].
    #[error("title must be at most {max} characters", max = MAX_TITLE_CHARS)]
    TitleTooLong,
    /// No to-do has the given id.
    #[error("no to-do with id {0}")]
    NotFound(u64),
}

/// An in-memory, ordered collection of to-dos.
///
/// # Examples
///
/// ```
/// use rust_first_app::todos::TodoList;
///
/// let mut list = TodoList::default();
/// let todo = list.add("water the plants")?;
/// list.toggle(todo.id)?;
/// assert!(list.all()[0].done);
/// # Ok::<(), rust_first_app::todos::TodoError>(())
/// ```
#[derive(Debug, Default)]
pub struct TodoList {
    items: Vec<Todo>,
    last_id: u64,
}

impl TodoList {
    /// Returns every to-do, oldest first.
    #[must_use]
    pub fn all(&self) -> &[Todo] {
        &self.items
    }

    /// Adds a new, not-yet-done to-do and returns a copy of it.
    ///
    /// Whitespace around `title` is trimmed.
    ///
    /// # Errors
    ///
    /// - [`TodoError::EmptyTitle`] if `title` is blank.
    /// - [`TodoError::TitleTooLong`] if `title` exceeds [`MAX_TITLE_CHARS`].
    pub fn add(&mut self, title: &str) -> Result<Todo, TodoError> {
        let title = validate_title(title)?;
        self.last_id += 1;
        let todo = Todo {
            id: self.last_id,
            title: title.to_owned(),
            done: false,
        };
        // The list keeps one copy; the caller gets another to send back.
        self.items.push(todo.clone());
        Ok(todo)
    }

    /// Flips the `done` state of the to-do with `id` and returns its new state.
    ///
    /// # Errors
    ///
    /// [`TodoError::NotFound`] if no to-do has that `id`.
    pub fn toggle(&mut self, id: u64) -> Result<Todo, TodoError> {
        let todo = self
            .items
            .iter_mut()
            .find(|todo| todo.id == id)
            .ok_or(TodoError::NotFound(id))?;
        todo.done = !todo.done;
        Ok(todo.clone())
    }

    /// Removes the to-do with `id` and returns it.
    ///
    /// # Errors
    ///
    /// [`TodoError::NotFound`] if no to-do has that `id`.
    pub fn remove(&mut self, id: u64) -> Result<Todo, TodoError> {
        let index = self
            .items
            .iter()
            .position(|todo| todo.id == id)
            .ok_or(TodoError::NotFound(id))?;
        Ok(self.items.remove(index))
    }
}

fn validate_title(title: &str) -> Result<&str, TodoError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(TodoError::EmptyTitle);
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(TodoError::TitleTooLong);
    }
    Ok(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod add {
        use super::*;

        #[test]
        fn trims_whitespace_from_title() {
            let mut list = TodoList::default();

            let todo = list.add("  learn Rust  ").unwrap();

            assert_eq!(todo.title, "learn Rust");
        }

        #[test]
        fn creates_todo_that_is_not_done() {
            let mut list = TodoList::default();

            let todo = list.add("learn Rust").unwrap();

            assert!(!todo.done);
        }

        #[test]
        fn gives_each_todo_a_new_id() {
            let mut list = TodoList::default();
            let first = list.add("first").unwrap();

            let second = list.add("second").unwrap();

            assert_ne!(first.id, second.id);
        }

        #[test]
        fn does_not_reuse_id_of_removed_todo() {
            let mut list = TodoList::default();
            let removed = list.add("first").unwrap();
            list.remove(removed.id).unwrap();

            let next = list.add("second").unwrap();

            assert_ne!(next.id, removed.id);
        }

        #[test]
        fn rejects_blank_title() {
            let mut list = TodoList::default();

            let err = list.add("   ").unwrap_err();

            assert_eq!(err, TodoError::EmptyTitle);
        }

        #[test]
        fn rejects_title_longer_than_limit() {
            let mut list = TodoList::default();
            let title = "a".repeat(MAX_TITLE_CHARS + 1);

            let err = list.add(&title).unwrap_err();

            assert_eq!(err, TodoError::TitleTooLong);
        }

        #[test]
        fn accepts_title_exactly_at_limit() {
            let mut list = TodoList::default();
            let title = "a".repeat(MAX_TITLE_CHARS);

            let result = list.add(&title);

            assert!(result.is_ok(), "expected Ok, got {result:?}");
        }

        #[test]
        fn counts_characters_not_bytes() {
            let mut list = TodoList::default();
            // Each "é" is 2 bytes, so this is over the limit in bytes but not in characters.
            let title = "é".repeat(MAX_TITLE_CHARS);

            let result = list.add(&title);

            assert!(result.is_ok(), "expected Ok, got {result:?}");
        }
    }

    mod toggle {
        use super::*;

        #[test]
        fn marks_open_todo_as_done() {
            let mut list = TodoList::default();
            let todo = list.add("learn Rust").unwrap();

            let toggled = list.toggle(todo.id).unwrap();

            assert!(toggled.done);
        }

        #[test]
        fn marks_done_todo_as_open_again() {
            let mut list = TodoList::default();
            let todo = list.add("learn Rust").unwrap();
            list.toggle(todo.id).unwrap();

            let toggled = list.toggle(todo.id).unwrap();

            assert!(!toggled.done);
        }

        #[test]
        fn returns_not_found_for_unknown_id() {
            let mut list = TodoList::default();

            let err = list.toggle(42).unwrap_err();

            assert_eq!(err, TodoError::NotFound(42));
        }
    }

    mod remove {
        use super::*;

        #[test]
        fn takes_todo_out_of_the_list() {
            let mut list = TodoList::default();
            let todo = list.add("learn Rust").unwrap();

            list.remove(todo.id).unwrap();

            assert_eq!(list.all(), []);
        }

        #[test]
        fn returns_not_found_for_unknown_id() {
            let mut list = TodoList::default();

            let err = list.remove(42).unwrap_err();

            assert_eq!(err, TodoError::NotFound(42));
        }
    }

    mod errors {
        use super::*;

        #[test]
        fn title_too_long_message_names_the_limit() {
            let message = TodoError::TitleTooLong.to_string();

            assert_eq!(message, "title must be at most 200 characters");
        }
    }
}
