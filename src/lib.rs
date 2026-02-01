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
    } else {
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

async fn search_repositories(filters: SearchFilters) -> Result<SearchResponse, String> {
    let search_query = build_search_query(&filters);

    let url = format!(
        "https://api.github.com/search/repositories?q={}&sort={}&order={}&per_page=30",
        urlencoding(&search_query),
        filters.sort_by.as_str(),
        filters.sort_order.as_str()
    );

    let response = reqwasm::http::Request::get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "oss-explorer")
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.status() == 403 {
        return Err("Rate limit exceeded. Please try again later.".to_string());
    }

    if !response.ok() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    response
        .json::<SearchResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))
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
    let (show_advanced, set_show_advanced) = signal(false);

    let do_search = move || {
        let filters = SearchFilters {
            query: query.get(),
            language: language.get(),
            min_stars: min_stars.get(),
            fork_filter: fork_filter.get(),
            archived_filter: archived_filter.get(),
            sort_by: sort_by.get(),
            sort_order: sort_order.get(),
        };

        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match search_repositories(filters).await {
                Ok(response) => {
                    set_total_count.set(response.total_count);
                    set_repositories.set(response.items);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    };

    let clear_filters = move |_| {
        set_query.set(String::new());
        set_language.set("All".to_string());
        set_min_stars.set(String::new());
        set_fork_filter.set(ForkFilter::All);
        set_archived_filter.set(ArchivedFilter::ActiveOnly);
        set_sort_by.set(SortBy::Stars);
        set_sort_order.set(SortOrder::Desc);
        do_search();
    };

    // Initial search on load
    {
        let do_search = do_search.clone();
        Effect::new(move |_| {
            do_search();
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
                                do_search();
                            }
                        }
                    />
                    <button on:click=move |_| do_search() disabled=move || loading.get()>
                        {move || if loading.get() { "Searching..." } else { "Search" }}
                    </button>
                </div>

                <div class="filters">
                    <div class="filter-group">
                        <label>"Language:"</label>
                        <select on:change=move |ev| {
                            set_language.set(event_target_value(&ev));
                            do_search();
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
                            do_search();
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
                            do_search();
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
                            do_search();
                        }>
                            <option value="desc" selected=move || sort_order.get() == SortOrder::Desc>"Descending"</option>
                            <option value="asc" selected=move || sort_order.get() == SortOrder::Asc>"Ascending"</option>
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
                                do_search();
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
                                do_search();
                            }>
                                <option value="active" selected=move || archived_filter.get() == ArchivedFilter::ActiveOnly>"Active Only"</option>
                                <option value="all" selected=move || archived_filter.get() == ArchivedFilter::All>"All (incl. Archived)"</option>
                                <option value="archived" selected=move || archived_filter.get() == ArchivedFilter::ArchivedOnly>"Archived Only"</option>
                            </select>
                        </div>
                    </div>
                })}
            </div>

            {move || error.get().map(|e| view! {
                <div class="error">
                    <strong>"Error: "</strong>{e}
                </div>
            })}

            <div class="results-header">
                <span class="count">
                    {move || format!("{} repositories found", format_number(total_count.get()))}
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

            <footer>
                <p>"Powered by the GitHub API | Built with Rust + Leptos"</p>
            </footer>
        </div>
    }
}
