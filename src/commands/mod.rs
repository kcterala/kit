use anyhow::Result;

mod ai;
pub mod ai_commit;
pub mod clone;
pub mod commit;
pub mod fork;
mod git;
pub mod github;
pub mod ip;

pub trait Command {
    fn execute(&self) -> Result<()>;
}
