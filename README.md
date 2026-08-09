# kit

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
kit clone git@github.com:user/repo.git
```
Clones a repository. Automatically adds upstream remote for forks you own.

### Fork
```bash
kit fork https://github.com/user/repo
```
Fork a repository (coming soon).

### Commit
```bash
kit commit "your commit message"
```
Stages all changes and commits with the given message.

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

### Setup agent conventions
```bash
kit setup agents
```
Downloads the coding conventions from `https://agents.kcterala.dev/agents.md` and overwrites the global instructions for detected Pi and Claude Code installations.
