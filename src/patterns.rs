pub struct PiiPattern {
    pub name: &'static str,
    pub regex: &'static str,
}

pub const DEFAULT_PATTERNS: &[PiiPattern] = &[
    PiiPattern {
        name: "Email",
        regex: r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+",
    },
    PiiPattern {
        name: "SSN",
        regex: r"\b(?:\d{3}(?:[-–]|[ \t]+)\d{2}(?:[-–]|[ \t]+)\d{4}|\d{9}|(?:\d[ \t]+){8}\d)\b",
    },
    PiiPattern {
        name: "Phone",
        regex: r"(?:\+?1[-. ]?)?\(?([0-9]{3})\)?[-. ]?([0-9]{3})[-. ]?([0-9]{4})",
    },
    PiiPattern {
        name: "Address",
        regex: r"\d+\s+[A-Z][a-z]+\s+(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Drive|Dr|Lane|Ln|Way|Court|Ct|Circle|Cir|Trail|Trl|Parkway|Pkwy)",
    },
    PiiPattern {
        name: "Age",
        regex: r"\b\d{1,3}\s*(years? old|yrs? old)\b",
    },
    PiiPattern {
        name: "Biometric Keywords",
        regex: r"(?i)fingerprint|retina|biometric|dna|iris scan",
    },
];

pub fn escape_literal(s: &str) -> String {
    regex::escape(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_email_regex() {
        let pattern = DEFAULT_PATTERNS.iter().find(|p| p.name == "Email").unwrap();
        let re = Regex::new(pattern.regex).unwrap();
        assert!(re.is_match("test@example.com"));
        assert!(re.is_match("user.name+tag@domain.co.uk"));
    }

    #[test]
    fn test_ssn_regex() {
        let pattern = DEFAULT_PATTERNS.iter().find(|p| p.name == "SSN").unwrap();
        let re = Regex::new(pattern.regex).unwrap();
        for ssn in [
            "000-12-3456",
            "000123456",
            "000 12  3456",
            "000–12–3456",
            "0 0 0 1 2 3 4 5 6",
        ] {
            assert_eq!(re.find(ssn).map(|hit| hit.as_str()), Some(ssn));
        }
        for other in [
            "000-123-456",
            "0001234567",
            "X000123456",
            "0 0 0 1 2\n3 4 5 6",
        ] {
            assert!(!re.is_match(other), "unexpected SSN match in {other:?}");
        }
    }

    #[test]
    fn test_address_regex() {
        let pattern = DEFAULT_PATTERNS.iter().find(|p| p.name == "Address").unwrap();
        let re = Regex::new(pattern.regex).unwrap();
        assert!(re.is_match("123 Main Street"));
        assert!(re.is_match("456 Broadway Ave"));
    }

    #[test]
    fn test_biometric_regex() {
        let pattern = DEFAULT_PATTERNS.iter().find(|p| p.name == "Biometric Keywords").unwrap();
        let re = Regex::new(pattern.regex).unwrap();
        assert!(re.is_match("Fingerprint recorded"));
        assert!(re.is_match("DNA sample found"));
        assert!(re.is_match("Retina scan successful"));
    }
}
