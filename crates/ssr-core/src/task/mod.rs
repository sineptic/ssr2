use std::{error::Error, time::SystemTime};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{BlocksDatabaseId, TaskDatabase};

pub mod level;

pub trait Task<'a>: Serialize + Deserialize<'a> {
    type SharedState: SharedState<'a>;

    /// blocks must contain interactive elements
    fn new(input: s_text_input_f::BlocksWithAnswer) -> Self;
    fn get_blocks(&self) -> s_text_input_f::BlocksWithAnswer;

    fn next_repetition(
        &self,
        shared_state: &Self::SharedState,
        desired_retention: f64,
    ) -> SystemTime;
    /// If an error occurs, the task will remain unmodified.
    /// # Errors
    /// If interaction return error.
    fn complete(
        &mut self,
        shared_state: &mut Self::SharedState,
        desired_retention: f64,
        interaction: &mut impl FnMut(
            s_text_input_f::Blocks,
        ) -> std::io::Result<s_text_input_f::Response>,
    ) -> std::io::Result<()>;
}

pub trait StatelessTask: Serialize + DeserializeOwned {
    type SharedState: SharedState<'static>;

    fn new(id: BlocksDatabaseId) -> Self;
    fn next_repetition(
        &self,
        shared_state: &Self::SharedState,
        desired_retention: f64,
    ) -> SystemTime;
    /// If an error occurs, the task will remain unmodified.
    /// # Errors
    /// If interaction return error.
    fn complete(
        &mut self,
        shared_state: &mut Self::SharedState,
        db: &impl TaskDatabase,
        desired_retention: f64,
        interaction: &mut impl FnMut(
            s_text_input_f::Blocks,
        ) -> std::io::Result<s_text_input_f::Response>,
    ) -> std::io::Result<()>;
}

pub trait SharedState<'a>: Default + Serialize + Deserialize<'a> {}
impl SharedState<'_> for () {}

pub trait SharedStateExt<'a, T: Task<'a>>: SharedState<'a> {
    /// # Errors
    /// Guarantee to not modify anything.
    fn optimize<'b>(
        &mut self,
        tasks: impl IntoIterator<Item = &'b T>,
    ) -> Result<(), Box<dyn Error>>
    where
        T: 'b;
}
