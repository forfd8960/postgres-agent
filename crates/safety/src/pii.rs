//! PII detection.

use lazy_static::lazy_static;
use regex::Regex;

/// PII types that can be detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiiType {
    /// Social Security Number.
    Ssn,
    /// Credit Card Number.
    CreditCard,
    /// Email Address.
    Email,
    /// Phone Number.
    Phone,
    /// IP Address.
    IpAddress,
}

/// PII detector.
#[derive(Debug, Default)]
pub struct PiiDetector {
    /// PII patterns.
    patterns: Vec<(Regex, PiiType)>,
}

impl PiiDetector {
    /// Create a new PII detector.
    #[must_use]
    pub fn new() -> Self {
        let patterns = vec![
            // SSN pattern (simplified)
            (Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(), PiiType::Ssn),
            // Credit card (simplified)
            (Regex::new(r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b").unwrap(), PiiType::CreditCard),
            // Email
            (Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap(), PiiType::Email),
            // Phone (various formats)
            (Regex::new(r"\b(?:\+?1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}\b").unwrap(), PiiType::Phone),
            // IP Address
            (Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap(), PiiType::IpAddress),
        ];
        Self { patterns }
    }

    /// Check if content contains PII.
    #[must_use]
    pub fn contains_pii(&self, content: &str) -> bool {
        self.patterns.iter().any(|(re, _)| re.is_match(content))
    }

    /// Redact PII from content.
    #[must_use]
    pub fn redact(&self, content: &str) -> String {
        let mut result = content.to_string();
        for (re, pii_type) in &self.patterns {
            let replacement = format!("[{}]", pii_type.label());
            result = re.replace_all(&result, replacement).to_string();
        }
        result
    }
}

impl PiiType {
    /// Get a human-readable label for the PII type.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ssn => "SSN",
            Self::CreditCard => "CREDIT_CARD",
            Self::Email => "EMAIL",
            Self::Phone => "PHONE",
            Self::IpAddress => "IP_ADDRESS",
        }
    }
}

lazy_static! {
    /// Default PII detector instance.
    pub static ref DEFAULT_DETECTOR: PiiDetector = PiiDetector::new();
}

/// Get the default PII detector.
#[must_use]
pub fn default_pii_detector() -> PiiDetector {
    PiiDetector::new()
}
