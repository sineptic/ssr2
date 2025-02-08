#![warn(clippy::pedantic)]

pub mod task;
pub mod tasks_facade;

pub type BlocksDatabaseId = u64;
pub trait TaskDatabase {
    fn get_blocks(
        &self,
        id: BlocksDatabaseId,
    ) -> Option<(
        s_text_input_f::Blocks,
        s_text_input_f::Response,
        Vec<s_text_input_f::Response>,
    )>;
}
