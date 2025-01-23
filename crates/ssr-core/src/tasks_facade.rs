use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::task::{SharedStateExt, Task};

pub type TaskId = u128;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no task to complete, time until next repetition: {}s", time_until_next_repetition.as_secs())]
    NoTaskToComplete {
        time_until_next_repetition: Duration,
    },
    #[error("tasks facade is empty")]
    NoTask,
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub trait TasksFacade<'a, T: Task<'a>>: Serialize + Deserialize<'a> {
    fn new(name: String, desired_retention: f64) -> Self;
    fn get_name(&self) -> &str;
    fn get_desired_retention(&self) -> f64;
    fn set_desired_retention(&mut self, desired_retention: f64);
    fn tasks_total(&self) -> usize;
    fn tasks_to_complete(&self) -> usize;

    /// If an error occurs, the tasks facade will remain unmodified.
    /// # Errors
    /// If interaction return error.
    fn complete_task(
        &mut self,
        interaction: &mut impl FnMut(
            TaskId,
            s_text_input_f::Blocks,
        ) -> std::io::Result<s_text_input_f::Response>,
    ) -> Result<(), Error>;
    fn insert(&mut self, task: T);
    fn create_task(&mut self, input: s_text_input_f::BlocksWithAnswer);

    /// Return itrator of (&task, id)
    fn iter<'t>(&'t self) -> impl Iterator<Item = (&'t T, TaskId)>
    where
        T: 't;
    /// Remove task.
    /// Returns whether such an element was present.
    fn remove(&mut self, id: TaskId) -> bool;

    /// # Errors
    /// If error occurs when optimizing.
    /// Guarantee to not modify anything.
    fn optimize(&mut self) -> Result<(), Box<dyn std::error::Error>>
    where
        T::SharedState: SharedStateExt<'a, T>;
}
