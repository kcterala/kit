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

### Morning brief
```bash
kit brief
kit brief --json
```
Shows current weather for Pune, Hyderabad, and Khammam; top Hacker News stories; and, when configured, today's Todoist tasks, assigned Zoho Sprints work items, and unread Stash feed items from the last 24 hours. Each source is independent, so one unavailable service does not hide the others. Weather data is provided by Open-Meteo.

Configure integrations with environment variables:

| Source | Variables |
| --- | --- |
| Todoist | `KIT_TODOIST_TOKEN` or `TODOIST_API_TOKEN` |
| Stash (experimental) | Short-lived `KIT_STASH_ACCESS_TOKEN`; optional `KIT_STASH_API_BASE` |
| Zoho Sprints | `KIT_ZOHO_TEAM_ID`, `KIT_ZOHO_PROJECT_ID`, `KIT_ZOHO_SPRINT_IDS` (comma-separated), and `KIT_ZOHO_USER_ID`; either `KIT_ZOHO_ACCESS_TOKEN` or durable `KIT_ZOHO_REFRESH_TOKEN`, `KIT_ZOHO_CLIENT_ID`, and `KIT_ZOHO_CLIENT_SECRET`; optional `KIT_ZOHO_API_BASE` and `KIT_ZOHO_ACCOUNTS_BASE` for your Zoho data center |

Zoho OAuth tokens require the `ZohoSprints.items.READ` scope. Stash is experimental because its access tokens expire after 30 minutes and it has no personal-access-token flow. It also loads at most 50 recent candidates before applying unread state, so the section is best-effort; `kit` applies the requested time window locally.
