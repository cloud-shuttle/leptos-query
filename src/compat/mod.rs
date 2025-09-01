//! Compatibility layer for different Leptos versions
//! 
//! This module provides a unified API that works across different versions
//! of Leptos, allowing users to upgrade their Leptos version without
//! breaking changes to their leptos-query code.

/// Version information for the compatibility layer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeptosVersion {
    V0_6,
    V0_8,
}

impl LeptosVersion {
    /// Get the current Leptos version being used
    pub fn current() -> Self {
        #[cfg(feature = "leptos-0-8")]
        {
            LeptosVersion::V0_8
        }
        #[cfg(not(feature = "leptos-0-8"))]
        {
            // Default to 0.6 if no version is specified
            LeptosVersion::V0_6
        }
    }
    
    /// Check if the current version is 0.8 or later
    pub fn is_0_8_or_later(&self) -> bool {
        matches!(self, LeptosVersion::V0_8)
    }
    
    /// Get the version string
    pub fn as_str(&self) -> &'static str {
        match self {
            LeptosVersion::V0_6 => "0.6",
            LeptosVersion::V0_8 => "0.8",
        }
    }
}

/// Get the current Leptos version
pub fn leptos_version() -> LeptosVersion {
    LeptosVersion::current()
}

/// Compatibility trait for different Leptos versions
pub trait LeptosCompat {
    /// Get the current Leptos version
    fn version() -> LeptosVersion;
    
    /// Check if this is Leptos 0.8 or later
    fn is_0_8_or_later() -> bool {
        Self::version().is_0_8_or_later()
    }
}

impl LeptosCompat for LeptosVersion {
    fn version() -> LeptosVersion {
        leptos_version()
    }
}

// Re-export Leptos types based on version
#[cfg(feature = "leptos-0-6")]
pub use leptos::{component, IntoView, create_signal, create_effect, create_memo, provide_context, use_context, create_resource, Signal, ReadSignal, WriteSignal, Resource};

#[cfg(feature = "leptos-0-8")]
pub use leptos_0_8::{component, IntoView, create_signal, create_effect, create_memo, provide_context, use_context, create_resource, Signal, ReadSignal, WriteSignal, Resource};

// Compatibility re-exports for common types
pub mod signals;
pub mod effects;
pub mod components;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_leptos_version_detection() {
        let version = leptos_version();
        // Should default to 0.6 in tests unless explicitly configured
        assert!(matches!(version, LeptosVersion::V0_6));
    }
    
    #[test]
    fn test_version_string() {
        assert_eq!(LeptosVersion::V0_6.as_str(), "0.6");
        assert_eq!(LeptosVersion::V0_8.as_str(), "0.8");
    }
    
    #[test]
    fn test_version_comparison() {
        assert!(!LeptosVersion::V0_6.is_0_8_or_later());
        assert!(LeptosVersion::V0_8.is_0_8_or_later());
    }
    
    #[test]
    fn test_compat_trait() {
        assert!(!<LeptosVersion as LeptosCompat>::is_0_8_or_later());
        assert_eq!(<LeptosVersion as LeptosCompat>::version(), LeptosVersion::V0_6);
    }
}
