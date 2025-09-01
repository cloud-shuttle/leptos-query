use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_query::*;
use crate::types::*;
use crate::api::*;
use crate::components::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Stylesheet id="leptos" href="/pkg/leptos-query-demo.css"/>
        <Title text="Leptos Query Demo - GitHub Issues Viewer"/>
        
        <div class="app">
            <header class="app-header">
                <div class="header-content">
                    <h1>
                        <a href="/">"Leptos Query Demo"</a>
                    </h1>
                    <p class="subtitle">"GitHub Issues Viewer - Powered by leptos-query"</p>
                </div>
            </header>

            <main class="app-main">
                <Router>
                    <Routes>
                        <Route path="/" view=HomePage/>
                        <Route path="/repo/:owner/:repo" view=RepoPage/>
                        <Route path="/issue/:number" view=IssuePage/>
                        <Route path="/*any" view=NotFound/>
                    </Routes>
                </Router>
            </main>

            <footer class="app-footer">
                <p>"Built with " <a href="https://github.com/cloud-shuttle/leptos-query" target="_blank">"leptos-query"</a></p>
            </footer>
        </div>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="home-page">
            <div class="hero-section">
                <h2>"Welcome to the Leptos Query Demo"</h2>
                <p>"This demo showcases the power of leptos-query for data fetching and caching in Leptos applications."</p>
                
                <div class="features">
                    <div class="feature">
                        <h3>"🔄 Automatic Caching"</h3>
                        <p>"Queries are automatically cached and shared across components"</p>
                    </div>
                    <div class="feature">
                        <h3>"⚡ Background Updates"</h3>
                        <p>"Data stays fresh with automatic background refetching"</p>
                    </div>
                    <div class="feature">
                        <h3>"🎯 Optimistic Updates"</h3>
                        <p>"Immediate UI feedback with optimistic mutations"</p>
                    </div>
                    <div class="feature">
                        <h3>"🛡️ Error Handling"</h3>
                        <p>"Built-in error handling and retry logic"</p>
                    </div>
                </div>
            </div>

            <div class="demo-section">
                <h3>"Try it out!"</h3>
                <p>"Search for a GitHub repository to see leptos-query in action:"</p>
                
                <RepositorySearch/>
            </div>

            <div class="examples-section">
                <h3>"Example Repositories"</h3>
                <div class="example-repos">
                    <a href="/repo/rust-lang/rust" class="example-repo">
                        <h4>"rust-lang/rust"</h4>
                        <p>"The Rust Programming Language"</p>
                    </a>
                    <a href="/repo/leptos-rs/leptos" class="example-repo">
                        <h4>"leptos-rs/leptos"</h4>
                        <p>"A full-stack, isomorphic Rust web framework"</p>
                    </a>
                    <a href="/repo/cloud-shuttle/leptos-query" class="example-repo">
                        <h4>"cloud-shuttle/leptos-query"</h4>
                        <p>"Data fetching and caching for Leptos"</p>
                    </a>
                </div>
            </div>
        </div>
    }
}

#[component]
fn RepoPage() -> impl IntoView {
    let params = use_params::<RepoParams>();
    let api = get_api();
    
    let owner = move || params.get().map(|p| p.owner).unwrap_or_default();
    let repo = move || params.get().map(|p| p.repo).unwrap_or_default();
    
    // Query for repository information
    let repo_query = use_query(
        move || &["repo", &owner(), &repo()][..],
        move || {
            let api = api.clone();
            let owner = owner();
            let repo = repo();
            async move { api.get_repository(&owner, &repo).await }
        },
        QueryOptions::default()
    );

    // Query for issues
    let issues_query = use_query(
        move || &["issues", &owner(), &repo()][..],
        move || {
            let api = api.clone();
            let owner = owner();
            let repo = repo();
            let filters = IssueFilters {
                state: "open".to_string(),
                labels: vec![],
                assignee: None,
                creator: None,
                sort: "created".to_string(),
                direction: "desc".to_string(),
            };
            async move { api.get_issues(&owner, &repo, &filters).await }
        },
        QueryOptions::default()
    );

    view! {
        <div class="repo-page">
            {move || match repo_query.data() {
                Some(Ok(repo_data)) => view! {
                    <div class="repo-info">
                        <div class="repo-header">
                            <h2>
                                <a href=repo_data.html_url target="_blank">
                                    {repo_data.full_name}
                                </a>
                            </h2>
                            <div class="repo-stats">
                                <span class="stars">{"⭐ "}{repo_data.stargazers_count}</span>
                                <span class="forks">{"🍴 "}{repo_data.forks_count}</span>
                                {if let Some(lang) = &repo_data.language {
                                    view! { <span class="language">{lang}</span> }
                                } else {
                                    view! { <span></span> }
                                }}
                            </div>
                        </div>
                        <p class="repo-description">
                            {repo_data.description.unwrap_or_else(|| "No description".to_string())}
                        </p>
                    </div>
                }.into_view(),
                Some(Err(e)) => view! {
                    <div class="error">
                        <h2>"Error loading repository"</h2>
                        <p>{e}</p>
                    </div>
                }.into_view(),
                None if repo_query.is_loading() => view! {
                    <div class="loading">
                        <h2>"Loading repository..."</h2>
                    </div>
                }.into_view(),
                None => view! {
                    <div class="loading">
                        <h2>"Initializing..."</h2>
                    </div>
                }.into_view(),
            }}

            <div class="issues-section">
                <h3>"Recent Issues"</h3>
                {move || match issues_query.data() {
                    Some(Ok(issues)) => view! {
                        <div class="issues-list">
                            {if issues.is_empty() {
                                view! { <p class="no-issues">"No open issues found"</p> }
                            } else {
                                view! {
                                    <div>
                                        {issues.iter().map(|issue| {
                                            view! { <IssueItem issue=issue.clone()/> }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }
                            }}
                        </div>
                    }.into_view(),
                    Some(Err(e)) => view! {
                        <div class="error">
                            <p>"Error loading issues: "{e}</p>
                        </div>
                    }.into_view(),
                    None if issues_query.is_loading() => view! {
                        <div class="loading">
                            <p>"Loading issues..."</p>
                        </div>
                    }.into_view(),
                    None => view! {
                        <div class="loading">
                            <p>"Waiting for repository data..."</p>
                        </div>
                    }.into_view(),
                }}
            </div>
        </div>
    }
}

#[component]
fn IssuePage() -> impl IntoView {
    let params = use_params::<IssueParams>();
    
    let issue_number = move || params.get().map(|p| p.number).unwrap_or(0);
    
    // For demo purposes, we'll use a hardcoded repository
    // In a real app, this would come from the URL or context
    let owner = "cloud-shuttle".to_string();
    let repo = "leptos-query".to_string();
    
    view! {
        <div class="issue-page">
            <div class="issue-navigation">
                <a href="/" class="back-link">"← Back to Home"</a>
                <a href=format!("/repo/{}/{}", owner, repo) class="repo-link">
                    {"Back to "}{owner}{"/"}{repo}
                </a>
            </div>
            
            <IssueDetail owner=owner repo=repo issue_number=issue_number()/>
        </div>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="not-found">
            <h2>"404 - Page Not Found"</h2>
            <p>"The page you're looking for doesn't exist."</p>
            <a href="/" class="home-link">"Go back home"</a>
        </div>
    }
}

#[derive(Params, PartialEq, Clone)]
struct RepoParams {
    owner: String,
    repo: String,
}

#[derive(Params, PartialEq, Clone)]
struct IssueParams {
    number: u32,
}
