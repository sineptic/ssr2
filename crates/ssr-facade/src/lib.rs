#![warn(clippy::pedantic)]
#![feature(extract_if, hash_extract_if, iter_collect_into)]

use std::time::{Duration, SystemTime};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ssr_core::{
    task::{SharedStateExt, Task},
    tasks_facade::{TaskId, TasksFacade},
};

fn serialize_id<S>(id: &TaskId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    id.serialize(serializer)
}
fn deserialize_id<'de, D>(deserializer: D) -> Result<TaskId, D::Error>
where
    D: Deserializer<'de>,
{
    const GEN_RANDOM: bool = false;

    let id = TaskId::deserialize(deserializer)?;
    let id = if GEN_RANDOM { rand::random() } else { id };
    Ok(id)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "T: Task<'de>"))]
struct TaskWrapper<T> {
    task: T,
    #[serde(serialize_with = "serialize_id", deserialize_with = "deserialize_id")]
    id: TaskId,
}

impl<'a, T: Task<'a>> TaskWrapper<T> {
    fn new(value: T) -> Self {
        Self {
            task: value,
            id: rand::random(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "'a: 'de, 'de: 'a"))]
pub struct Facade<'a, T>
where
    T: Task<'a>,
{
    name: String,
    tasks_pool: Vec<TaskWrapper<T>>,
    tasks_to_recall: Vec<TaskWrapper<T>>,
    desired_retention: f64,
    state: T::SharedState,
}

impl<'a, T: Task<'a>> Facade<'a, T> {
    pub fn find_tasks_to_recall(&mut self) {
        let now = SystemTime::now() + Duration::from_secs(10);
        self.tasks_pool
            .extract_if(.., |t| {
                t.task.next_repetition(&self.state, self.desired_retention) <= now
            })
            .collect_into(&mut self.tasks_to_recall);
    }
    pub fn reload_all_tasks_timings(&mut self) {
        self.tasks_to_recall
            .drain(..)
            .collect_into(&mut self.tasks_pool);
        self.find_tasks_to_recall();
    }

    fn take_random_task(&mut self) -> Option<TaskWrapper<T>> {
        if self.tasks_to_recall.is_empty() {
            return None;
        }
        let index = thread_rng().gen_range(0..self.tasks_to_recall.len());
        Some(self.tasks_to_recall.swap_remove(index))
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
                    t.task
                        .next_repetition(&self.state, self.desired_retention)
                        .duration_since(SystemTime::now())
                        .unwrap_or(Duration::default())
                })
                .min()
        }
    }
}
impl<'a, F: Task<'a>> Facade<'a, F> {
    /// # Warning
    /// You will loose all progress.
    pub fn migrate<T: Task<'a> + std::fmt::Debug>(&self) -> Facade<'a, T>
    where
        T::SharedState: std::fmt::Debug,
    {
        let task_templates = self
            .tasks_pool
            .iter()
            .chain(self.tasks_to_recall.iter())
            .map(|t| t.task.get_blocks());
        let mut new_facade = Facade::new(self.name.clone(), self.desired_retention);
        for i in task_templates {
            new_facade.create_task(i);
        }
        new_facade
    }
}
impl<'a, T: Task<'a>> TasksFacade<'a, T> for Facade<'a, T> {
    fn new(name: String, desired_retention: f64) -> Self {
        Self {
            name,
            tasks_pool: Vec::default(),
            tasks_to_recall: Vec::default(),
            desired_retention,
            state: T::SharedState::default(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn tasks_total(&self) -> usize {
        self.tasks_pool.len() + self.tasks_to_recall.len()
    }
    fn tasks_to_complete(&self) -> usize {
        self.tasks_to_recall.len()
    }

    fn complete_task(
        &mut self,
        interaction: &mut impl FnMut(
            TaskId,
            s_text_input_f::Blocks,
        ) -> std::io::Result<s_text_input_f::Response>,
    ) -> Result<(), ssr_core::tasks_facade::Error> {
        self.find_tasks_to_recall();
        let Some(TaskWrapper { mut task, id }) = self.take_random_task() else {
            return match self.until_next_repetition() {
                Some(time_until_next_repetition) => {
                    Err(ssr_core::tasks_facade::Error::NoTaskToComplete {
                        time_until_next_repetition,
                    })
                }
                None => Err(ssr_core::tasks_facade::Error::NoTask),
            };
        };
        task.complete(&mut self.state, self.desired_retention, &mut |blocks| {
            interaction(id, blocks)
        })?;
        self.tasks_pool.push(TaskWrapper { task, id });
        Ok(())
    }

    fn insert(&mut self, task: T) {
        self.tasks_pool.push(TaskWrapper::new(task));
    }

    fn iter<'t>(&'t self) -> impl Iterator<Item = (&'t T, TaskId)>
    where
        T: 't,
    {
        self.tasks_pool
            .iter()
            .chain(self.tasks_to_recall.iter())
            .map(|TaskWrapper { task, id }| (task, *id))
    }

    fn remove(&mut self, id: TaskId) -> bool {
        let mut removed = false;
        self.tasks_to_recall.retain(|task_wrapper| {
            if task_wrapper.id == id {
                removed = true;
                false
            } else {
                true
            }
        });
        if !removed {
            self.tasks_pool.retain(|task_wrapper| {
                if task_wrapper.id == id {
                    removed = true;
                    false
                } else {
                    true
                }
            });
        }
        removed
    }

    fn get_desired_retention(&self) -> f64 {
        self.desired_retention
    }

    fn set_desired_retention(&mut self, desired_retention: f64) {
        self.desired_retention = desired_retention;

        self.reload_all_tasks_timings();
    }

    fn create_task(&mut self, input: s_text_input_f::BlocksWithAnswer) {
        self.insert(T::new(input));
    }

    fn optimize(&mut self) -> Result<(), Box<dyn std::error::Error>>
    where
        T::SharedState: SharedStateExt<'a, T>,
    {
        let items = self
            .tasks_pool
            .iter()
            .chain(self.tasks_to_recall.iter())
            .map(|x| &x.task);
        self.state.optimize(items)?;

        self.reload_all_tasks_timings();
        Ok(())
    }
}
