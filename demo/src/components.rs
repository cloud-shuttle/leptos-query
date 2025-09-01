use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_query::*;
use crate::types::*;
use crate::api::*;

// Repository card component
#[component]
pub fn RepositoryCard(repo: Repository) -> impl IntoView {
    view! {
        <div class="repo-card">
            <div class="repo-header">
                <h3>
                    <a href=repo.html_url target="_blank">
                        {repo.full_name}
                    </a>
                </h3>
                <div class="repo-stats">
                    <span class="stars">{"⭐ "}{repo.stargazers_count}</span>
                    <span class="forks">{"🍴 "}{repo.forks_count}</span>
                </div>
            </div>
            <p class="repo-description">
                {repo.description.unwrap_or_else(|| "No description".to_string())}
            </p>
            <div class="repo-meta">
                {if let Some(lang) = repo.language {
                    view! { <span class="language">{lang}</span> }
                } else {
                    view! { <span class="language">"Unknown"</span> }
                }}
                <span class="updated">
                    {"Updated "}{repo.updated_at.format("%Y-%m-%d").to_string()}
                </span>
            </div>
        </div>
    }
}

// Issue list item component
#[component]
pub fn IssueItem(issue: Issue) -> impl IntoView {
    let navigate = use_navigate();
    
    view! {
        <div class="issue-item" on:click=move |_| {
            navigate(&format!("/issue/{}", issue.number), Default::default());
        }>
            <div class="issue-header">
                <h4 class="issue-title">{issue.title}</h4>
                <span class=format!("issue-state issue-{}", issue.state)>
                    {issue.state.to_uppercase()}
                </span>
            </div>
            <div class="issue-meta">
                <span class="issue-number">{"#"}{issue.number}</span>
                <span class="issue-author">
                    {"by "}{issue.user.login}
                </span>
                <span class="issue-date">
                    {issue.created_at.format("%Y-%m-%d").to_string()}
                </span>
                {if issue.comments > 0 {
                    view! {
                        <span class="issue-comments">
                            {"💬 "}{issue.comments}
                        </span>
                    }
                } else {
                    view! { <span></span> }
                }}
            </div>
            <div class="issue-labels">
                {issue.labels.iter().map(|label| {
                    let style = format!("background-color: #{};", label.color);
                    view! {
                        <span class="label" style=style>
                            {label.name.clone()}
                        </span>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

// Issue detail component
#[component]
pub fn IssueDetail(owner: String, repo: String, issue_number: u32) -> impl IntoView {
    let api = get_api();
    
    // Query for issue details
    let issue_query = use_query(
        move || &["issue", &owner, &repo, &issue_number.to_string()][..],
        move || {
            let api = api.clone();
            let owner = owner.clone();
            let repo = repo.clone();
            async move { api.get_issue(&owner, &repo, issue_number).await }
        },
        QueryOptions::default()
    );

    // Query for issue comments
    let comments_query = use_query(
        move || &["issue", &owner, &repo, &issue_number.to_string(), "comments"][..],
        move || {
            let api = api.clone();
            let owner = owner.clone();
            let repo = repo.clone();
            async move { api.get_issue_comments(&owner, &repo, issue_number).await }
        },
        QueryOptions::default()
            .enabled(issue_query.data().is_some()) // Only fetch comments if issue exists
    );

    view! {
        <div class="issue-detail">
            {move || match issue_query.data() {
                Some(Ok(issue)) => view! {
                    <div class="issue-content">
                        <div class="issue-header">
                            <h1 class="issue-title">{issue.title}</h1>
                            <div class="issue-meta">
                                <span class=format!("issue-state issue-{}", issue.state)>
                                    {issue.state.to_uppercase()}
                                </span>
                                <span class="issue-number">{"#"}{issue.number}</span>
                                <span class="issue-author">
                                    {"by "}{issue.user.login}
                                </span>
                                <span class="issue-date">
                                    {issue.created_at.format("%Y-%m-%d %H:%M").to_string()}
                                </span>
                            </div>
                        </div>
                        
                        <div class="issue-body">
                            {if let Some(body) = &issue.body {
                                view! { <p>{body}</p> }
                            } else {
                                view! { <p class="no-body">"No description provided"</p> }
                            }}
                        </div>

                        <div class="issue-labels">
                            <h4>"Labels"</h4>
                            <div class="labels-list">
                                {issue.labels.iter().map(|label| {
                                    let style = format!("background-color: #{};", label.color);
                                    view! {
                                        <span class="label" style=style>
                                            {label.name.clone()}
                                        </span>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>

                        <div class="issue-comments">
                            <h4>"Comments"</h4>
                            {move || match comments_query.data() {
                                Some(Ok(comments)) => view! {
                                    <div class="comments-list">
                                        {if comments.is_empty() {
                                            view! { <p class="no-comments">"No comments yet"</p> }
                                        } else {
                                            view! {
                                                <div>
                                                    {comments.iter().map(|comment| {
                                                        view! {
                                                            <div class="comment">
                                                                <div class="comment-header">
                                                                    <img 
                                                                        src=comment.user.avatar_url.clone()
                                                                        alt=comment.user.login.clone()
                                                                        class="comment-avatar"
                                                                    />
                                                                    <span class="comment-author">
                                                                        {comment.user.login}
                                                                    </span>
                                                                    <span class="comment-date">
                                                                        {comment.created_at.format("%Y-%m-%d %H:%M").to_string()}
                                                                    </span>
                                                                </div>
                                                                <div class="comment-body">
                                                                    {comment.body.clone()}
                                                                </div>
                                                            </div>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            }
                                        }}
                                    </div>
                                }.into_view(),
                                Some(Err(e)) => view! {
                                    <div class="error">
                                        <p>"Error loading comments: "{e}</p>
                                    </div>
                                }.into_view(),
                                None if comments_query.is_loading() => view! {
                                    <div class="loading">
                                        <p>"Loading comments..."</p>
                                    </div>
                                }.into_view(),
                                None => view! {
                                    <div class="loading">
                                        <p>"Waiting for issue data..."</p>
                                    </div>
                                }.into_view(),
                            }}
                        </div>
                    </div>
                }.into_view(),
                Some(Err(e)) => view! {
                    <div class="error">
                        <h2>"Error loading issue"</h2>
                        <p>{e}</p>
                    </div>
                }.into_view(),
                None if issue_query.is_loading() => view! {
                    <div class="loading">
                        <h2>"Loading issue..."</h2>
                    </div>
                }.into_view(),
                None => view! {
                    <div class="loading">
                        <h2>"Initializing..."</h2>
                    </div>
                }.into_view(),
            }}
        </div>
    }
}

// Repository search component
#[component]
pub fn RepositorySearch() -> impl IntoView {
    let (search_query, set_search_query) = create_signal(String::new());
    let (debounced_query, set_debounced_query) = create_signal(String::new());
    let api = get_api();

    // Debounce search query
    create_effect(move |_| {
        let query = search_query.get();
        if !query.is_empty() {
            let set_query = set_debounced_query.clone();
            gloo_timers::callback::Timeout::new(500, move || {
                set_query.set(query.clone());
            }).forget();
        }
    });

    // Search query
    let search_query_result = use_query(
        move || &["search", "repos", &debounced_query.get()][..],
        move || {
            let api = api.clone();
            let query = debounced_query.get();
            async move { api.search_repositories(&query).await }
        },
        QueryOptions::default()
            .enabled(!debounced_query.get().is_empty())
    );

    view! {
        <div class="search-section">
            <div class="search-input">
                <input
                    type="text"
                    placeholder="Search repositories..."
                    value=search_query
                    on:input=move |ev| set_search_query.set(event_target_value(&ev))
                />
            </div>

            {move || if !debounced_query.get().is_empty() {
                match search_query_result.data() {
                    Some(Ok(result)) => view! {
                        <div class="search-results">
                            <h3>"Search Results"</h3>
                            <p class="result-count">
                                {"Found "}{result.total_count}{" repositories"}
                            </p>
                            <div class="repos-grid">
                                {result.items.iter().map(|repo| {
                                    view! { <RepositoryCard repo=repo.clone()/> }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_view(),
                    Some(Err(e)) => view! {
                        <div class="error">
                            <p>"Search error: "{e}</p>
                        </div>
                    }.into_view(),
                    None if search_query_result.is_loading() => view! {
                        <div class="loading">
                            <p>"Searching..."</p>
                        </div>
                    }.into_view(),
                    None => view! {
                        <div class="loading">
                            <p>"Ready to search..."</p>
                        </div>
                    }.into_view(),
                }
            } else {
                view! {
                    <div class="search-placeholder">
                        <p>"Enter a repository name to search..."</p>
                    </div>
                }.into_view()
            }}
        </div>
    }
}

// Loading spinner component
#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="loading-spinner">
            <div class="spinner"></div>
            <p>"Loading..."</p>
        </div>
    }
}

// Error component
#[component]
pub fn ErrorMessage(message: String) -> impl IntoView {
    view! {
        <div class="error-message">
            <p>{message}</p>
        </div>
    }
}
