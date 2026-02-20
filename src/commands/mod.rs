use anyhow::Result;

mod ai_commit;
mod clone;
mod commit;
mod fork;
mod ip;
mod network;

pub use ai_commit::AiCommitCommand;
pub use clone::CloneCommand;
pub use commit::CommitCommand;
pub use fork::ForkCommand;
pub use ip::IpCommand;
pub use network::NetworkCommand;

pub trait Command {
    fn execute(&self) -> Result<()>;
}
