# Getting Started with Leptos Query

Welcome to Leptos Query! This guide will help you get up and running with powerful data fetching and caching for your Leptos applications.

## Installation

Add Leptos Query to your `Cargo.toml`:

```toml
[dependencies]
leptos = "0.6"
leptos-query = "0.1"

# For HTTP requests (recommended)
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
```

## Basic Setup

### 1. Initialize the Query Client

First, create and provide a `QueryClient` to your application:

```rust
use leptos::*;
use leptos_query::*;

#[component]
fn App() -> impl IntoView {
    // Create a query client with default configuration
    let client = QueryClient::new(QueryClientConfig::default());
    provide_context(client);
    
    view! {
        <div class="app">
            <h1>"My App"</h1>
            <UserList />
        </div>
    }
}
```

### 2. Your First Query

Let's create a simple query to fetch user data:

```rust
#[component]
fn UserList() -> impl IntoView {
    // Define the query
    let users = use_query(
        || ["users"],                    // Query key
        || async {                       // Query function
            fetch_users().await
        },
        QueryOptions::default()          // Configuration
    );
    
    view! {
        <div>
            <h2>"Users"</h2>
            
            // Show loading state
            <Show 
                when=move || users.is_loading.get()
                fallback=move || view! {
                    // Show error or data
                    <Show
                        when=move || users.is_error.get()
                        fallback=move || {
                            // Render the data
                            match users.data.get() {
                                Some(user_list) => view! {
                                    <ul>
                                        {user_list.into_iter()
                                            .map(|user| view! {
                                                <li key={user.id}>
                                                    {user.name} " (" {user.email} ")"
                                                </li>
                                            })
                                            .collect::<Vec<_>>()
                                        }
                                    </ul>
                                }.into_view(),
                                None => view! { <p>"No users found"</p> }.into_view()
                            }
                        }
                    >
                        // Error state
                        <div class="error">
                            <p>"Failed to load users: " {users.error.get().unwrap().to_string()}</p>
                            <button on:click=move |_| users.refetch.call(())>
                                "Retry"
                            </button>
                        </div>
                    </Show>
                }
            >
                // Loading state
                <div class="loading">
                    <p>"Loading users..."</p>
                </div>
            </Show>
            
            // Refetch button
            <button 
                on:click=move |_| users.refetch.call(())
                disabled=move || users.is_fetching.get()
            >
                {move || if users.is_fetching.get() { "Refreshing..." } else { "Refresh" }}
            </button>
        </div>
    }
}

// API function
async fn fetch_users() -> Result<Vec<User>, QueryError> {
    let response = reqwest::get("https://jsonplaceholder.typicode.com/users")
        .await
        .map_err(|e| QueryError::network(e.to_string()))?;
        
    let users = response.json::<Vec<User>>()
        .await
        .map_err(|e| QueryError::deserialization(e.to_string()))?;
        
    Ok(users)
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}
```

## Understanding Query Keys

Query keys uniquely identify your queries and are used for caching, invalidation, and deduplication.

### Static Keys

```rust
// Simple static key
let todos = use_query(
    || ["todos"],
    || fetch_todos(),
    QueryOptions::default()
);
```

### Dynamic Keys

```rust
// Key that changes with reactive state
let user_id = create_rw_signal(1u32);

let user = use_query(
    move || ["users", user_id.get().to_string()],
    move || fetch_user(user_id.get()),
    QueryOptions::default()
);
```

### Hierarchical Keys

```rust
// Nested keys for related data
let user_posts = use_query(
    move || ["users", user_id.get().to_string(), "posts"],
    move || fetch_user_posts(user_id.get()),
    QueryOptions::default()
);
```

## Query Configuration

Customize query behavior with `QueryOptions`:

```rust
let posts = use_query(
    move || ["posts", page.get().to_string()],
    move || fetch_posts(page.get()),
    QueryOptions {
        // Data becomes stale after 5 minutes
        stale_time: Duration::from_secs(5 * 60),
        
        // Keep data in cache for 10 minutes
        cache_time: Duration::from_secs(10 * 60),
        
        // Refetch every 30 seconds in the background
        refetch_interval: Some(Duration::from_secs(30)),
        
        // Refetch when the window regains focus
        refetch_on_window_focus: true,
        
        // Refetch when the network reconnects
        refetch_on_reconnect: true,
        
        // Show previous data while fetching new data
        keep_previous_data: true,
        
        ..Default::default()
    }
);
```

## Working with Loading States

Leptos Query provides several loading state indicators:

```rust
let data = use_query(/* ... */);

view! {
    <div>
        // Initial loading (no data yet)
        {move || if data.is_loading.get() {
            view! { <div>"Loading for the first time..."</div> }
        } else {
            view! { <div></div> }
        }}
        
        // Background fetching (has data, but refetching)
        {move || if data.is_fetching.get() && !data.is_loading.get() {
            view! { <div class="fetching-indicator">"Updating..."</div> }
        } else {
            view! { <div></div> }
        }}
        
        // Success state
        {move || if data.is_success.get() {
            view! {
                <div>"Data loaded successfully!"</div>
            }
        } else {
            view! { <div></div> }
        }}
        
        // Error state
        {move || if data.is_error.get() {
            view! {
                <div class="error">
                    "Error: " {data.error.get().unwrap().to_string()}
                </div>
            }
        } else {
            view! { <div></div> }
        }}
    </div>
}
```

## Error Handling

Handle different types of errors appropriately:

```rust
let query = use_query(/* ... */);

view! {
    <div>
        {move || match query.error.get() {
            Some(QueryError::Network { message, .. }) => view! {
                <div class="error network-error">
                    <p>"Network Error: " {message}</p>
                    <p>"Please check your internet connection."</p>
                    <button on:click=move |_| query.refetch.call(())>
                        "Retry"
                    </button>
                </div>
            }.into_view(),
            
            Some(QueryError::Http { status, message, .. }) => {
                match status {
                    404 => view! {
                        <div class="error not-found">
                            <p>"Resource not found"</p>
                        </div>
                    }.into_view(),
                    401 | 403 => view! {
                        <div class="error auth-error">
                            <p>"Authentication required"</p>
                            <button on:click=|_| redirect_to_login()>
                                "Login"
                            </button>
                        </div>
                    }.into_view(),
                    500..=599 => view! {
                        <div class="error server-error">
                            <p>"Server Error: " {message}</p>
                            <button on:click=move |_| query.refetch.call(())>
                                "Retry"
                            </button>
                        </div>
                    }.into_view(),
                    _ => view! {
                        <div class="error">
                            <p>"HTTP " {status.to_string()} ": " {message}</p>
                        </div>
                    }.into_view(),
                }
            },
            
            Some(QueryError::Timeout { timeout_ms }) => view! {
                <div class="error timeout-error">
                    <p>"Request timed out after " {timeout_ms.to_string()} "ms"</p>
                    <button on:click=move |_| query.refetch.call(())>
                        "Retry"
                    </button>
                </div>
            }.into_view(),
            
            Some(error) => view! {
                <div class="error">
                    <p>"Unexpected error: " {error.to_string()}</p>
                    <button on:click=move |_| query.refetch.call(())>
                        "Retry"
                    </button>
                </div>
            }.into_view(),
            
            None => view! { <div></div> }.into_view(),
        }}
    </div>
}

fn redirect_to_login() {
    // Navigate to login page
    window().location().set_href("/login").unwrap();
}
```

## Conditional Queries

Sometimes you want to conditionally execute queries:

```rust
let user_id = create_rw_signal(None::<u32>);

let user = use_query(
    move || ["users", user_id.get().unwrap_or(0).to_string()],
    move || async move {
        if let Some(id) = user_id.get() {
            fetch_user(id).await
        } else {
            Err(QueryError::custom("No user ID provided"))
        }
    },
    QueryOptions {
        // Only run the query when we have a user ID
        enabled: Signal::derive(move || user_id.get().is_some()),
        ..Default::default()
    }
);
```

## Dependent Queries

Create queries that depend on data from other queries:

```rust
// First query: get user
let user = use_query(
    move || ["users", user_id.get().to_string()],
    move || fetch_user(user_id.get()),
    QueryOptions::default()
);

// Second query: get user's posts (depends on user data)
let user_posts = use_query(
    move || {
        if let Some(user_data) = user.data.get() {
            ["users", user_data.id.to_string(), "posts"]
        } else {
            ["disabled"] // Disabled key
        }
    },
    move || async move {
        if let Some(user_data) = user.data.get() {
            fetch_user_posts(user_data.id).await
        } else {
            Err(QueryError::custom("No user data available"))
        }
    },
    QueryOptions {
        // Only run when we have user data
        enabled: Signal::derive(move || user.data.get().is_some()),
        ..Default::default()
    }
);
```

## Mutations

Mutations handle data modifications (POST, PUT, DELETE requests):

```rust
#[component]
fn CreateUserForm() -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    
    let create_user = use_mutation(
        |user_data: CreateUserDto| async move {
            create_user_api(user_data).await
        },
        MutationOptions {
            invalidates: vec![
                QueryKeyPattern::Prefix(QueryKey::new(["users"])),
            ],
            on_success: Some(Box::new(|_data, _vars, _ctx| {
                show_success_toast("User created successfully!");
            })),
            on_error: Some(Box::new(|error, _vars, _ctx| {
                show_error_toast(&format!("Failed to create user: {}", error));
            })),
            ..Default::default()
        },
    );
    
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        create_user.mutate.call(CreateUserDto {
            name: name.get(),
            email: email.get(),
        });
        
        // Clear form
        set_name.set(String::new());
        set_email.set(String::new());
    };
    
    view! {
        <form on:submit=on_submit>
            <div>
                <label>"Name:"</label>
                <input
                    type="text"
                    value=name
                    on:input=move |ev| set_name.set(event_target_value(&ev))
                    disabled=move || create_user.is_loading.get()
                />
            </div>
            <div>
                <label>"Email:"</label>
                <input
                    type="email"
                    value=email
                    on:input=move |ev| set_email.set(event_target_value(&ev))
                    disabled=move || create_user.is_loading.get()
                />
            </div>
            <button 
                type="submit"
                disabled=move || create_user.is_loading.get()
            >
                {move || if create_user.is_loading.get() {
                    "Creating..."
                } else {
                    "Create User"
                }}
            </button>
        </form>
    }
}

#[derive(Clone, serde::Serialize)]
struct CreateUserDto {
    name: String,
    email: String,
}

async fn create_user_api(user: CreateUserDto) -> Result<User, QueryError> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://jsonplaceholder.typicode.com/users")
        .json(&user)
        .send()
        .await
        .map_err(|e| QueryError::network(e.to_string()))?;
        
    response.json::<User>()
        .await
        .map_err(|e| QueryError::deserialization(e.to_string()))
}

fn show_success_toast(message: &str) {
    // Your toast implementation
    log::info!("Success: {}", message);
}

fn show_error_toast(message: &str) {
    // Your toast implementation
    log::error!("Error: {}", message);
}
```

## Next Steps

Now that you understand the basics, explore these advanced features:

1. **[Optimistic Updates](./optimistic-updates.md)** - Instant UI feedback
2. **[Infinite Queries](./infinite-queries.md)** - Pagination and infinite scrolling
3. **[Cache Management](./cache-management.md)** - Advanced caching strategies
4. **[Error Handling](./error-handling.md)** - Comprehensive error management
5. **[DevTools](./devtools.md)** - Debugging and development tools
6. **[SSR & Hydration](./ssr.md)** - Server-side rendering
7. **[Offline Support](./offline.md)** - Working without internet

## Common Patterns

### Global Loading State

```rust
#[component]
fn App() -> impl IntoView {
    let client = use_context::<QueryClient>().unwrap();
    
    // You could track global loading state here
    view! {
        <div>
            <Router>
                <Routes>
                    // Your routes
                </Routes>
            </Router>
        </div>
    }
}
```

### Authentication Integration

```rust
fn fetch_with_auth<T>(url: &str) -> impl Future<Output = Result<T, QueryError>>
where
    T: serde::de::DeserializeOwned,
{
    let url = url.to_string();
    async move {
        let token = get_auth_token().ok_or_else(|| {
            QueryError::custom("No authentication token")
        })?;
        
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| QueryError::network(e.to_string()))?;
            
        if response.status() == 401 {
            // Handle token expiry
            clear_auth_token();
            redirect_to_login();
            return Err(QueryError::http(401, "Token expired"));
        }
        
        response.json::<T>()
            .await
            .map_err(|e| QueryError::deserialization(e.to_string()))
    }
}
```

Happy querying! 🚀