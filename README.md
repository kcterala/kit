# Kit - GitHub CLI Tool

A command-line tool for GitHub operations with automatic authentication.

## Features

- GitHub device flow authentication (login once, use everywhere)
- Clone repositories with automatic upstream remote setup for forks
- Colored logs for better visibility
- Fast and lightweight

## Installation

### Quick Install (macOS only)

```bash
curl -sSL https://raw.githubusercontent.com/kcterala/kit/main/install.sh | bash
```

This will download the pre-built binary for your system (Intel or Apple Silicon).

### Manual Download

Download the latest binary for your platform from the [releases page](https://github.com/kcterala/kit/releases):

- macOS (Apple Silicon): `kit-macos-arm64`
- macOS (Intel): `kit-macos-amd64`

Then:
```bash
chmod +x kit-macos-*
sudo mv kit-macos-* /usr/local/bin/kit
```

### Build from Source

If you have Rust installed:
```bash
git clone https://github.com/kcterala/kit.git
cd kit
cargo build --release
sudo cp target/release/kit /usr/local/bin/
```

## Prerequisites

- macOS (Intel or Apple Silicon)
- GitHub account

## Usage

### Clone a repository

```bash
kit clone https://github.com/user/repo
# or
kit clone git@github.com:user/repo.git
```

If the repository is a fork, it will automatically add the parent repository as an upstream remote.

### Fork a repository (coming soon)

```bash
kit fork https://github.com/user/repo
```

## Using as Git Alias

To override `git clone` with `kit clone`:

```bash
git config --global alias.clone '!kit clone'
```

Now you can use:
```bash
git clone https://github.com/user/repo
```

And it will use `kit` under the hood!