use std::collections::HashMap;
use std::env;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Duration, Local, Utc};
use clap::Args;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::commands::Command;
use crate::http;

const HACKER_NEWS_API_BASE: &str = "https://hacker-news.firebaseio.com/v0";
const OPEN_METEO_API_URL: &str = "https://api.open-meteo.com/v1/forecast";
const TODOIST_API_BASE: &str = "https://api.todoist.com/api/v1";
const DEFAULT_STASH_API_BASE: &str = "https://api.stash.kcterala.dev";
const DEFAULT_ZOHO_API_BASE: &str = "https://sprintsapi.zoho.com/zsapi";
const DEFAULT_ZOHO_ACCOUNTS_BASE: &str = "https://accounts.zoho.com";
const MAX_TODOIST_TASKS: usize = 300;

const WEATHER_LOCATIONS: [WeatherLocation; 3] = [
    WeatherLocation {
        name: "Pune",
        latitude: 18.5204,
        longitude: 73.8567,
    },
    WeatherLocation {
        name: "Hyderabad",
        latitude: 17.3850,
        longitude: 78.4867,
    },
    WeatherLocation {
        name: "Khammam",
        latitude: 17.2473,
        longitude: 80.1514,
    },
];

#[derive(Args)]
pub struct BriefCommand {
    #[arg(long, help = "Print the brief as JSON")]
    json: bool,

    #[arg(long, default_value_t = 5, help = "Number of Hacker News stories")]
    hacker_news_limit: usize,

    #[arg(
        long,
        default_value_t = 24,
        help = "Include Stash unread items published within this many hours"
    )]
    since_hours: i64,
}

impl Command for BriefCommand {
    fn execute(&self) -> Result<()> {
        if self.hacker_news_limit == 0 {
            bail!("Hacker News limit must be greater than zero");
        }
        if self.since_hours <= 0 {
            bail!("Since hours must be greater than zero");
        }

        let brief = create_morning_brief(self.hacker_news_limit, self.since_hours);

        if self.json {
            println!("{}", serde_json::to_string_pretty(&brief)?);
        } else {
            print_morning_brief(&brief);
        }

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MorningBrief {
    generated_at: DateTime<Utc>,
    stash_since_hours: i64,
    weather: BriefSection<WeatherSummary>,
    hacker_news: BriefSection<HackerNewsStory>,
    todoist: BriefSection<TodoistTask>,
    zoho_sprints: BriefSection<ZohoSprintTask>,
    stash: BriefSection<StashFeedItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BriefSection<T> {
    status: BriefSectionStatus,
    items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum BriefSectionStatus {
    Ready,
    Skipped,
    Failed,
}

impl<T> BriefSection<T> {
    fn ready(items: Vec<T>) -> Self {
        Self {
            status: BriefSectionStatus::Ready,
            items,
            message: None,
        }
    }

    fn ready_with_message(items: Vec<T>, message: impl Into<String>) -> Self {
        Self {
            status: BriefSectionStatus::Ready,
            items,
            message: Some(message.into()),
        }
    }

    fn skipped(message: impl Into<String>) -> Self {
        Self {
            status: BriefSectionStatus::Skipped,
            items: Vec::new(),
            message: Some(message.into()),
        }
    }

    fn failed(source: &str, error: anyhow::Error) -> Self {
        log::debug!("Failed to load {source}: {error:#}");

        Self {
            status: BriefSectionStatus::Failed,
            items: Vec::new(),
            message: Some(format!(
                "Could not load {source}; check its credentials, configuration, and connectivity"
            )),
        }
    }
}

fn create_morning_brief(hacker_news_limit: usize, since_hours: i64) -> MorningBrief {
    MorningBrief {
        generated_at: Utc::now(),
        stash_since_hours: since_hours,
        weather: fetch_weather()
            .map(BriefSection::ready)
            .unwrap_or_else(|error| BriefSection::failed("weather", error)),
        hacker_news: fetch_hacker_news_stories(hacker_news_limit)
            .map(BriefSection::ready)
            .unwrap_or_else(|error| BriefSection::failed("Hacker News", error)),
        todoist: create_todoist_section(),
        zoho_sprints: create_zoho_sprints_section(),
        stash: create_stash_section(since_hours),
    }
}

struct WeatherLocation {
    name: &'static str,
    latitude: f64,
    longitude: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WeatherSummary {
    location: String,
    condition: String,
    temperature_celsius: f64,
    apparent_temperature_celsius: f64,
    high_celsius: f64,
    low_celsius: f64,
    precipitation_probability_percent: u8,
}

#[derive(Deserialize)]
struct OpenMeteoForecast {
    current: OpenMeteoCurrentWeather,
    daily: OpenMeteoDailyWeather,
}

#[derive(Deserialize)]
struct OpenMeteoCurrentWeather {
    temperature_2m: f64,
    apparent_temperature: f64,
    weather_code: u8,
}

#[derive(Deserialize)]
struct OpenMeteoDailyWeather {
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    precipitation_probability_max: Vec<u8>,
}

fn fetch_weather() -> Result<Vec<WeatherSummary>> {
    let latitudes = WEATHER_LOCATIONS
        .iter()
        .map(|location| location.latitude.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let longitudes = WEATHER_LOCATIONS
        .iter()
        .map(|location| location.longitude.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let forecasts = http::get_client()
        .get(OPEN_METEO_API_URL)
        .query(&[
            ("latitude", latitudes.as_str()),
            ("longitude", longitudes.as_str()),
            (
                "current",
                "temperature_2m,apparent_temperature,weather_code",
            ),
            (
                "daily",
                "temperature_2m_max,temperature_2m_min,precipitation_probability_max",
            ),
            ("timezone", "Asia/Kolkata"),
            ("forecast_days", "1"),
        ])
        .send()
        .context("Failed to fetch weather")?
        .error_for_status()
        .context("Open-Meteo rejected the weather request")?
        .json::<Vec<OpenMeteoForecast>>()
        .context("Failed to parse weather")?;

    if forecasts.len() != WEATHER_LOCATIONS.len() {
        bail!(
            "Open-Meteo returned {} locations instead of {}",
            forecasts.len(),
            WEATHER_LOCATIONS.len()
        );
    }

    WEATHER_LOCATIONS
        .iter()
        .zip(forecasts)
        .map(|(location, forecast)| {
            Ok(WeatherSummary {
                location: location.name.to_string(),
                condition: weather_condition(forecast.current.weather_code).to_string(),
                temperature_celsius: forecast.current.temperature_2m,
                apparent_temperature_celsius: forecast.current.apparent_temperature,
                high_celsius: first_daily_value(
                    &forecast.daily.temperature_2m_max,
                    "maximum temperature",
                )?,
                low_celsius: first_daily_value(
                    &forecast.daily.temperature_2m_min,
                    "minimum temperature",
                )?,
                precipitation_probability_percent: first_daily_value(
                    &forecast.daily.precipitation_probability_max,
                    "precipitation probability",
                )?,
            })
        })
        .collect()
}

fn first_daily_value<T: Copy>(values: &[T], field_name: &str) -> Result<T> {
    values
        .first()
        .copied()
        .with_context(|| format!("Weather response has no {field_name}"))
}

fn weather_condition(code: u8) -> &'static str {
    match code {
        0 => "Clear",
        1 => "Mostly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Foggy",
        51 | 53 | 55 => "Drizzle",
        56 | 57 => "Freezing drizzle",
        61 | 63 | 65 => "Rain",
        66 | 67 => "Freezing rain",
        71 | 73 | 75 | 77 => "Snow",
        80..=82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorms",
        96 | 99 => "Thunderstorms with hail",
        _ => "Unknown",
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct HackerNewsStory {
    id: u64,
    title: String,
    score: u64,
    comments: u64,
    url: String,
}

#[derive(Deserialize)]
struct HackerNewsApiItem {
    id: u64,
    title: String,
    #[serde(default)]
    score: u64,
    #[serde(default)]
    descendants: u64,
    url: Option<String>,
}

fn fetch_hacker_news_stories(limit: usize) -> Result<Vec<HackerNewsStory>> {
    let client = http::get_client();
    let story_ids = client
        .get(format!("{HACKER_NEWS_API_BASE}/topstories.json"))
        .send()
        .context("Failed to fetch Hacker News top stories")?
        .error_for_status()
        .context("Hacker News rejected the top stories request")?
        .json::<Vec<u64>>()
        .context("Failed to parse Hacker News top stories")?;

    story_ids
        .into_iter()
        .take(limit)
        .map(|story_id| {
            let item = client
                .get(format!("{HACKER_NEWS_API_BASE}/item/{story_id}.json"))
                .send()
                .with_context(|| format!("Failed to fetch Hacker News story {story_id}"))?
                .error_for_status()
                .with_context(|| format!("Hacker News rejected story {story_id}"))?
                .json::<HackerNewsApiItem>()
                .with_context(|| format!("Failed to parse Hacker News story {story_id}"))?;

            Ok(HackerNewsStory {
                id: item.id,
                title: item.title,
                score: item.score,
                comments: item.descendants,
                url: item
                    .url
                    .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={story_id}")),
            })
        })
        .collect()
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TodoistTask {
    id: String,
    content: String,
    priority: u8,
    due: Option<TodoistDueDate>,
    deadline: Option<TodoistDeadline>,
    #[serde(default)]
    url: String,
}

#[derive(Deserialize, Serialize)]
struct TodoistDueDate {
    date: String,
    string: String,
}

#[derive(Deserialize, Serialize)]
struct TodoistDeadline {
    date: String,
}

impl TodoistTask {
    fn effective_date(&self) -> &str {
        self.due
            .as_ref()
            .map(|due| due.date.as_str())
            .or_else(|| {
                self.deadline
                    .as_ref()
                    .map(|deadline| deadline.date.as_str())
            })
            .unwrap_or("")
    }

    fn due_label(&self) -> &str {
        self.due
            .as_ref()
            .map(|due| due.string.as_str())
            .or_else(|| self.deadline.as_ref().map(|_| "deadline"))
            .unwrap_or("unscheduled")
    }
}

#[derive(Deserialize)]
struct TodoistTasksResponse {
    results: Vec<TodoistTask>,
    next_cursor: Option<String>,
}

fn create_todoist_section() -> BriefSection<TodoistTask> {
    let Some(token) = first_environment_variable(&["KIT_TODOIST_TOKEN", "TODOIST_API_TOKEN"])
    else {
        return BriefSection::skipped(
            "Set KIT_TODOIST_TOKEN or TODOIST_API_TOKEN to include Todoist tasks",
        );
    };

    fetch_todoist_tasks(&token)
        .map(BriefSection::ready)
        .unwrap_or_else(|error| BriefSection::failed("Todoist", error))
}

fn fetch_todoist_tasks(token: &str) -> Result<Vec<TodoistTask>> {
    let client = http::get_client();
    let mut tasks = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let remaining_tasks = MAX_TODOIST_TASKS - tasks.len();
        if remaining_tasks == 0 {
            break;
        }

        let page_size = remaining_tasks.min(200).to_string();
        let mut query = vec![
            ("query", "(today | overdue) & (assigned to: me | !assigned)"),
            ("limit", page_size.as_str()),
        ];
        if let Some(cursor) = cursor.as_deref() {
            query.push(("cursor", cursor));
        }

        let response = client
            .get(format!("{TODOIST_API_BASE}/tasks/filter"))
            .bearer_auth(token)
            .query(&query)
            .send()
            .context("Failed to fetch Todoist tasks")?
            .error_for_status()
            .context("Todoist rejected the tasks request")?
            .json::<TodoistTasksResponse>()
            .context("Failed to parse Todoist tasks")?;

        tasks.extend(response.results);
        cursor = response.next_cursor;
        if cursor.is_none() {
            break;
        }
    }

    tasks.sort_by(|left, right| {
        left.effective_date()
            .cmp(right.effective_date())
            .then_with(|| right.priority.cmp(&left.priority))
    });

    Ok(tasks)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ZohoSprintTask {
    id: String,
    name: String,
    sprint_id: String,
    status_id: Option<String>,
    priority_id: Option<String>,
    end_date: Option<String>,
}

struct ZohoConfiguration {
    access_token: String,
    api_base: String,
    team_id: String,
    project_id: String,
    sprint_ids: Vec<String>,
    user_id: String,
}

#[derive(Deserialize)]
struct ZohoItemsResponse {
    #[serde(default)]
    next: bool,
    #[serde(default, alias = "nextIndex")]
    next_index: Option<usize>,
    item_prop: HashMap<String, usize>,
    #[serde(rename = "itemIds")]
    item_ids: Vec<String>,
    #[serde(rename = "itemJObj")]
    item_objects: HashMap<String, Vec<Value>>,
}

fn create_zoho_sprints_section() -> BriefSection<ZohoSprintTask> {
    let configuration = match load_zoho_configuration() {
        Ok(Some(configuration)) => configuration,
        Ok(None) => {
            return BriefSection::skipped(
                "Configure Zoho authentication, KIT_ZOHO_TEAM_ID, KIT_ZOHO_PROJECT_ID, \
                 KIT_ZOHO_SPRINT_IDS, and KIT_ZOHO_USER_ID to include Zoho Sprints tasks",
            );
        }
        Err(error) => return BriefSection::failed("Zoho Sprints", error),
    };

    fetch_zoho_sprint_tasks(&configuration)
        .map(BriefSection::ready)
        .unwrap_or_else(|error| BriefSection::failed("Zoho Sprints", error))
}

fn load_zoho_configuration() -> Result<Option<ZohoConfiguration>> {
    let identifiers = [
        env::var("KIT_ZOHO_TEAM_ID").ok(),
        env::var("KIT_ZOHO_PROJECT_ID").ok(),
        env::var("KIT_ZOHO_SPRINT_IDS").ok(),
        env::var("KIT_ZOHO_USER_ID").ok(),
    ];
    let has_access_token = env::var("KIT_ZOHO_ACCESS_TOKEN").is_ok();
    let has_refresh_token = env::var("KIT_ZOHO_REFRESH_TOKEN").is_ok();

    if identifiers.iter().all(Option::is_none) && !has_access_token && !has_refresh_token {
        return Ok(None);
    }
    if identifiers.iter().any(Option::is_none) {
        bail!("Zoho Sprints configuration is incomplete; all Zoho identifier values are required");
    }

    let [team_id, project_id, sprint_ids, user_id] = identifiers.map(Option::unwrap);
    let sprint_ids = sprint_ids
        .split(',')
        .map(str::trim)
        .filter(|sprint_id| !sprint_id.is_empty())
        .map(String::from)
        .collect::<Vec<_>>();
    if sprint_ids.is_empty() {
        bail!("KIT_ZOHO_SPRINT_IDS must contain at least one sprint ID");
    }
    let access_token = load_zoho_access_token()?;

    Ok(Some(ZohoConfiguration {
        access_token,
        api_base: env::var("KIT_ZOHO_API_BASE")
            .unwrap_or_else(|_| DEFAULT_ZOHO_API_BASE.to_string()),
        team_id,
        project_id,
        sprint_ids,
        user_id,
    }))
}

#[derive(Deserialize)]
struct ZohoAccessTokenResponse {
    access_token: String,
}

fn load_zoho_access_token() -> Result<String> {
    if let Ok(refresh_token) = env::var("KIT_ZOHO_REFRESH_TOKEN") {
        let client_id = env::var("KIT_ZOHO_CLIENT_ID")
            .context("KIT_ZOHO_CLIENT_ID is required with KIT_ZOHO_REFRESH_TOKEN")?;
        let client_secret = env::var("KIT_ZOHO_CLIENT_SECRET")
            .context("KIT_ZOHO_CLIENT_SECRET is required with KIT_ZOHO_REFRESH_TOKEN")?;
        let accounts_base = env::var("KIT_ZOHO_ACCOUNTS_BASE")
            .unwrap_or_else(|_| DEFAULT_ZOHO_ACCOUNTS_BASE.to_string());

        return http::get_client()
            .post(format!(
                "{}/oauth/v2/token",
                accounts_base.trim_end_matches('/')
            ))
            .form(&[
                ("refresh_token", refresh_token.as_str()),
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .context("Failed to refresh Zoho access token")?
            .error_for_status()
            .context("Zoho rejected the token refresh request")?
            .json::<ZohoAccessTokenResponse>()
            .context("Failed to parse the Zoho token refresh response")
            .map(|response| response.access_token);
    }

    env::var("KIT_ZOHO_ACCESS_TOKEN")
        .context("Set KIT_ZOHO_ACCESS_TOKEN or KIT_ZOHO_REFRESH_TOKEN with client credentials")
}

fn fetch_zoho_sprint_tasks(configuration: &ZohoConfiguration) -> Result<Vec<ZohoSprintTask>> {
    let mut tasks = Vec::new();

    for sprint_id in &configuration.sprint_ids {
        tasks.extend(fetch_tasks_for_zoho_sprint(configuration, sprint_id)?);
    }

    tasks.sort_by(|left, right| {
        left.end_date
            .cmp(&right.end_date)
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(tasks)
}

fn fetch_tasks_for_zoho_sprint(
    configuration: &ZohoConfiguration,
    sprint_id: &str,
) -> Result<Vec<ZohoSprintTask>> {
    let client = http::get_client();
    let url = format!(
        "{}/team/{}/projects/{}/sprints/{}/item/",
        configuration.api_base.trim_end_matches('/'),
        configuration.team_id,
        configuration.project_id,
        sprint_id
    );
    let filter = serde_json::json!({
        "queryType": 1,
        "jsontmpl": "item_default",
        "I-owner": [&configuration.user_id],
        "I-statustype": ["0", "2"]
    })
    .to_string();
    let mut index = 1;
    let mut tasks = Vec::new();

    loop {
        let response = client
            .get(&url)
            .header(
                "Authorization",
                format!("Zoho-oauthtoken {}", configuration.access_token),
            )
            .query(&[
                ("action", "data".to_string()),
                ("index", index.to_string()),
                ("range", "250".to_string()),
                ("subitem", "true".to_string()),
                ("filter", filter.clone()),
            ])
            .send()
            .with_context(|| format!("Failed to fetch Zoho sprint {sprint_id}"))?
            .error_for_status()
            .with_context(|| format!("Zoho rejected the request for sprint {sprint_id}"))?
            .json::<ZohoItemsResponse>()
            .with_context(|| format!("Failed to parse Zoho sprint {sprint_id}"))?;

        tasks.extend(parse_zoho_tasks(&response, sprint_id));
        if !response.next {
            break;
        }
        let next_index = response
            .next_index
            .context("Zoho indicated another page without returning nextIndex")?;
        if next_index <= index {
            bail!("Zoho returned a non-advancing nextIndex for sprint {sprint_id}");
        }
        index = next_index;
    }

    Ok(tasks)
}

fn parse_zoho_tasks(response: &ZohoItemsResponse, sprint_id: &str) -> Vec<ZohoSprintTask> {
    response
        .item_ids
        .iter()
        .filter_map(|item_id| {
            let fields = response.item_objects.get(item_id)?;
            let name = zoho_field_as_string(fields, &response.item_prop, "itemName")?;

            Some(ZohoSprintTask {
                id: item_id.clone(),
                name,
                sprint_id: sprint_id.to_string(),
                status_id: zoho_field_as_string(fields, &response.item_prop, "statusId"),
                priority_id: zoho_field_as_string(fields, &response.item_prop, "projPriorityId"),
                end_date: zoho_field_as_string(fields, &response.item_prop, "endDate"),
            })
        })
        .collect()
}

fn zoho_field_as_string(
    fields: &[Value],
    properties: &HashMap<String, usize>,
    property_name: &str,
) -> Option<String> {
    let value = fields.get(*properties.get(property_name)?)?;

    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StashFeedItem {
    id: i64,
    feed_title: String,
    url: String,
    title: String,
    published_at: Option<DateTime<Utc>>,
}

fn create_stash_section(since_hours: i64) -> BriefSection<StashFeedItem> {
    let Some(access_token) = env::var("KIT_STASH_ACCESS_TOKEN").ok() else {
        return BriefSection::skipped(
            "Experimental: set a short-lived KIT_STASH_ACCESS_TOKEN to include unread feed items",
        );
    };
    let api_base =
        env::var("KIT_STASH_API_BASE").unwrap_or_else(|_| DEFAULT_STASH_API_BASE.to_string());

    fetch_stash_feed_items(&api_base, &access_token, since_hours)
        .map(|items| {
            BriefSection::ready_with_message(
                items,
                "Experimental: access tokens expire after 30 minutes, and Stash filters unread \
                 state after loading its 50 newest candidates",
            )
        })
        .unwrap_or_else(|error| BriefSection::failed("Stash", error))
}

fn fetch_stash_feed_items(
    api_base: &str,
    access_token: &str,
    since_hours: i64,
) -> Result<Vec<StashFeedItem>> {
    let cutoff = Utc::now() - Duration::hours(since_hours);
    let mut items = http::get_client()
        .get(format!(
            "{}/api/feeds/items",
            api_base.trim_end_matches('/')
        ))
        .bearer_auth(access_token)
        .query(&[("filter", "unread")])
        .send()
        .context("Failed to fetch unread Stash items")?
        .error_for_status()
        .context("Stash rejected the unread items request")?
        .json::<Vec<StashFeedItem>>()
        .context("Failed to parse unread Stash items")?;

    items.retain(|item| {
        item.published_at
            .map(|published_at| published_at >= cutoff)
            .unwrap_or(false)
    });
    items.sort_by(|left, right| right.published_at.cmp(&left.published_at));

    Ok(items)
}

fn first_environment_variable(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| env::var(name).ok())
}

fn print_morning_brief(brief: &MorningBrief) {
    println!(
        "{}\n{}\n",
        "MORNING BRIEF".bold(),
        brief
            .generated_at
            .with_timezone(&Local)
            .format("%A, %B %-d · %-I:%M %p")
            .to_string()
            .dimmed()
    );

    print_section("WEATHER", &brief.weather, |weather, _| {
        format!(
            "• {}: {:.1}°C (feels {:.1}°C) · {} · H {:.1}° / L {:.1}° · Rain today {}%",
            weather.location,
            weather.temperature_celsius,
            weather.apparent_temperature_celsius,
            weather.condition,
            weather.high_celsius,
            weather.low_celsius,
            weather.precipitation_probability_percent
        )
    });
    print_section("HACKER NEWS", &brief.hacker_news, |story, index| {
        format!(
            "{}. {} {}",
            index + 1,
            story.title,
            format!("({} points, {} comments)", story.score, story.comments).dimmed()
        )
    });
    print_section("TODOIST", &brief.todoist, |task, _| {
        format!(
            "• {} {}",
            task.content,
            format!("({})", task.due_label()).dimmed()
        )
    });
    print_section("ZOHO SPRINTS", &brief.zoho_sprints, |task, _| {
        let due = task
            .end_date
            .as_deref()
            .map(|date| format!(" · due {date}"))
            .unwrap_or_default();
        format!("• {}{}", task.name, due.dimmed())
    });
    print_section(
        &format!(
            "STASH (EXPERIMENTAL) · LAST {} HOURS",
            brief.stash_since_hours
        ),
        &brief.stash,
        |item, _| {
            format!(
                "• {} {}",
                item.title,
                format!("({})", item.feed_title).dimmed()
            )
        },
    );
}

fn print_section<T>(
    title: &str,
    section: &BriefSection<T>,
    format_item: impl Fn(&T, usize) -> String,
) {
    println!("{}", title.bold().cyan());

    match section.status {
        BriefSectionStatus::Ready => {
            if section.items.is_empty() {
                println!("  Nothing waiting");
            } else {
                for (index, item) in section.items.iter().enumerate() {
                    println!("  {}", format_item(item, index));
                }
            }
            if let Some(message) = &section.message {
                println!("  {} {}", "Note:".yellow(), message);
            }
        }
        BriefSectionStatus::Skipped => println!(
            "  {} {}",
            "Skipped:".yellow(),
            section.message.as_deref().unwrap_or_default()
        ),
        BriefSectionStatus::Failed => println!(
            "  {} {}",
            "Unavailable:".red(),
            section.message.as_deref().unwrap_or_default()
        ),
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zoho_property_mapped_items() {
        let response = ZohoItemsResponse {
            next: false,
            next_index: None,
            item_prop: HashMap::from([
                ("itemName".to_string(), 0),
                ("endDate".to_string(), 1),
                ("statusId".to_string(), 2),
            ]),
            item_ids: vec!["item-1".to_string()],
            item_objects: HashMap::from([(
                "item-1".to_string(),
                vec![
                    Value::String("Ship morning brief".to_string()),
                    Value::String("2026-08-10".to_string()),
                    Value::String("open".to_string()),
                ],
            )]),
        };

        let tasks = parse_zoho_tasks(&response, "sprint-1");

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Ship morning brief");
        assert_eq!(tasks[0].status_id.as_deref(), Some("open"));
    }

    #[test]
    fn serializes_skipped_sections_without_items_or_secrets() {
        let section = BriefSection::<TodoistTask>::skipped("token is not configured");

        let json = serde_json::to_value(section).unwrap();

        assert_eq!(json["status"], "skipped");
        assert_eq!(json["items"], serde_json::json!([]));
        assert_eq!(json["message"], "token is not configured");
    }

    #[test]
    fn parses_todoist_deadline_only_tasks() {
        let task = serde_json::from_value::<TodoistTask>(serde_json::json!({
            "id": "task-1",
            "content": "Submit report",
            "priority": 4,
            "due": null,
            "deadline": { "date": "2026-08-09" },
            "url": "https://app.todoist.com/task/task-1"
        }))
        .unwrap();

        assert_eq!(task.effective_date(), "2026-08-09");
        assert_eq!(task.due_label(), "deadline");
    }

    #[test]
    fn parses_zoho_camel_case_next_index() {
        let response = serde_json::from_value::<ZohoItemsResponse>(serde_json::json!({
            "next": true,
            "nextIndex": 2,
            "item_prop": {},
            "itemIds": [],
            "itemJObj": {}
        }))
        .unwrap();

        assert_eq!(response.next_index, Some(2));
    }

    #[test]
    fn describes_wmo_weather_codes() {
        assert_eq!(weather_condition(0), "Clear");
        assert_eq!(weather_condition(63), "Rain");
        assert_eq!(weather_condition(95), "Thunderstorms");
        assert_eq!(weather_condition(100), "Unknown");
    }
}
