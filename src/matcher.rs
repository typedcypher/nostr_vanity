use crate::generator::NostrKeyPair;

#[derive(Debug, Clone)]
pub enum MatchType {
    Prefix,
    Suffix,
    Contains,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub value: String,
    pub match_type: MatchType,
    pub case_sensitive: bool,
}

impl Pattern {
    pub fn new(value: String, match_type: MatchType, case_sensitive: bool) -> Self {
        let value = if case_sensitive {
            value
        } else {
            value.to_lowercase()
        };
        
        Pattern {
            value,
            match_type,
            case_sensitive,
        }
    }
    
    pub fn matches(&self, npub: &str) -> bool {
        let npub_without_prefix = &npub[5..];
        
        let compare_str = if self.case_sensitive {
            npub_without_prefix.to_string()
        } else {
            npub_without_prefix.to_lowercase()
        };
        
        match self.match_type {
            MatchType::Prefix => compare_str.starts_with(&self.value),
            MatchType::Suffix => compare_str.ends_with(&self.value),
            MatchType::Contains => compare_str.contains(&self.value),
        }
    }
}

pub struct PatternMatcher {
    patterns: Vec<Pattern>,
}

impl PatternMatcher {
    pub fn from_strings(
        values: Vec<String>, 
        match_type: MatchType, 
        case_sensitive: bool
    ) -> Self {
        let patterns = values
            .into_iter()
            .map(|v| Pattern::new(v, match_type.clone(), case_sensitive))
            .collect();
        
        PatternMatcher { patterns }
    }
    
    pub fn find_match(&self, keypair: &NostrKeyPair) -> Option<Pattern> {
        for pattern in &self.patterns {
            if pattern.matches(&keypair.npub) {
                return Some(pattern.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_matching() {
        let pattern = Pattern::new("test".to_string(), MatchType::Prefix, false);
        assert!(pattern.matches("npub1test123456"));
        assert!(!pattern.matches("npub1abc123456"));
        
        let pattern = Pattern::new("end".to_string(), MatchType::Suffix, false);
        assert!(pattern.matches("npub1123456end"));
        assert!(!pattern.matches("npub1123456abc"));
        
        let pattern = Pattern::new("mid".to_string(), MatchType::Contains, false);
        assert!(pattern.matches("npub1123mid456"));
        assert!(!pattern.matches("npub1123456789"));
    }
}