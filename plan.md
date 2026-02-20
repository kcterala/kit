# Plan: Implement Command Trait Pattern

## Context
The CLI dispatch in `main.rs` uses a `match` on a flat `Commands` enum, calling free functions from `commands/mod.rs`. The `commands/` directory also mixes command handlers with service helpers (`ai.rs`, `git.rs`, `github.rs`). This refactor introduces a Command trait pattern and reorganizes the project so `commands/` only has commands and `services/` groups the domain helpers.

## File structure after change

```
src/
├── main.rs
├── auth.rs             # (unchanged)
├── config.rs           # (unchanged)
├── http.rs             # (unchanged)
├── utils/mod.rs        # (unchanged)
├── services/
│   ├── mod.rs          # module declarations
│   ├── ai.rs           # (moved from commands/ai.rs)
│   ├── git.rs          # (moved from commands/git.rs)
│   └── github.rs       # (moved from commands/github.rs)
└── commands/
    ├── mod.rs           # Command trait + re-exports
    ├── clone.rs         # CloneCommand
    ├── fork.rs          # ForkCommand
    ├── commit.rs        # CommitCommand
    ├── ai_commit.rs     # AiCommitCommand
    ├── ip.rs            # IpCommand
    └── network.rs       # NetworkCommand
```

## Approach

### 1. Create `src/services/` and move helpers

- Create `src/services/mod.rs` declaring `pub mod ai; pub mod git; pub mod github;`
- Move `src/commands/ai.rs` → `src/services/ai.rs`
- Move `src/commands/git.rs` → `src/services/git.rs`
- Move `src/commands/github.rs` → `src/services/github.rs`
- Update imports in moved files from `crate::commands::github` → `crate::services::github` etc.
- Add `mod services;` to `main.rs`

### 2. Define a `Command` trait in `src/commands/mod.rs`

```rust
pub trait Command {
    fn execute(&self) -> Result<()>;
}
```

### 3. Create one file per command

Each file defines its struct with `#[derive(clap::Args)]` and implements `Command`.

- **`clone.rs`** — `CloneCommand { repo }` + `execute()` with current `clone_repository` + `resolve()` + `should_add_upstream` logic
- **`fork.rs`** — `ForkCommand { repo }` + `execute()`
- **`commit.rs`** — `CommitCommand { message }` + `execute()` with current `commit` logic
- **`ai_commit.rs`** — `AiCommitCommand { message }` + `execute()` with current `ai_commit` logic (calls into `CommitCommand` for the actual commit)
- **`ip.rs`** — `IpCommand { copy }` + `execute()` with current `ip` logic + `BASE_URL_FOR_IP` constant
- **`network.rs`** — `NetworkCommand` + `execute()` with current `network` logic

### 4. Update `src/commands/mod.rs`

- Declare `Command` trait
- Declare submodules: `mod clone; mod fork; mod commit; mod ai_commit; mod ip; mod network;`
- Re-export: `pub use clone::CloneCommand;` etc.
- Remove all free functions, `resolve()`, `BASE_URL_FOR_IP`, and old `mod ai; mod git; pub mod github;`

### 5. Update `src/main.rs`

Update `Commands` enum to tuple variants + simplify dispatch:

```rust
enum Commands {
    Clone(CloneCommand),
    Fork(ForkCommand),
    Commit(CommitCommand),
    AiCommit(AiCommitCommand),
    Ip(IpCommand),
    Network(NetworkCommand),
}

// in main():
let command: Box<dyn Command> = match cli.command {
    Commands::Clone(cmd) => Box::new(cmd),
    Commands::Fork(cmd) => Box::new(cmd),
    // ...
};
command.execute()?;
```

## Files to create
- `src/services/mod.rs`
- `src/commands/clone.rs`
- `src/commands/fork.rs`
- `src/commands/commit.rs`
- `src/commands/ai_commit.rs`
- `src/commands/ip.rs`
- `src/commands/network.rs`

## Files to modify
- `src/main.rs` — add `mod services`, update enum + dispatch
- `src/commands/mod.rs` — trait, re-exports, remove handler functions

## Files to move (git mv)
- `src/commands/ai.rs` → `src/services/ai.rs`
- `src/commands/git.rs` → `src/services/git.rs`
- `src/commands/github.rs` → `src/services/github.rs`

## Import updates needed in moved files
- `services/ai.rs`: `crate::config` and `crate::http` (already correct, no change)
- `services/github.rs`: `crate::auth` and `crate::http` (already correct, no change)
- `services/git.rs`: `crate::commands::github::GetRepoResponse` → `crate::services::github::GetRepoResponse`

## Verification
- `cargo build` to confirm compilation
- `cargo run -- ip` and `cargo run -- network` to verify commands work
