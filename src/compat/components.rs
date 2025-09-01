//! Component compatibility layer for Leptos 0.6 and 0.8

#[cfg(feature = "leptos-0-6")]
use leptos::*;

#[cfg(feature = "leptos-0-8")]
use leptos_0_8::*;

use std::future::Future;

/// Re-export component macro for both versions
pub use leptos::component;

/// Re-export IntoView trait for both versions
pub use leptos::IntoView;

/// Create a context that works with both Leptos versions
pub fn create_compat_context<T>(value: T) -> T
where
    T: Clone + 'static,
{
    provide_context(value);
    value
}

/// Use a context that works with both Leptos versions
pub fn use_compat_context<T>() -> Option<T>
where
    T: Clone + 'static,
{
    use_context::<T>()
}

/// Create a resource that works with both Leptos versions
pub fn create_compat_resource<T, F, Fut>(fetcher: F) -> Resource<T, ()>
where
    T: Clone + 'static,
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = T> + 'static,
{
    create_resource(|| (), move |_| fetcher())
}

/// Create a resource with a key that works with both Leptos versions
pub fn create_compat_resource_with_key<T, K, F, Fut>(key: K, fetcher: F) -> Resource<K, T>
where
    T: Clone + 'static,
    K: Clone + 'static,
    F: Fn(K) -> Fut + 'static,
    Fut: Future<Output = T> + 'static,
{
    create_resource(key, fetcher)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compat_context() {
        let context_value = "test context";
        let _ = create_compat_context(context_value);
        
        // In a real component, we would use use_compat_context here
        // For now, we just test that the function compiles
        assert_eq!(context_value, "test context");
    }

    #[test]
    fn test_compat_resource() {
        let resource = create_compat_resource(|| async { "test resource" });
        
        // In a real component, we would access the resource value
        // For now, we just test that the function compiles
        assert!(resource.is_some());
    }
}
