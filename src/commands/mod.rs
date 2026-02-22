use anyhow::Result;

pub mod ai_commit;
pub mod clone;
pub mod commit;
pub mod fork;
pub mod ip;

pub trait Command {
    fn execute(&self) -> Result<()>;
}
