# Kit

A lightweight developer CLI with handy utilities.

## Installation

```bash
curl -sSL https://raw.githubusercontent.com/kcterala/kit/main/install.sh | bash
```

Or build from source with `cargo build --release`.

## Commands

### Clone
```bash
kit clone https://github.com/user/repo
```
Clones a repository. Automatically adds upstream remote for forks.

### AI Commit
```bash
kit ai-commit "your commit message"
```
Polishes your commit message using AI and offers multiple options to choose from.

### IP
```bash
kit ip        # Display your public IP
kit ip -c     # Copy IP to clipboard
```

## Git Alias

Override `git clone` with kit:
```bash
git config --global alias.clone '!kit clone'
```
