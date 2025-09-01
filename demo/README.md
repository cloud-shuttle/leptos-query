# Leptos Query Demo - GitHub Issues Viewer

A live demonstration of `leptos-query` showcasing real-world data fetching patterns, caching, and state management in a Leptos application.

## 🚀 Features Demonstrated

### Core leptos-query Features
- **Automatic Caching**: Queries are cached and shared across components
- **Background Updates**: Data stays fresh with automatic refetching
- **Query Dependencies**: Comments are only fetched after issue data loads
- **Debounced Search**: Search queries are debounced to prevent API spam
- **Error Handling**: Graceful error states and retry logic
- **Loading States**: Smooth loading indicators and skeleton states

### Real-world Patterns
- **GitHub API Integration**: Real API calls to GitHub's REST API
- **Repository Search**: Search and browse GitHub repositories
- **Issue Management**: View repository issues and comments
- **Responsive Design**: Mobile-friendly interface
- **Modern UI**: Clean, professional design with smooth animations

## 🛠️ Technical Stack

- **Framework**: Leptos 0.6
- **Data Fetching**: leptos-query (this library)
- **Routing**: leptos_router
- **Styling**: Modern CSS with Grid and Flexbox
- **HTTP Client**: reqwest
- **Build Tool**: Trunk
- **API**: GitHub REST API

## 🏃‍♂️ Quick Start

### Prerequisites
- Rust 1.70+
- Trunk (`cargo install trunk`)

### Running the Demo

1. **Navigate to the demo directory**:
   ```bash
   cd demo
   ```

2. **Install dependencies**:
   ```bash
   cargo build
   ```

3. **Start the development server**:
   ```bash
   trunk serve
   ```

4. **Open your browser**:
   Navigate to `http://localhost:8080`

## 📱 Demo Features

### Home Page
- **Feature showcase**: Highlights leptos-query capabilities
- **Repository search**: Search for any GitHub repository
- **Example repositories**: Quick links to popular repos

### Repository Page
- **Repository info**: Stars, forks, language, description
- **Issues list**: Recent open issues with labels and metadata
- **Real-time data**: Fresh data from GitHub API

### Issue Detail Page
- **Issue content**: Full issue details and description
- **Comments**: All comments with user avatars and timestamps
- **Navigation**: Easy navigation between pages

## 🔧 Development

### Project Structure
```
demo/
├── src/
│   ├── main.rs          # Application entry point
│   ├── app.rs           # Main app component and routing
│   ├── components.rs    # Reusable UI components
│   ├── api.rs           # GitHub API client
│   └── types.rs         # Data structures
├── styles.css           # Application styles
├── index.html           # HTML template
├── Trunk.toml           # Build configuration
└── Cargo.toml           # Dependencies
```

### Key Components

#### Query Patterns
```rust
// Basic query with caching
let repo_query = use_query(
    move || &["repo", &owner(), &repo()][..],
    move || async move { api.get_repository(&owner(), &repo()).await },
    QueryOptions::default()
);

// Dependent query (comments only load after issue)
let comments_query = use_query(
    move || &["issue", &owner(), &repo(), &issue_number.to_string(), "comments"][..],
    move || async move { api.get_issue_comments(&owner(), &repo(), issue_number).await },
    QueryOptions::default()
        .enabled(issue_query.data().is_some())
);

// Debounced search query
let search_query_result = use_query(
    move || &["search", "repos", &debounced_query.get()][..],
    move || async move { api.search_repositories(&debounced_query.get()).await },
    QueryOptions::default()
        .enabled(!debounced_query.get().is_empty())
);
```

#### Component Patterns
```rust
// Loading and error states
{move || match query.data() {
    Some(Ok(data)) => view! { /* Success UI */ },
    Some(Err(e)) => view! { /* Error UI */ },
    None if query.is_loading() => view! { /* Loading UI */ },
    None => view! { /* Initial state */ },
}}
```

### Styling
- **Modern CSS**: Grid layouts, Flexbox, CSS custom properties
- **Responsive design**: Mobile-first approach
- **Smooth animations**: Hover effects and transitions
- **Loading states**: Spinners and skeleton screens

## 🌐 Deployment

### Building for Production
```bash
trunk build --release
```

### Static Hosting
The demo can be deployed to any static hosting service:
- **Netlify**: Drag and drop the `dist` folder
- **Vercel**: Connect your repository
- **GitHub Pages**: Push to a GitHub repository
- **Cloudflare Pages**: Connect your repository

### Environment Variables
No environment variables are required for the demo. It uses GitHub's public API with rate limiting.

## 🔍 API Usage

The demo uses GitHub's public REST API:
- **Rate limiting**: 60 requests per hour for unauthenticated requests
- **Endpoints used**:
  - `GET /repos/{owner}/{repo}` - Repository information
  - `GET /repos/{owner}/{repo}/issues` - Repository issues
  - `GET /repos/{owner}/{repo}/issues/{number}` - Specific issue
  - `GET /repos/{owner}/{repo}/issues/{number}/comments` - Issue comments
  - `GET /search/repositories` - Repository search

## 🎯 Learning Objectives

This demo demonstrates:

1. **Query Management**: How to structure and organize queries
2. **Caching Strategy**: Automatic caching and invalidation
3. **Error Handling**: Graceful error states and user feedback
4. **Performance**: Debouncing, dependent queries, and loading states
5. **Real-world Integration**: Working with external APIs
6. **Component Architecture**: Reusable components and patterns

## 🤝 Contributing

Want to improve the demo? Here are some ideas:

- **Add more features**: Pull requests, user profiles, etc.
- **Improve styling**: Better animations, themes, etc.
- **Add tests**: Unit tests for components and API functions
- **Performance optimizations**: Virtual scrolling, pagination, etc.
- **Accessibility**: ARIA labels, keyboard navigation, etc.

## 📚 Resources

- [leptos-query Documentation](../README.md)
- [Leptos Framework](https://leptos.dev/)
- [GitHub API Documentation](https://docs.github.com/en/rest)
- [Trunk Build Tool](https://trunkrs.dev/)

---

**Built with ❤️ using leptos-query**
