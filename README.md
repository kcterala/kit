# kit

A lightweight developer CLI with handy utilities.

## Installation

The installer supports macOS and Linux on x86_64/amd64 and ARM64/aarch64:

```bash
curl -sSL https://raw.githubusercontent.com/kcterala/kit/main/install.sh | bash
```

Or install Rust and build from source with `cargo build --release`.

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
kit brief --email-to you@example.com
```
Shows current weather for Pune, Hyderabad, and Khammam; top Hacker News stories; and, when configured, today's Todoist tasks, assigned Zoho Sprints work items, and unread Stash feed items from the last 24 hours. Each source is independent, so one unavailable service does not hide the others. Weather data is provided by Open-Meteo.

Configure integrations with environment variables:

| Source | Variables |
| --- | --- |
| Todoist | `KIT_TODOIST_TOKEN` or `TODOIST_API_TOKEN` |
| Stash (experimental) | Short-lived `KIT_STASH_ACCESS_TOKEN`; optional `KIT_STASH_API_BASE` |
| Zoho Sprints | `KIT_ZOHO_TEAM_ID`, `KIT_ZOHO_PROJECT_ID`, `KIT_ZOHO_SPRINT_IDS` (comma-separated), and `KIT_ZOHO_USER_ID`; either `KIT_ZOHO_ACCESS_TOKEN` or durable `KIT_ZOHO_REFRESH_TOKEN`, `KIT_ZOHO_CLIENT_ID`, and `KIT_ZOHO_CLIENT_SECRET`; optional `KIT_ZOHO_API_BASE` and `KIT_ZOHO_ACCOUNTS_BASE` for your Zoho data center |

Zoho OAuth tokens require the `ZohoSprints.items.READ` scope. Stash is experimental because its access tokens expire after 30 minutes and it has no personal-access-token flow. It also loads at most 50 recent candidates before applying unread state, so the section is best-effort; `kit` applies the requested time window locally.

#### Email delivery with Resend

`--email-to` sends the brief through [Resend](https://resend.com). Set its API key in
`KIT_RESEND_API_KEY`. By default, kit sends from `Kit <onboarding@resend.dev>`, which Resend only
allows when the recipient is the email address associated with your Resend account. For other
recipients, verify a domain in Resend and set `KIT_BRIEF_EMAIL_FROM`, for example
`Kit <brief@mail.example.com>`.

1. Create a [Resend account](https://resend.com/signup) using the address that will receive the
   initial test email.
2. Create an API key from the [Resend API Keys page](https://resend.com/api-keys). Do not commit or
   paste the key into logs, issues, or chat.
3. Build and install the current version of kit on the VPS:

   ```bash
   cargo build --release
   sudo install -m 755 target/release/kit /usr/local/bin/kit
   ```

4. Store the environment in a file readable only by root:

   ```bash
   sudo install -d -m 700 /etc/kit
   sudoedit /etc/kit/brief.env
   sudo chmod 600 /etc/kit/brief.env
   ```

   At minimum, `/etc/kit/brief.env` needs the following:

   ```bash
   KIT_RESEND_API_KEY=re_replace_with_your_api_key
   # KIT_BRIEF_EMAIL_FROM='Kit <brief@mail.example.com>'
   ```

Add the Todoist and Zoho variables from the table above to the same file if those sections should
be included.

##### Test email delivery

Send a brief immediately before creating the schedule:

```bash
sudo sh -c 'set -a; . /etc/kit/brief.env; set +a; /usr/local/bin/kit brief --email-to kcterala@gmail.com'
```

A successful request prints `Morning brief emailed to kcterala@gmail.com`. Check the inbox and
spam folder, then check the [Resend email logs](https://resend.com/emails) to confirm delivery. If
Resend rejects the request, verify that the API key is active and that the Resend account uses
`kcterala@gmail.com`. The default `onboarding@resend.dev` sender cannot send to arbitrary
recipients.

After verifying a custom domain in Resend, add the sender to `/etc/kit/brief.env` and repeat the
test:

```bash
KIT_BRIEF_EMAIL_FROM='Kit <brief@mail.example.com>'
```

##### Schedule delivery for 6:00 AM IST

Add this to root's crontab with `sudo crontab -e` (adjust the `kit` path if needed):

```cron
CRON_TZ=Asia/Kolkata
0 6 * * * set -a; . /etc/kit/brief.env; set +a; /usr/local/bin/kit brief --email-to kcterala@gmail.com >> /var/log/kit-brief.log 2>&1
```

Confirm that the entry was saved and inspect its output after a scheduled run:

```bash
sudo crontab -l
sudo tail -n 50 /var/log/kit-brief.log
```
