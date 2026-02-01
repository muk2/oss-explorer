use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// GitHub API response structures
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub open_issues_count: u32,
    pub created_at: String,
    pub updated_at: String,
    pub owner: Owner,
    #[serde(default)]
    pub fork: bool,
    #[serde(default)]
    pub archived: bool,
    pub topics: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Owner {
    pub login: String,
    pub avatar_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total_count: u32,
    pub incomplete_results: bool,
    pub items: Vec<Repository>,
}

#[derive(Clone, Debug, Default)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub response: SearchResponse,
    pub rate_limit: Option<RateLimitInfo>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SortBy {
    Stars,
    Forks,
    Issues,
    Created,
    Updated,
}

impl SortBy {
    fn as_str(&self) -> &'static str {
        match self {
            SortBy::Stars => "stars",
            SortBy::Forks => "forks",
            SortBy::Issues => "help-wanted-issues",
            SortBy::Created => "created",
            SortBy::Updated => "updated",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SortOrder {
    Desc,
    Asc,
}

impl SortOrder {
    fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Desc => "desc",
            SortOrder::Asc => "asc",
        }
    }
}

// Results per page options
pub const PER_PAGE_OPTIONS: &[u32] = &[10, 30, 50, 100];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ForkFilter {
    All,
    OriginalOnly,
    ForksOnly,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArchivedFilter {
    All,
    ActiveOnly,
    ArchivedOnly,
}

// Star range presets for the filter
pub const STAR_RANGES: &[(&str, &str)] = &[
    ("Any", ""),
    ("1+", ">=1"),
    ("10+", ">=10"),
    ("100+", ">=100"),
    ("1K+", ">=1000"),
    ("10K+", ">=10000"),
    ("100K+", ">=100000"),
];

// Popular programming languages for the filter
pub const LANGUAGES: &[&str] = &[
    "All",
    "Rust",
    "Python",
    "JavaScript",
    "TypeScript",
    "Go",
    "Java",
    "C",
    "C++",
    "C#",
    "Ruby",
    "PHP",
    "Swift",
    "Kotlin",
    "Scala",
    "Haskell",
    "Elixir",
    "Clojure",
    "Lua",
    "R",
    "Julia",
    "Dart",
    "Zig",
    "Nim",
    "OCaml",
    "Shell",
    "Vue",
    "HTML",
    "CSS",
    "Markdown",
];

#[derive(Clone, Debug, Default)]
pub struct SearchFilters {
    pub query: String,
    pub language: String,
    pub min_stars: String,
    pub fork_filter: ForkFilter,
    pub archived_filter: ArchivedFilter,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub page: u32,
    pub per_page: u32,
}

impl Default for ForkFilter {
    fn default() -> Self {
        ForkFilter::All
    }
}

impl Default for ArchivedFilter {
    fn default() -> Self {
        ArchivedFilter::ActiveOnly
    }
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Stars
    }
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Desc
    }
}

fn build_search_query(filters: &SearchFilters) -> String {
    let mut parts = Vec::new();

    // Add user query or default
    if filters.query.is_empty() && filters.min_stars.is_empty() {
        parts.push("stars:>100".to_string());
    } else if !filters.query.is_empty() {
        parts.push(filters.query.clone());
    }

    // Add language filter
    if filters.language != "All" && !filters.language.is_empty() {
        parts.push(format!("language:{}", filters.language));
    }

    // Add star filter
    if !filters.min_stars.is_empty() {
        parts.push(format!("stars:{}", filters.min_stars));
    }

    // Add fork filter
    match filters.fork_filter {
        ForkFilter::All => {}
        ForkFilter::OriginalOnly => parts.push("fork:false".to_string()),
        ForkFilter::ForksOnly => parts.push("fork:true".to_string()),
    }

    // Add archived filter
    match filters.archived_filter {
        ArchivedFilter::All => {}
        ArchivedFilter::ActiveOnly => parts.push("archived:false".to_string()),
        ArchivedFilter::ArchivedOnly => parts.push("archived:true".to_string()),
    }

    parts.join(" ")
}

async fn search_repositories(filters: SearchFilters) -> Result<SearchResult, String> {
    let search_query = build_search_query(&filters);

    let url = format!(
        "https://api.github.com/search/repositories?q={}&sort={}&order={}&per_page={}&page={}",
        urlencoding(&search_query),
        filters.sort_by.as_str(),
        filters.sort_order.as_str(),
        filters.per_page,
        filters.page
    );

    let response = reqwasm::http::Request::get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "oss-explorer")
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    // Extract rate limit headers
    let rate_limit = extract_rate_limit_info(&response);

    if response.status() == 403 {
        if let Some(ref rl) = rate_limit {
            if rl.remaining == 0 {
                let reset_time = format_reset_time(rl.reset_timestamp);
                return Err(format!(
                    "Rate limit exceeded. Resets at {}. Try again later.",
                    reset_time
                ));
            }
        }
        return Err("Rate limit exceeded. Please try again later.".to_string());
    }

    if response.status() == 422 {
        return Err("Search query too complex or invalid. Try simplifying your search.".to_string());
    }

    if !response.ok() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    let search_response = response
        .json::<SearchResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))?;

    Ok(SearchResult {
        response: search_response,
        rate_limit,
    })
}

fn extract_rate_limit_info(response: &reqwasm::http::Response) -> Option<RateLimitInfo> {
    let limit = response
        .headers()
        .get("x-ratelimit-limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let remaining = response
        .headers()
        .get("x-ratelimit-remaining")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let reset_timestamp = response
        .headers()
        .get("x-ratelimit-reset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if limit > 0 {
        Some(RateLimitInfo {
            limit,
            remaining,
            reset_timestamp,
        })
    } else {
        None
    }
}

fn format_reset_time(timestamp: u64) -> String {
    // Convert Unix timestamp to a readable format
    // Since we're in WASM, we'll use JS Date via web-sys
    use wasm_bindgen::JsValue;
    let date = js_sys::Date::new(&JsValue::from_f64(timestamp as f64 * 1000.0));
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    format!("{:02}:{:02}", hours, minutes)
}

fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push_str("%20"),
            ':' => result.push_str("%3A"),
            '>' => result.push_str("%3E"),
            '<' => result.push_str("%3C"),
            '=' => result.push_str("%3D"),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result
}

fn format_date(date_str: &str) -> String {
    // Parse ISO 8601 date and return a more readable format
    if let Some(date_part) = date_str.split('T').next() {
        date_part.to_string()
    } else {
        date_str.to_string()
    }
}

fn format_number(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn calculate_total_pages(total_count: u32, per_page: u32) -> u32 {
    // GitHub API limits to 1000 results max
    let effective_total = total_count.min(1000);
    (effective_total + per_page - 1) / per_page
}

// Default avatar as a data URI (simple gray circle with user icon)
const DEFAULT_AVATAR: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 40 40'%3E%3Ccircle cx='20' cy='20' r='20' fill='%2330363d'/%3E%3Ccircle cx='20' cy='16' r='7' fill='%238b949e'/%3E%3Cpath d='M6 36c0-8 6-14 14-14s14 6 14 14' fill='%238b949e'/%3E%3C/svg%3E";

/// Validates that a URL is safe to use (not a browser extension URL or other problematic scheme)
fn is_safe_image_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    // Allow https, http, and data URIs only
    url_lower.starts_with("https://")
        || url_lower.starts_with("http://")
        || url_lower.starts_with("data:")
}

/// Returns a safe avatar URL, falling back to default if the URL is invalid
fn get_safe_avatar_url(url: &str) -> String {
    if is_safe_image_url(url) {
        url.to_string()
    } else {
        DEFAULT_AVATAR.to_string()
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (language, set_language) = signal("All".to_string());
    let (min_stars, set_min_stars) = signal(String::new());
    let (fork_filter, set_fork_filter) = signal(ForkFilter::All);
    let (archived_filter, set_archived_filter) = signal(ArchivedFilter::ActiveOnly);
    let (sort_by, set_sort_by) = signal(SortBy::Stars);
    let (sort_order, set_sort_order) = signal(SortOrder::Desc);
    let (repositories, set_repositories) = signal(Vec::<Repository>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (total_count, set_total_count) = signal(0u32);
    let (current_page, set_current_page) = signal(1u32);
    let (per_page, set_per_page) = signal(30u32);
    let (rate_limit, set_rate_limit) = signal(Option::<RateLimitInfo>::None);
    let (incomplete_results, set_incomplete_results) = signal(false);
    let (show_advanced, set_show_advanced) = signal(false);

    let total_pages = move || calculate_total_pages(total_count.get(), per_page.get());

    let do_search = move |page: u32| {
        let filters = SearchFilters {
            query: query.get(),
            language: language.get(),
            min_stars: min_stars.get(),
            fork_filter: fork_filter.get(),
            archived_filter: archived_filter.get(),
            sort_by: sort_by.get(),
            sort_order: sort_order.get(),
            page,
            per_page: per_page.get(),
        };

        set_loading.set(true);
        set_error.set(None);
        set_current_page.set(page);

        leptos::task::spawn_local(async move {
            match search_repositories(filters).await {
                Ok(result) => {
                    set_total_count.set(result.response.total_count);
                    set_repositories.set(result.response.items);
                    set_rate_limit.set(result.rate_limit);
                    set_incomplete_results.set(result.response.incomplete_results);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    };

    let go_to_page = move |page: u32| {
        if page >= 1 && page <= total_pages() && !loading.get() {
            do_search(page);
        }
    };

    let go_prev = move |_| {
        let page = current_page.get();
        if page > 1 {
            go_to_page(page - 1);
        }
    };

    let go_next = move |_| {
        let page = current_page.get();
        if page < total_pages() {
            go_to_page(page + 1);
        }
    };

    let go_first = move |_| {
        go_to_page(1);
    };

    let go_last = move |_| {
        go_to_page(total_pages());
    };

    let clear_filters = move |_| {
        set_query.set(String::new());
        set_language.set("All".to_string());
        set_min_stars.set(String::new());
        set_fork_filter.set(ForkFilter::All);
        set_archived_filter.set(ArchivedFilter::ActiveOnly);
        set_sort_by.set(SortBy::Stars);
        set_sort_order.set(SortOrder::Desc);
        do_search(1);
    };

    // Initial search on load
    {
        let do_search = do_search.clone();
        Effect::new(move |_| {
            do_search(1);
        });
    }

    view! {
        <div class="app">
            <header>
                <h1>"OSS Explorer"</h1>
                <p class="subtitle">"Discover open source projects by language, creation date, and activity"</p>
            </header>

            <div class="controls">
                <div class="search-box">
                    <input
                        type="text"
                        placeholder="Search repositories (e.g., 'web framework', 'machine learning')"
                        prop:value=move || query.get()
                        on:input=move |ev| {
                            set_query.set(event_target_value(&ev));
                        }
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                do_search(1);
                            }
                        }
                    />
                    <button on:click=move |_| do_search(1) disabled=move || loading.get()>
                        {move || if loading.get() { "Searching..." } else { "Search" }}
                    </button>
                </div>

                <div class="filters">
                    <div class="filter-group">
                        <label>"Language:"</label>
                        <select on:change=move |ev| {
                            set_language.set(event_target_value(&ev));
                            do_search(1);
                        }>
                            {LANGUAGES.iter().map(|lang| {
                                view! {
                                    <option value=*lang selected=move || language.get() == *lang>
                                        {*lang}
                                    </option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>

                    <div class="filter-group">
                        <label>"Min Stars:"</label>
                        <select on:change=move |ev| {
                            set_min_stars.set(event_target_value(&ev));
                            do_search(1);
                        }>
                            {STAR_RANGES.iter().map(|(label, value)| {
                                view! {
                                    <option value=*value selected=move || min_stars.get() == *value>
                                        {*label}
                                    </option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>

                    <div class="filter-group">
                        <label>"Sort by:"</label>
                        <select on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_sort_by.set(match value.as_str() {
                                "stars" => SortBy::Stars,
                                "forks" => SortBy::Forks,
                                "issues" => SortBy::Issues,
                                "created" => SortBy::Created,
                                "updated" => SortBy::Updated,
                                _ => SortBy::Stars,
                            });
                            do_search(1);
                        }>
                            <option value="stars" selected=move || sort_by.get() == SortBy::Stars>"Stars"</option>
                            <option value="forks" selected=move || sort_by.get() == SortBy::Forks>"Forks"</option>
                            <option value="issues" selected=move || sort_by.get() == SortBy::Issues>"Issues"</option>
                            <option value="created" selected=move || sort_by.get() == SortBy::Created>"Created Date"</option>
                            <option value="updated" selected=move || sort_by.get() == SortBy::Updated>"Last Updated"</option>
                        </select>
                    </div>

                    <div class="filter-group">
                        <label>"Order:"</label>
                        <select on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_sort_order.set(if value == "asc" { SortOrder::Asc } else { SortOrder::Desc });
                            do_search(1);
                        }>
                            <option value="desc" selected=move || sort_order.get() == SortOrder::Desc>"Descending"</option>
                            <option value="asc" selected=move || sort_order.get() == SortOrder::Asc>"Ascending"</option>
                        </select>
                    </div>

                    <div class="filter-group">
                        <label>"Per page:"</label>
                        <select on:change=move |ev| {
                            let value: u32 = event_target_value(&ev).parse().unwrap_or(30);
                            set_per_page.set(value);
                            do_search(1);
                        }>
                            {PER_PAGE_OPTIONS.iter().map(|&n| {
                                view! {
                                    <option value=n.to_string() selected=move || per_page.get() == n>
                                        {n.to_string()}
                                    </option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                </div>

                <div class="advanced-toggle">
                    <button class="toggle-btn" on:click=move |_| set_show_advanced.update(|v| *v = !*v)>
                        {move || if show_advanced.get() { "Hide Advanced Filters" } else { "Show Advanced Filters" }}
                    </button>
                    <button class="clear-btn" on:click=clear_filters>
                        "Clear All Filters"
                    </button>
                </div>

                {move || show_advanced.get().then(|| view! {
                    <div class="advanced-filters">
                        <div class="filter-group">
                            <label>"Repository Type:"</label>
                            <select on:change=move |ev| {
                                let value = event_target_value(&ev);
                                set_fork_filter.set(match value.as_str() {
                                    "original" => ForkFilter::OriginalOnly,
                                    "forks" => ForkFilter::ForksOnly,
                                    _ => ForkFilter::All,
                                });
                                do_search(1);
                            }>
                                <option value="all" selected=move || fork_filter.get() == ForkFilter::All>"All Repos"</option>
                                <option value="original" selected=move || fork_filter.get() == ForkFilter::OriginalOnly>"Original Only"</option>
                                <option value="forks" selected=move || fork_filter.get() == ForkFilter::ForksOnly>"Forks Only"</option>
                            </select>
                        </div>

                        <div class="filter-group">
                            <label>"Status:"</label>
                            <select on:change=move |ev| {
                                let value = event_target_value(&ev);
                                set_archived_filter.set(match value.as_str() {
                                    "active" => ArchivedFilter::ActiveOnly,
                                    "archived" => ArchivedFilter::ArchivedOnly,
                                    _ => ArchivedFilter::All,
                                });
                                do_search(1);
                            }>
                                <option value="active" selected=move || archived_filter.get() == ArchivedFilter::ActiveOnly>"Active Only"</option>
                                <option value="all" selected=move || archived_filter.get() == ArchivedFilter::All>"All (incl. Archived)"</option>
                                <option value="archived" selected=move || archived_filter.get() == ArchivedFilter::ArchivedOnly>"Archived Only"</option>
                            </select>
                        </div>
                    </div>
                })}
            </div>

            // Rate limit indicator
            {move || rate_limit.get().map(|rl| {
                let percentage = (rl.remaining as f64 / rl.limit as f64) * 100.0;
                let status_class = if percentage > 50.0 {
                    "rate-limit-ok"
                } else if percentage > 20.0 {
                    "rate-limit-warning"
                } else {
                    "rate-limit-danger"
                };
                view! {
                    <div class=format!("rate-limit-info {}", status_class)>
                        <span class="rate-limit-label">"API Rate Limit: "</span>
                        <span class="rate-limit-value">{rl.remaining}" / "{rl.limit}</span>
                        {(rl.remaining < 10).then(|| view! {
                            <span class="rate-limit-reset">" (resets at "{format_reset_time(rl.reset_timestamp)}")"</span>
                        })}
                    </div>
                }
            })}

            {move || error.get().map(|e| view! {
                <div class="error">
                    <strong>"Error: "</strong>{e}
                </div>
            })}

            {move || incomplete_results.get().then(|| view! {
                <div class="warning">
                    <strong>"Warning: "</strong>"Results may be incomplete due to GitHub API timeout. Try a more specific search."
                </div>
            })}

            <div class="results-header">
                <span class="count">
                    {move || {
                        let total = total_count.get();
                        if total > 1000 {
                            format!("{} repositories found (showing first 1,000)", format_number(total))
                        } else {
                            format!("{} repositories found", format_number(total))
                        }
                    }}
                </span>
                <span class="page-info">
                    {move || format!("Page {} of {}", current_page.get(), total_pages().max(1))}
                </span>
            </div>

            <div class="results">
                {move || {
                    if loading.get() && repositories.get().is_empty() {
                        view! { <div class="loading">"Loading repositories..."</div> }.into_any()
                    } else if repositories.get().is_empty() {
                        view! { <div class="empty">"No repositories found. Try a different search."</div> }.into_any()
                    } else {
                        view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th>"Repository"</th>
                                        <th>"Language"</th>
                                        <th>"Stars"</th>
                                        <th>"Forks"</th>
                                        <th>"Issues"</th>
                                        <th>"Created"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || repositories.get().into_iter().map(|repo| {
                                        let repo_url = repo.html_url.clone();
                                        let repo_name = repo.full_name.clone();
                                        let description = repo.description.clone().unwrap_or_default();
                                        let language = repo.language.clone().unwrap_or_else(|| "Unknown".to_string());
                                        let stars = format_number(repo.stargazers_count);
                                        let forks = format_number(repo.forks_count);
                                        let issues = format_number(repo.open_issues_count);
                                        let created = format_date(&repo.created_at);
                                        let avatar = get_safe_avatar_url(&repo.owner.avatar_url);
                                        let fallback_avatar = DEFAULT_AVATAR.to_string();
                                        let is_fork = repo.fork;
                                        let is_archived = repo.archived;

                                        view! {
                                            <tr class:archived=is_archived class:forked=is_fork>
                                                <td class="repo-cell">
                                                    <div class="repo-info">
                                                        <img
                                                            src=avatar
                                                            alt="avatar"
                                                            class="avatar"
                                                            on:error=move |ev| {
                                                                // Replace with default avatar on load error
                                                                if let Some(target) = ev.target() {
                                                                    use wasm_bindgen::JsCast;
                                                                    if let Ok(img) = target.dyn_into::<web_sys::HtmlImageElement>() {
                                                                        img.set_src(&fallback_avatar);
                                                                    }
                                                                }
                                                            }
                                                        />
                                                        <div class="repo-details">
                                                            <div class="repo-name-row">
                                                                <a href=repo_url target="_blank" class="repo-name">
                                                                    {repo_name}
                                                                </a>
                                                                {is_fork.then(|| view! { <span class="badge fork-badge">"Fork"</span> })}
                                                                {is_archived.then(|| view! { <span class="badge archived-badge">"Archived"</span> })}
                                                            </div>
                                                            <p class="repo-description">{description}</p>
                                                        </div>
                                                    </div>
                                                </td>
                                                <td><span class="language-badge">{language}</span></td>
                                                <td class="stat">{stars}</td>
                                                <td class="stat">{forks}</td>
                                                <td class="stat">{issues}</td>
                                                <td class="date">{created}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        }.into_any()
                    }
                }}
            </div>

            // Pagination controls
            {move || (total_pages() > 1).then(|| {
                let page = current_page.get();
                let pages = total_pages();
                view! {
                    <div class="pagination">
                        <button
                            class="page-btn"
                            on:click=go_first
                            disabled=move || page == 1 || loading.get()
                        >
                            "First"
                        </button>
                        <button
                            class="page-btn"
                            on:click=go_prev
                            disabled=move || page == 1 || loading.get()
                        >
                            "Prev"
                        </button>

                        <div class="page-numbers">
                            {(1..=pages).filter(move |&p| {
                                // Show first, last, current, and 2 pages around current
                                p == 1 || p == pages || (p >= page.saturating_sub(2) && p <= page + 2)
                            }).map(|p| {
                                let show_ellipsis_before = p > 1 && p > page.saturating_sub(2) && p != 2;
                                let show_ellipsis_after = p < pages && p < page + 2 && p != pages - 1;
                                view! {
                                    <>
                                        {show_ellipsis_before.then(|| view! { <span class="ellipsis">"..."</span> })}
                                        <button
                                            class="page-num"
                                            class:active=move || current_page.get() == p
                                            on:click=move |_| go_to_page(p)
                                            disabled=move || loading.get()
                                        >
                                            {p}
                                        </button>
                                        {show_ellipsis_after.then(|| view! { <span class="ellipsis">"..."</span> })}
                                    </>
                                }
                            }).collect::<Vec<_>>()}
                        </div>

                        <button
                            class="page-btn"
                            on:click=go_next
                            disabled=move || page == pages || loading.get()
                        >
                            "Next"
                        </button>
                        <button
                            class="page-btn"
                            on:click=go_last
                            disabled=move || page == pages || loading.get()
                        >
                            "Last"
                        </button>
                    </div>
                }
            })}

            <footer>
                <p>"Powered by the GitHub API | Built with Rust + Leptos"</p>
            </footer>
        </div>
    }
}
