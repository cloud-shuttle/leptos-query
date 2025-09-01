//! Signal compatibility layer for Leptos 0.6 and 0.8

#[cfg(feature = "leptos-0-6")]
use leptos::*;

#[cfg(feature = "leptos-0-8")]
use leptos_0_8::*;

/// Unified signal types that work with both Leptos 0.6 and 0.8
pub type CompatSignal<T> = Signal<T>;
pub type CompatReadSignal<T> = ReadSignal<T>;
pub type CompatWriteSignal<T> = WriteSignal<T>;

/// Create a signal that works with both Leptos versions
pub fn create_compat_signal<T>(initial: T) -> (CompatReadSignal<T>, CompatWriteSignal<T>)
where
    T: Clone + 'static,
{
    create_signal(initial)
}

/// Create a memo that works with both Leptos versions
pub fn create_compat_memo<T, F>(f: F) -> CompatReadSignal<T>
where
    F: Fn() -> T + 'static,
    T: Clone + 'static,
{
    create_memo(f)
}

/// Create an effect that works with both Leptos versions
pub fn create_compat_effect<F>(f: F)
where
    F: Fn() + 'static,
{
    create_effect(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compat_signal_creation() {
        let (read, write) = create_compat_signal(42);
        assert_eq!(read.get(), 42);
        
        write.set(100);
        assert_eq!(read.get(), 100);
    }

    #[test]
    fn test_compat_memo() {
        let (count, set_count) = create_compat_signal(5);
        let doubled = create_compat_memo(move || count.get() * 2);
        
        assert_eq!(doubled.get(), 10);
        
        set_count.set(10);
        assert_eq!(doubled.get(), 20);
    }
}
