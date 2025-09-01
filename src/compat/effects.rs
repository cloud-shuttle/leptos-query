//! Effects compatibility layer for Leptos 0.6 and 0.8

#[cfg(feature = "leptos-0-6")]
use leptos::*;

#[cfg(feature = "leptos-0-8")]
use leptos_0_8::*;

/// Create an effect that works with both Leptos versions
pub fn create_compat_effect<F>(f: F)
where
    F: Fn() + 'static,
{
    create_effect(f)
}

/// Create a memo that works with both Leptos versions
pub fn create_compat_memo<T, F>(f: F) -> ReadSignal<T>
where
    F: Fn() -> T + 'static,
    T: Clone + 'static,
{
    create_memo(f)
}

/// Create a derived signal that works with both Leptos versions
pub fn create_compat_derived<T, F>(f: F) -> ReadSignal<T>
where
    F: Fn() -> T + 'static,
    T: Clone + 'static,
{
    create_memo(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compat_effect() {
        let (count, set_count) = create_signal(0);
        let effect_called = std::rc::Rc::new(std::cell::RefCell::new(false));
        let effect_called_clone = effect_called.clone();
        
        create_compat_effect(move || {
            let _ = count.get(); // Access the signal
            *effect_called_clone.borrow_mut() = true;
        });
        
        // Effect should be called when signal changes
        set_count.set(1);
        assert!(*effect_called.borrow());
    }

    #[test]
    fn test_compat_memo() {
        let (input, set_input) = create_signal(5);
        let computed = create_compat_memo(move || input.get() * 2);
        
        assert_eq!(computed.get(), 10);
        
        set_input.set(10);
        assert_eq!(computed.get(), 20);
    }
}
