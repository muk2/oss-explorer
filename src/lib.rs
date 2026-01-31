use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;
use leptos_router::hooks::use_params_map;
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
    pub topics: Option<Vec<String>>,
    pub license: Option<License>,
    pub default_branch: Option<String>,
    pub watchers_count: Option<u32>,
    pub subscribers_count: Option<u32>,
    pub size: Option<u32>,
    #[serde(default)]
    pub fork: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub has_wiki: bool,
    #[serde(default)]
    pub has_issues: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Owner {
    pub login: String,
    pub avatar_url: String,
    pub html_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct License {
    pub key: String,
    pub name: String,
    pub spdx_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total_count: u32,
    pub incomplete_results: bool,
    pub items: Vec<Repository>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contributor {
    pub login: String,
    pub avatar_url: String,
    pub html_url: String,
    pub contributions: u32,
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
];

async fn search_repositories(
    query: String,
    language: String,
    sort_by: SortBy,
    sort_order: SortOrder,
) -> Result<SearchResponse, String> {
    let mut search_query = if query.is_empty() {
        "stars:>100".to_string()
    } else {
        query
    };

    if language != "All" && !language.is_empty() {
        search_query = format!("{} language:{}", search_query, language);
    }

    let url = format!(
        "https://api.github.com/search/repositories?q={}&sort={}&order={}&per_page=30",
        urlencoding(&search_query),
        sort_by.as_str(),
        sort_order.as_str()
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

async fn fetch_repository(owner: &str, repo: &str) -> Result<Repository, String> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

    let response = reqwasm::http::Request::get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "oss-explorer")
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.status() == 404 {
        return Err("Repository not found.".to_string());
    }

    if response.status() == 403 {
        return Err("Rate limit exceeded. Please try again later.".to_string());
    }

    if !response.ok() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    response
        .json::<Repository>()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))
}

async fn fetch_contributors(owner: &str, repo: &str) -> Result<Vec<Contributor>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contributors?per_page=10",
        owner, repo
    );

    let response = reqwasm::http::Request::get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "oss-explorer")
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if !response.ok() {
        return Ok(Vec::new()); // Return empty on error
    }

    response
        .json::<Vec<Contributor>>()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))
}

async fn fetch_readme(owner: &str, repo: &str) -> Result<String, String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/readme",
        owner, repo
    );

    let response = reqwasm::http::Request::get(&url)
        .header("Accept", "application/vnd.github.html+json")
        .header("User-Agent", "oss-explorer")
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if !response.ok() {
        return Err("README not found".to_string());
    }

    response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {:?}", e))
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

fn format_size(kb: u32) -> String {
    if kb >= 1_000_000 {
        format!("{:.1} GB", kb as f64 / 1_000_000.0)
    } else if kb >= 1_000 {
        format!("{:.1} MB", kb as f64 / 1_000.0)
    } else {
        format!("{} KB", kb)
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Page not found">
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/repo/:owner/:name") view=RepoDetailPage />
            </Routes>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (language, set_language) = signal("All".to_string());
    let (sort_by, set_sort_by) = signal(SortBy::Stars);
    let (sort_order, set_sort_order) = signal(SortOrder::Desc);
    let (repositories, set_repositories) = signal(Vec::<Repository>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (total_count, set_total_count) = signal(0u32);

    let do_search = move || {
        let q = query.get();
        let lang = language.get();
        let sort = sort_by.get();
        let order = sort_order.get();

        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match search_repositories(q, lang, sort, order).await {
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
                                        <th>"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || repositories.get().into_iter().map(|repo| {
                                        let detail_url = format!("/repo/{}", repo.full_name);
                                        let github_url = repo.html_url.clone();
                                        let repo_name = repo.full_name.clone();
                                        let description = repo.description.clone().unwrap_or_default();
                                        let language = repo.language.clone().unwrap_or_else(|| "Unknown".to_string());
                                        let stars = format_number(repo.stargazers_count);
                                        let forks = format_number(repo.forks_count);
                                        let issues = format_number(repo.open_issues_count);
                                        let created = format_date(&repo.created_at);
                                        let avatar = repo.owner.avatar_url.clone();

                                        view! {
                                            <tr>
                                                <td class="repo-cell">
                                                    <div class="repo-info">
                                                        <img src=avatar alt="avatar" class="avatar" />
                                                        <div class="repo-details">
                                                            <A href=detail_url.clone() class="repo-name">
                                                                {repo_name}
                                                            </A>
                                                            <p class="repo-description">{description}</p>
                                                        </div>
                                                    </div>
                                                </td>
                                                <td><span class="language-badge">{language}</span></td>
                                                <td class="stat">{stars}</td>
                                                <td class="stat">{forks}</td>
                                                <td class="stat">{issues}</td>
                                                <td class="date">{created}</td>
                                                <td class="actions">
                                                    <A href=detail_url class="action-btn view-btn">"View Details"</A>
                                                    <a href=github_url target="_blank" class="action-btn github-btn">"GitHub"</a>
                                                </td>
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

#[component]
fn RepoDetailPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.get().get("owner").unwrap_or_default();
    let name = move || params.get().get("name").unwrap_or_default();

    let (repo, set_repo) = signal(Option::<Repository>::None);
    let (contributors, set_contributors) = signal(Vec::<Contributor>::new());
    let (readme, set_readme) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    // Fetch repo data when component loads
    Effect::new(move |_| {
        let o = owner();
        let n = name();
        if o.is_empty() || n.is_empty() {
            return;
        }

        set_loading.set(true);
        set_error.set(None);

        let o_clone = o.clone();
        let n_clone = n.clone();

        leptos::task::spawn_local(async move {
            // Fetch repository details
            match fetch_repository(&o_clone, &n_clone).await {
                Ok(r) => {
                    set_repo.set(Some(r));
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                    return;
                }
            }

            // Fetch contributors
            if let Ok(c) = fetch_contributors(&o_clone, &n_clone).await {
                set_contributors.set(c);
            }

            // Fetch README
            if let Ok(r) = fetch_readme(&o_clone, &n_clone).await {
                set_readme.set(Some(r));
            }

            set_loading.set(false);
        });
    });

    view! {
        <div class="app">
            <header class="detail-header">
                <A href="/" class="back-link">"< Back to Search"</A>
                <h1>"Repository Details"</h1>
            </header>

            {move || error.get().map(|e| view! {
                <div class="error">
                    <strong>"Error: "</strong>{e}
                </div>
            })}

            {move || {
                if loading.get() {
                    view! { <div class="loading">"Loading repository details..."</div> }.into_any()
                } else if let Some(r) = repo.get() {
                    let github_url = r.html_url.clone();
                    let owner_url = r.owner.html_url.clone().unwrap_or_default();
                    let topics = r.topics.clone().unwrap_or_default();
                    let license_name = r.license.as_ref().map(|l| l.name.clone()).unwrap_or_else(|| "No license".to_string());
                    let default_branch = r.default_branch.clone().unwrap_or_else(|| "main".to_string());

                    view! {
                        <div class="repo-detail">
                            <div class="repo-header">
                                <img src=r.owner.avatar_url.clone() alt="owner avatar" class="repo-avatar" />
                                <div class="repo-header-info">
                                    <h2 class="repo-title">{r.full_name.clone()}</h2>
                                    <p class="repo-desc">{r.description.clone().unwrap_or_else(|| "No description".to_string())}</p>
                                    <div class="repo-meta">
                                        <a href=owner_url target="_blank" class="owner-link">
                                            "@"{r.owner.login.clone()}
                                        </a>
                                        {r.fork.then(|| view! { <span class="badge fork-badge">"Fork"</span> })}
                                        {r.archived.then(|| view! { <span class="badge archived-badge">"Archived"</span> })}
                                    </div>
                                </div>
                                <a href=github_url target="_blank" class="github-link">"View on GitHub"</a>
                            </div>

                            <div class="stats-grid">
                                <div class="stat-card">
                                    <span class="stat-value">{format_number(r.stargazers_count)}</span>
                                    <span class="stat-label">"Stars"</span>
                                </div>
                                <div class="stat-card">
                                    <span class="stat-value">{format_number(r.forks_count)}</span>
                                    <span class="stat-label">"Forks"</span>
                                </div>
                                <div class="stat-card">
                                    <span class="stat-value">{format_number(r.open_issues_count)}</span>
                                    <span class="stat-label">"Open Issues"</span>
                                </div>
                                <div class="stat-card">
                                    <span class="stat-value">{format_number(r.watchers_count.unwrap_or(0))}</span>
                                    <span class="stat-label">"Watchers"</span>
                                </div>
                            </div>

                            <div class="detail-sections">
                                <div class="detail-section">
                                    <h3>"Information"</h3>
                                    <div class="info-grid">
                                        <div class="info-item">
                                            <span class="info-label">"Language"</span>
                                            <span class="info-value">{r.language.clone().unwrap_or_else(|| "Unknown".to_string())}</span>
                                        </div>
                                        <div class="info-item">
                                            <span class="info-label">"License"</span>
                                            <span class="info-value">{license_name}</span>
                                        </div>
                                        <div class="info-item">
                                            <span class="info-label">"Default Branch"</span>
                                            <span class="info-value">{default_branch}</span>
                                        </div>
                                        <div class="info-item">
                                            <span class="info-label">"Size"</span>
                                            <span class="info-value">{format_size(r.size.unwrap_or(0))}</span>
                                        </div>
                                        <div class="info-item">
                                            <span class="info-label">"Created"</span>
                                            <span class="info-value">{format_date(&r.created_at)}</span>
                                        </div>
                                        <div class="info-item">
                                            <span class="info-label">"Last Updated"</span>
                                            <span class="info-value">{format_date(&r.updated_at)}</span>
                                        </div>
                                    </div>
                                </div>

                                {(!topics.is_empty()).then(|| {
                                    let topics_clone = topics.clone();
                                    view! {
                                        <div class="detail-section">
                                            <h3>"Topics"</h3>
                                            <div class="topics-list">
                                                {topics_clone.into_iter().map(|topic| {
                                                    view! { <span class="topic-badge">{topic}</span> }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    }
                                })}

                                {move || {
                                    let contribs = contributors.get();
                                    (!contribs.is_empty()).then(|| {
                                        view! {
                                            <div class="detail-section">
                                                <h3>"Top Contributors"</h3>
                                                <div class="contributors-list">
                                                    {contribs.into_iter().map(|c| {
                                                        view! {
                                                            <a href=c.html_url.clone() target="_blank" class="contributor">
                                                                <img src=c.avatar_url.clone() alt=c.login.clone() class="contributor-avatar" />
                                                                <span class="contributor-name">{c.login.clone()}</span>
                                                                <span class="contributor-commits">{c.contributions}" commits"</span>
                                                            </a>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        }
                                    })
                                }}

                                {move || readme.get().map(|content| {
                                    view! {
                                        <div class="detail-section readme-section">
                                            <h3>"README"</h3>
                                            <div class="readme-content" inner_html=content></div>
                                        </div>
                                    }
                                })}
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div class="empty">"Repository not found."</div> }.into_any()
                }
            }}

            <footer>
                <p>"Powered by the GitHub API | Built with Rust + Leptos"</p>
            </footer>
        </div>
    }
}
