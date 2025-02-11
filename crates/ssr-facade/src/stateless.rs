use std::time::{Duration, SystemTime};

use rand::{Rng as _, thread_rng};
use ssr_core::{BlocksDatabaseId, task::StatelessTask};

pub struct Facade<T, U>
where
    T: StatelessTask,
{
    pub user_id: U,
    tasks_pool: Vec<T>,
    tasks_to_recall: Vec<T>,
    pub desired_retention: f64,
    state: T::SharedState,
}

impl<T, U> Facade<T, U>
where
    T: StatelessTask,
{
    pub fn find_tasks_to_recall(&mut self) {
        let now = SystemTime::now() + Duration::from_secs(10);
        self.tasks_pool
            .extract_if(.., |t| {
                t.next_repetition(&self.state, self.desired_retention) <= now
            })
            .collect_into(&mut self.tasks_to_recall);
    }
    pub fn reload_all_tasks_timings(&mut self) {
        self.tasks_to_recall
            .drain(..)
            .collect_into(&mut self.tasks_pool);
        self.find_tasks_to_recall();
    }

    fn take_random_task(&mut self) -> Option<T> {
        if self.tasks_to_recall.is_empty() {
            return None;
        }
        let index = thread_rng().gen_range(0..self.tasks_to_recall.len());
        Some(self.tasks_to_recall.swap_remove(index))
    }

    fn tasks_total(&self) -> usize {
        self.tasks_pool.len() + self.tasks_to_recall.len()
    }
    pub fn tasks_to_complete(&self) -> usize {
        self.tasks_to_recall.len()
    }

    pub fn until_next_repetition(&self) -> Option<Duration> {
        if self.tasks_total() == 0 {
            None
        } else if self.tasks_to_complete() > 0 {
            Some(Duration::default())
        } else {
            self.tasks_pool
                .iter()
                .map(|t| {
                    t.next_repetition(&self.state, self.desired_retention)
                        .duration_since(SystemTime::now())
                        .unwrap_or_default()
                })
                .min()
        }
    }

    pub fn new(user_id: U, desired_retention: f64, tasks: &[BlocksDatabaseId]) -> Self {
        Self {
            user_id,
            tasks_pool: vec![],
            tasks_to_recall: tasks.iter().map(|x| T::new(*x)).collect(),
            desired_retention,
            state: Default::default(),
        }
    }

    /// If an error occurs, the tasks facade will remain unmodified.
    /// # Errors
    /// If interaction return error.
    pub fn complete_task(
        &mut self,
        check: impl FnOnce(BlocksDatabaseId) -> bool,
        interaction: &mut impl FnMut(
            s_text_input_f::Blocks,
        ) -> std::io::Result<s_text_input_f::Response>,
    ) -> Result<(), ssr_core::tasks_facade::Error> {
        self.find_tasks_to_recall();
        let Some(mut task) = self.take_random_task() else {
            return match self.until_next_repetition() {
                Some(time_until_next_repetition) => {
                    Err(ssr_core::tasks_facade::Error::NoTaskToComplete {
                        time_until_next_repetition,
                    })
                }
                None => Err(ssr_core::tasks_facade::Error::NoTask),
            };
        };
        let err = task.complete(
            check(task.get_id()),
            &mut self.state,
            self.desired_retention,
            interaction,
        );
        if let Err(err) = err {
            self.tasks_pool.push(task);
            return Err(err.into());
        }

        self.tasks_pool.push(task);

        Ok(())
    }
}
