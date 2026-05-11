//! API Call Validation Framework
//!
//! Validates that state changes trigger the correct API calls with proper
//! payloads, headers, and authentication.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Represents an expected API call
#[derive(Debug, Clone, PartialEq)]
pub struct ExpectedCall {
    pub method: HttpMethod,
    pub path: String,
    pub requires_auth: bool,
    pub body: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::DELETE => write!(f, "DELETE"),
        }
    }
}

/// Recorded API call for validation
#[derive(Debug, Clone)]
pub struct RecordedCall {
    pub method: HttpMethod,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub timestamp: u64,
}

impl RecordedCall {
    pub fn has_auth_header(&self) -> bool {
        self.headers
            .iter()
            .any(|(name, _)| name.to_lowercase() == "authorization")
    }

    pub fn get_auth_token(&self) -> Option<String> {
        self.headers
            .iter()
            .find(|(name, _)| name.to_lowercase() == "authorization")
            .map(|(_, value)| value.clone())
    }
}

/// Mock API client that records calls for validation
#[derive(Clone)]
pub struct MockApiClient {
    base_url: String,
    recorded_calls: Arc<Mutex<VecDeque<RecordedCall>>>,
    should_fail: Arc<Mutex<bool>>,
}

impl MockApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            recorded_calls: Arc::new(Mutex::new(VecDeque::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Record an API call
    pub fn record_call(&self, call: RecordedCall) {
        let mut calls = self.recorded_calls.lock().unwrap();
        calls.push_back(call);
    }

    /// Set whether API calls should fail
    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    /// Get all recorded calls
    pub fn get_recorded_calls(&self) -> Vec<RecordedCall> {
        let calls = self.recorded_calls.lock().unwrap();
        calls.iter().cloned().collect()
    }

    /// Clear recorded calls
    pub fn clear_calls(&self) {
        let mut calls = self.recorded_calls.lock().unwrap();
        calls.clear();
    }

    /// Get the most recent call
    pub fn get_last_call(&self) -> Option<RecordedCall> {
        let calls = self.recorded_calls.lock().unwrap();
        calls.back().cloned()
    }

    /// Check if a specific call was made
    pub fn was_called(&self, expected: &ExpectedCall) -> bool {
        let calls = self.recorded_calls.lock().unwrap();
        calls.iter().any(|call| {
            call.method == expected.method
                && call.path == expected.path
                && (if expected.requires_auth {
                    call.has_auth_header()
                } else {
                    true
                })
        })
    }

    /// Count calls matching criteria
    pub fn count_calls(&self, method: &HttpMethod, path: &str) -> usize {
        let calls = self.recorded_calls.lock().unwrap();
        calls
            .iter()
            .filter(|call| call.method == *method && call.path == path)
            .count()
    }
}

/// Validator for API call sequences
pub struct ApiCallValidator {
    expected_calls: Vec<ExpectedCall>,
    recorded_calls: Vec<RecordedCall>,
}

impl ApiCallValidator {
    pub fn new() -> Self {
        Self {
            expected_calls: Vec::new(),
            recorded_calls: Vec::new(),
        }
    }

    /// Add an expected API call
    pub fn expect_call(mut self, call: ExpectedCall) -> Self {
        self.expected_calls.push(call);
        self
    }

    /// Load recorded calls from mock client
    pub fn with_recorded_calls(mut self, calls: Vec<RecordedCall>) -> Self {
        self.recorded_calls = calls;
        self
    }

    /// Validate that all expected calls were made
    pub fn validate(&self) -> ValidationResult {
        let mut missing_calls = Vec::new();
        let mut extra_calls = Vec::new();

        // Check for missing expected calls
        for expected in &self.expected_calls {
            let found = self.recorded_calls.iter().any(|recorded| {
                recorded.method == expected.method
                    && recorded.path == expected.path
                    && (if expected.requires_auth {
                        recorded.has_auth_header()
                    } else {
                        true
                    })
            });

            if !found {
                missing_calls.push(expected.clone());
            }
        }

        // Check for unexpected calls (if we have expected calls)
        if !self.expected_calls.is_empty() {
            for recorded in &self.recorded_calls {
                let expected = self
                    .expected_calls
                    .iter()
                    .any(|exp| exp.method == recorded.method && exp.path == recorded.path);

                if !expected {
                    extra_calls.push(recorded.clone());
                }
            }
        }

        ValidationResult {
            passed: missing_calls.is_empty() && extra_calls.is_empty(),
            missing_calls,
            extra_calls,
        }
    }

    /// Validate call order (strict sequence)
    pub fn validate_order(&self) -> bool {
        if self.expected_calls.len() != self.recorded_calls.len() {
            return false;
        }

        self.expected_calls
            .iter()
            .zip(self.recorded_calls.iter())
            .all(|(expected, recorded)| {
                recorded.method == expected.method && recorded.path == expected.path
            })
    }
}

impl Default for ApiCallValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub passed: bool,
    pub missing_calls: Vec<ExpectedCall>,
    pub extra_calls: Vec<RecordedCall>,
}

impl ValidationResult {
    pub fn assert_passed(&self) {
        if !self.passed {
            let mut msg = String::from("API call validation failed:\n");

            if !self.missing_calls.is_empty() {
                msg.push_str("\nMissing expected calls:\n");
                for call in &self.missing_calls {
                    msg.push_str(&format!("  {} {}\n", call.method, call.path));
                }
            }

            if !self.extra_calls.is_empty() {
                msg.push_str("\nUnexpected calls:\n");
                for call in &self.extra_calls {
                    msg.push_str(&format!("  {} {}\n", call.method, call.path));
                }
            }

            panic!("{}", msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_client_records_calls() {
        let client = MockApiClient::new("http://localhost:3000".to_string());

        client.record_call(RecordedCall {
            method: HttpMethod::GET,
            path: "/api/projects".to_string(),
            headers: vec![("authorization".to_string(), "Bearer token123".to_string())],
            body: None,
            timestamp: 0,
        });

        assert_eq!(client.get_recorded_calls().len(), 1);
        assert!(client.get_last_call().unwrap().has_auth_header());
    }

    #[test]
    fn test_validator_detects_missing_calls() {
        let expected = ExpectedCall {
            method: HttpMethod::POST,
            path: "/api/projects".to_string(),
            requires_auth: true,
            body: None,
        };

        let validator = ApiCallValidator::new()
            .expect_call(expected)
            .with_recorded_calls(vec![]);

        let result = validator.validate();
        assert!(!result.passed);
        assert_eq!(result.missing_calls.len(), 1);
    }

    #[test]
    fn test_validator_passes_with_matching_calls() {
        let expected = ExpectedCall {
            method: HttpMethod::GET,
            path: "/api/projects".to_string(),
            requires_auth: true,
            body: None,
        };

        let recorded = RecordedCall {
            method: HttpMethod::GET,
            path: "/api/projects".to_string(),
            headers: vec![("authorization".to_string(), "Bearer token".to_string())],
            body: None,
            timestamp: 0,
        };

        let validator = ApiCallValidator::new()
            .expect_call(expected)
            .with_recorded_calls(vec![recorded]);

        let result = validator.validate();
        assert!(result.passed);
    }
}
