use fsrs::{FSRS, FSRSItem};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::Task;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Weights {
    weights: [f32; 19],
}
impl Weights {
    pub fn fsrs(&self) -> FSRS {
        FSRS::new(Some(&self.weights)).unwrap()
    }
}
impl Default for Weights {
    fn default() -> Self {
        Self {
            weights: fsrs::DEFAULT_PARAMETERS,
        }
    }
}
impl ssr_core::task::SharedState<'_> for Weights {}
fn extract_first_long_term_reviews<'a>(
    items: impl IntoIterator<Item = &'a FSRSItem>,
) -> Vec<FSRSItem> {
    items
        .into_iter()
        .filter_map(|i| {
            let a = i
                .reviews
                .iter()
                .take_while_inclusive(|r| r.delta_t < 1)
                .copied()
                .collect_vec();
            if a.last()?.delta_t < 1 || a.len() == i.reviews.len() {
                return None;
            }
            Some(FSRSItem { reviews: a })
        })
        .collect()
}

impl ssr_core::task::SharedStateExt<'_, Task> for Weights {
    fn optimize<'b>(
        &mut self,
        tasks: impl IntoIterator<Item = &'b Task>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Task: 'b,
    {
        let mut tasks = tasks
            .into_iter()
            .filter_map(|t| t.level.history())
            .collect::<Vec<_>>();
        tasks.extend(extract_first_long_term_reviews(&tasks));
        let fsrs = FSRS::new(None)?;
        let best_params: [f32; 19] = fsrs
            .compute_parameters(tasks, None, true)?
            .try_into()
            .expect("fsrs library should return exactly '19' weights");
        self.weights = best_params;
        Ok(())
    }
}
