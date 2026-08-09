use anyhow::Result;

pub mod ai_commit;
pub mod brief;
pub mod clone;
pub mod commit;
pub mod fork;
pub mod ip;
pub mod network;
pub mod setup;
pub mod update;

pub trait Command {
    fn execute(&self) -> Result<()>;
}
