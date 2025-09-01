# Migration Guide

Coming from React Query/TanStack Query? This guide will help you migrate your existing code to Leptos Query.

## Key Differences

| React Query | Leptos Query | Notes |
|-------------|--------------|-------|
| `useQuery` | `use_query` | Function naming follows Rust conventions |
| `useMutation` | `use_mutation` | Same functionality, different syntax |
| `queryKey` | `key_fn` | Function that returns the key |
| `queryFn` | `query_fn` | Function that returns the actual query function |
| `enabled` | `enabled` | Same concept, different type |
| `staleTime` | `stale_time` | Same concept, different naming |
| `cacheTime` | `cache_time` | Same concept, different naming |
| `refetchInterval` | `refetch_interval` | Same concept, different naming |
| `retry` | `retry` | Same concept, different configuration |

## Basic Query Migration

### React Query (JavaScript/TypeScript)

```javascript
import { useQuery } from '@tanstack/react-query';

function UserProfile({ userId }) {
  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ['users', userId],
    queryFn: () => fetchUser(userId),
    staleTime: 60000,
    cacheTime: 300000,
  });

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;
  if (!data) return <div>No data</div>;

  return (
    <div>
      <h3>{data.name}</h3>
      <p>Email: {data.email}</p>
      <button onClick={() => refetch()}>Refresh</button>
    </div>
  );
}
```

### Leptos Query (Rust)

```rust
use leptos::*;
use leptos_query::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

async fn fetch_user(id: u32) -> Result<User, QueryError> {
    // Your API call here
    Ok(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

#[component]
fn UserProfile(user_id: u32) -> impl IntoView {
    let user_query = use_query(
        move || &["users", &user_id.to_string()][..],
        move || || async move { fetch_user(user_id).await },
        QueryOptions::default()
            .with_stale_time(Duration::from_secs(60))
            .with_cache_time(Duration::from_secs(300))
    );

    view! {
        <div>
            {move || {
                if user_query.is_loading.get() {
                    view! { <div>"Loading..."</div> }
                } else if let Some(error) = user_query.error.get() {
                    view! { <div>"Error: " {error.to_string()}</div> }
                } else if let Some(user) = user_query.data.get() {
                    view! {
                        <div>
                            <h3>{user.name}</h3>
                            <p>"Email: " {user.email}</p>
                        </div>
                    }
                } else {
                    view! { <div>"No data"</div> }
                }
            }}
            <button on:click=move |_| user_query.refetch.call(())>
                "Refresh"
            </button>
        </div>
    }
}
```

## Mutation Migration

### React Query (JavaScript/TypeScript)

```javascript
import { useMutation, useQueryClient } from '@tanstack/react-query';

function CreateUserForm() {
  const queryClient = useQueryClient();
  
  const createUserMutation = useMutation({
    mutationFn: createUser,
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
      console.log('User created:', data);
    },
    onError: (error) => {
      console.error('Failed to create user:', error);
    },
  });

  const handleSubmit = (formData) => {
    createUserMutation.mutate(formData);
  };

  return (
    <form onSubmit={handleSubmit}>
      <input name="name" placeholder="Name" />
      <input name="email" placeholder="Email" />
      <button type="submit" disabled={createUserMutation.isPending}>
        {createUserMutation.isPending ? 'Creating...' : 'Create User'}
      </button>
    </form>
  );
}
```

### Leptos Query (Rust)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

async fn create_user(request: CreateUserRequest) -> Result<User, QueryError> {
    // Your API call here
    Ok(User {
        id: 999,
        name: request.name,
        email: request.email,
    })
}

#[component]
fn CreateUserForm() -> impl IntoView {
    let create_user_mutation = use_mutation::<User, CreateUserRequest, (), _, _>(
        |request| async move { create_user(request).await },
        MutationOptions::default()
            .with_invalidates(&[&["users"][..]])
            .with_on_success(Box::new(|data, _vars, _ctx| {
                log::info!("User created: {:?}", data);
            }))
            .with_on_error(Box::new(|error, _vars, _ctx| {
                log::error!("Failed to create user: {:?}", error);
            }))
    );

    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());

    let handle_submit = move |_| {
        let request = CreateUserRequest {
            name: name.get(),
            email: email.get(),
        };
        create_user_mutation.mutate.call(request);
    };

    view! {
        <form on:submit=handle_submit>
            <input
                placeholder="Name"
                on:input=move |ev| set_name.set(event_target_value(&ev))
            />
            <input
                placeholder="Email"
                on:input=move |ev| set_email.set(event_target_value(&ev))
            />
            <button type="submit" disabled=move || create_user_mutation.is_loading.get()>
                {move || if create_user_mutation.is_loading.get() { "Creating..." } else { "Create User" }}
            </button>
        </form>
    }
}
```

## Query Key Patterns

### React Query (JavaScript/TypeScript)

```javascript
// Simple key
['users']

// Key with parameters
['users', userId]

// Complex key
['users', userId, 'profile', { includePosts: true }]

// Invalidation patterns
queryClient.invalidateQueries({ queryKey: ['users'] });
queryClient.invalidateQueries({ queryKey: ['users'], exact: true });
queryClient.invalidateQueries({ 
  predicate: (query) => query.queryKey[0] === 'users' 
});
```

### Leptos Query (Rust)

```rust
// Simple key
|| &["users"][..]

// Key with parameters
move || &["users", &user_id.to_string()][..]

// Complex key
move || {
    let include_posts = true;
    &["users", &user_id.to_string(), "profile", &include_posts.to_string()][..]
}

// Invalidation patterns
client.invalidate_queries(&QueryKeyPattern::Prefix(QueryKey::new(&["users"])));
client.invalidate_queries(&QueryKeyPattern::Exact(QueryKey::new(&["users"])));
client.invalidate_queries(&QueryKeyPattern::Contains("users".to_string()));
```

## Configuration Migration

### React Query (JavaScript/TypeScript)

```javascript
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60000,
      cacheTime: 300000,
      retry: 3,
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
    },
    mutations: {
      retry: 1,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <YourApp />
    </QueryClientProvider>
  );
}
```

### Leptos Query (Rust)

```rust
use leptos::*;
use leptos_query::*;

#[component]
fn App() -> impl IntoView {
    let config = QueryClientConfig::default()
        .with_default_stale_time(Duration::from_secs(60))
        .with_default_cache_time(Duration::from_secs(300))
        .with_default_retry(RetryConfig {
            max_attempts: 3,
            delay: RetryDelay::Exponential {
                initial: Duration::from_millis(1000),
                multiplier: 2.0,
                max: Duration::from_secs(30),
            },
            jitter: false,
        });

    view! {
        <QueryClientProvider config>
            <YourApp />
        </QueryClientProvider>
    }
}
```

## Error Handling Migration

### React Query (JavaScript/TypeScript)

```javascript
const { data, error, isLoading } = useQuery({
  queryKey: ['users', userId],
  queryFn: fetchUser,
});

if (error) {
  if (error.name === 'NetworkError') {
    return <div>Network error occurred</div>;
  }
  if (error.response?.status === 404) {
    return <div>User not found</div>;
  }
  return <div>Error: {error.message}</div>;
}
```

### Leptos Query (Rust)

```rust
let user_query = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
);

{move || {
    if let Some(error) = user_query.error.get() {
        match error {
            QueryError::Network { message } => {
                view! { <div>"Network error occurred"</div> }
            }
            QueryError::Http { status: 404, .. } => {
                view! { <div>"User not found"</div> }
            }
            _ => {
                view! { <div>"Error: " {error.to_string()}</div> }
            }
        }
    } else {
        view! { <div></div> }
    }
}}
```

## Optimistic Updates Migration

### React Query (JavaScript/TypeScript)

```javascript
const updateUserMutation = useMutation({
  mutationFn: updateUser,
  onMutate: async (newUser) => {
    // Cancel outgoing refetches
    await queryClient.cancelQueries({ queryKey: ['users', newUser.id] });
    
    // Snapshot previous value
    const previousUser = queryClient.getQueryData(['users', newUser.id]);
    
    // Optimistically update
    queryClient.setQueryData(['users', newUser.id], newUser);
    
    return { previousUser };
  },
  onError: (err, newUser, context) => {
    // Rollback on error
    queryClient.setQueryData(['users', newUser.id], context.previousUser);
  },
  onSettled: () => {
    // Always refetch after error or success
    queryClient.invalidateQueries({ queryKey: ['users'] });
  },
});
```

### Leptos Query (Rust)

```rust
let optimistic_mutation = use_optimistic_mutation(
    QueryKey::new(&["users", &user_id.to_string()]),
    |request| async move { update_user(request).await },
    |request| User {
        id: request.id,
        name: request.name.clone(),
        email: request.email.clone(),
    }
);
```

## Common Migration Patterns

### 1. Conditional Queries

**React Query:**
```javascript
const { data } = useQuery({
  queryKey: ['users', userId],
  queryFn: () => fetchUser(userId),
  enabled: !!userId,
});
```

**Leptos Query:**
```rust
let user_query = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
        .with_enabled(Signal::derive(move || user_id.get() > 0))
);
```

### 2. Dependent Queries

**React Query:**
```javascript
const { data: user } = useQuery({
  queryKey: ['users', userId],
  queryFn: () => fetchUser(userId),
});

const { data: posts } = useQuery({
  queryKey: ['users', userId, 'posts'],
  queryFn: () => fetchUserPosts(userId),
  enabled: !!user,
});
```

**Leptos Query:**
```rust
let user_query = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
);

let posts_query = use_query(
    move || &["users", &user_id.to_string(), "posts"][..],
    move || || async move { fetch_user_posts(user_id).await },
    QueryOptions::default()
        .with_enabled(Signal::derive(move || user_query.data.get().is_some()))
);
```

### 3. Infinite Queries

**Note:** Infinite queries are not yet implemented in Leptos Query, but are planned for future releases.

## Tips for Migration

1. **Start Simple**: Begin with basic queries and mutations before moving to complex patterns
2. **Use Type Annotations**: Explicitly specify generic types to help with compilation
3. **Error Handling**: Leptos Query has more structured error types - take advantage of them
4. **Reactive Patterns**: Embrace Leptos's reactive patterns for better performance
5. **Testing**: Use the provided test utilities to ensure your queries work correctly

## Need Help?

- Check the [API Reference](./api-reference.md) for complete documentation
- See [Examples](../examples/) for real-world implementations
- Join the [Leptos Discord](https://discord.gg/leptos) for community support

Happy migrating! 🚀
