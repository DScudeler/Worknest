//! Application mode configuration

/// Application operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Demo mode: Use in-memory data only, no backend API calls
    Demo,
    /// Integrated mode: Connect to real backend API (default)
    Integrated,
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Integrated
    }
}

impl AppMode {
    /// Parse mode from string (e.g., URL parameter)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "demo" => Self::Demo,
            "integrated" => Self::Integrated,
            _ => Self::default(), // Default to Integrated for invalid values
        }
    }

    /// Check if currently in demo mode
    pub fn is_demo(self) -> bool {
        matches!(self, Self::Demo)
    }

    /// Check if currently in integrated mode
    pub fn is_integrated(self) -> bool {
        matches!(self, Self::Integrated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_integrated() {
        assert_eq!(AppMode::default(), AppMode::Integrated);
    }

    #[test]
    fn test_parse_demo() {
        assert_eq!(AppMode::from_str("demo"), AppMode::Demo);
        assert_eq!(AppMode::from_str("Demo"), AppMode::Demo);
        assert_eq!(AppMode::from_str("DEMO"), AppMode::Demo);
    }

    #[test]
    fn test_parse_integrated() {
        assert_eq!(AppMode::from_str("integrated"), AppMode::Integrated);
        assert_eq!(AppMode::from_str("Integrated"), AppMode::Integrated);
    }

    #[test]
    fn test_parse_invalid_defaults_to_integrated() {
        assert_eq!(AppMode::from_str("invalid"), AppMode::Integrated);
        assert_eq!(AppMode::from_str(""), AppMode::Integrated);
        assert_eq!(AppMode::from_str("test"), AppMode::Integrated);
    }

    #[test]
    fn test_is_demo() {
        assert!(AppMode::Demo.is_demo());
        assert!(!AppMode::Integrated.is_demo());
    }

    #[test]
    fn test_is_integrated() {
        assert!(!AppMode::Demo.is_integrated());
        assert!(AppMode::Integrated.is_integrated());
    }
}
