# Ecosystem Integration

This document outlines how `leptos-query` integrates with the broader Rust and Leptos ecosystem.

## Leptos Ecosystem

### Compatible Leptos Versions

`leptos-query` is designed to work seamlessly with both Leptos 0.6 and 0.8:

```toml
# For Leptos 0.6
[dependencies]
leptos-query = { version = "0.1.0", features = ["leptos-0-6"] }

# For Leptos 0.8
[dependencies]
leptos-query = { version = "0.1.0", features = ["leptos-0-8"] }
```

### Integration with Other Leptos Libraries

#### Leptos Router
```rust
use leptos::*;
use leptos_router::*;
use leptos_query::*;

#[component]
fn UserPage() -> impl IntoView {
    let params = use_params::<UserParams>();
    let user_id = move || params.get().map(|p| p.id).unwrap_or(0);
    
    let user_query = use_query(
        move || &["users", &user_id().to_string()][..],
        move || || async move { fetch_user(user_id()).await },
        QueryOptions::default()
    );
    
    // Component implementation...
}
```

#### Leptos Meta
```rust
use leptos::*;
use leptos_meta::*;
use leptos_query::*;

#[component]
fn UserProfile(user_id: u32) -> impl IntoView {
    let user_query = use_query(
        move || &["users", &user_id.to_string()][..],
        move || || async move { fetch_user(user_id).await },
        QueryOptions::default()
    );
    
    view! {
        <Title text=move || {
            user_query.data()
                .and_then(|r| r.ok())
                .map(|user| format!("{} - Profile", user.name))
                .unwrap_or_else(|| "User Profile".to_string())
        }/>
        // Rest of component...
    }
}
```

## HTTP Client Integration

### Reqwest
```rust
use leptos::*;
use leptos_query::*;
use reqwest;

async fn fetch_user_reqwest(id: u32) -> Result<User, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("https://api.example.com/users/{}", id))
        .send()
        .await?;
    
    let user: User = response.json().await?;
    Ok(user)
}

#[component]
fn UserComponent(user_id: u32) -> impl IntoView {
    let user_query = use_query(
        move || &["users", &user_id.to_string()][..],
        move || || async move { fetch_user_reqwest(user_id).await },
        QueryOptions::default()
    );
    
    // Component implementation...
}
```

### Axum (Server-side)
```rust
use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use leptos_query::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, StatusCode> {
    // Your database query logic here
    let user = User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    };
    
    Ok(Json(user))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/users/:id", get(get_user));
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## State Management Integration

### With Leptos Signals
```rust
use leptos::*;
use leptos_query::*;

#[component]
fn UserDashboard() -> impl IntoView {
    let (selected_user_id, set_selected_user_id) = create_signal(1u32);
    
    let user_query = use_query(
        move || &["users", &selected_user_id.get().to_string()][..],
        move || || async move { fetch_user(selected_user_id.get()).await },
        QueryOptions::default()
    );
    
    view! {
        <div>
            <select on:change=move |ev| {
                let value = event_target_value(&ev).parse::<u32>().unwrap_or(1);
                set_selected_user_id.set(value);
            }>
                <option value="1">"User 1"</option>
                <option value="2">"User 2"</option>
                <option value="3">"User 3"</option>
            </select>
            
            // User data display...
        </div>
    }
}
```

### With Global State
```rust
use leptos::*;
use leptos_query::*;
use std::collections::HashMap;

#[derive(Clone)]
struct AppState {
    users: HashMap<u32, User>,
}

#[component]
fn App() -> impl IntoView {
    provide_context(AppState {
        users: HashMap::new(),
    });
    
    view! {
        <div>
            <UserList/>
            <UserDetail/>
        </div>
    }
}

#[component]
fn UserList() -> impl IntoView {
    let users_query = use_query(
        || &["users"][..],
        || || async move { fetch_all_users().await },
        QueryOptions::default()
    );
    
    // Component implementation...
}
```

## Testing Integration

### With Leptos Testing
```rust
use leptos::*;
use leptos_query::*;
use leptos_testing::*;

#[test]
fn test_user_query() {
    let app = create_runtime();
    
    let (cx, _) = app.run_scope(|cx| {
        let user_query = use_query(
            || &["users", "1"][..],
            || || async move { Ok(User { id: 1, name: "Test User".to_string() }) },
            QueryOptions::default()
        );
        
        // Test query state
        assert!(user_query.is_loading());
        
        cx
    });
    
    // Wait for query to complete
    app.dispose();
}
```

### Mock API Testing
```rust
use leptos::*;
use leptos_query::*;
use mockall::*;

mock! {
    UserService {}
    
    impl UserService {
        async fn fetch_user(&self, id: u32) -> Result<User, String>;
    }
}

#[test]
fn test_with_mock_service() {
    let mut mock_service = MockUserService::new();
    mock_service
        .expect_fetch_user()
        .times(1)
        .returning(|id| Ok(User { id, name: "Mock User".to_string() }));
    
    // Test implementation...
}
```

## Performance Monitoring

### Query Performance Tracking
```rust
use leptos::*;
use leptos_query::*;
use std::time::Instant;

async fn fetch_user_with_timing(id: u32) -> Result<User, String> {
    let start = Instant::now();
    let result = fetch_user(id).await;
    let duration = start.elapsed();
    
    // Log or send metrics
    log::info!("User fetch took {:?} for user {}", duration, id);
    
    result
}

#[component]
fn MonitoredUserComponent(user_id: u32) -> impl IntoView {
    let user_query = use_query(
        move || &["users", &user_id.to_string()][..],
        move || || async move { fetch_user_with_timing(user_id).await },
        QueryOptions::default()
    );
    
    // Component implementation...
}
```

## Community Resources

### Related Projects
- [Leptos](https://github.com/leptos-rs/leptos) - The main Leptos framework
- [Leptos Router](https://github.com/leptos-rs/leptos_router) - Routing for Leptos
- [Leptos Meta](https://github.com/leptos-rs/leptos_meta) - Document head management
- [Leptos Testing](https://github.com/leptos-rs/leptos_testing) - Testing utilities

### Useful Tools
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) - Build tool for Leptos
- [wasm-pack](https://github.com/rustwasm/wasm-pack) - WebAssembly packaging
- [trunk](https://github.com/thedodd/trunk) - Web application bundler

### Community Channels
- [Leptos Discord](https://discord.gg/leptos)
- [Rust Community Discord](https://discord.gg/rust)
- [Reddit r/rust](https://reddit.com/r/rust)

## Contributing to the Ecosystem

We welcome contributions to improve ecosystem integration:

1. **Report Issues**: Found a compatibility issue? Open an issue!
2. **Submit PRs**: Have an improvement? Submit a pull request!
3. **Share Examples**: Built something cool? Share it with the community!
4. **Documentation**: Help improve our documentation and examples!

For more information, see our [Contributing Guide](../CONTRIBUTING.md).
