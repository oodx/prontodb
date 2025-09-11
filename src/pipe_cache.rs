// Pipe Cache System for ProntoDB
// Automatically caches piped content when addresses are invalid,
// providing zero data loss with user-friendly recovery workflow

use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum allowed pipe input size (10MB)
const MAX_PIPE_SIZE: usize = 10 * 1024 * 1024;

/// Default TTL for pipe cache entries (15 minutes)
pub const DEFAULT_PIPE_CACHE_TTL: u64 = 900;

/// Generate unique cache key from content and invalid address
pub fn generate_cache_key(content: &str, invalid_address: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let content_hash = format!("{:x}", md5::compute(content));
    let safe_address = invalid_address
        .replace(".", "_")
        .replace("/", "_")
        .replace(":", "_")
        .replace(" ", "_");
    
    format!("pipe.cache.{}_{}_{}", timestamp, &content_hash[..8], safe_address)
}

/// Detect if stdin has piped content
pub fn detect_pipe_input() -> Option<String> {
    // Check if stdin is not a TTY (i.e., it's piped)
    if !atty::is(atty::Stream::Stdin) {
        let mut buffer = String::new();
        let mut stdin = io::stdin();
        let mut limited_reader = stdin.take(MAX_PIPE_SIZE as u64);
        
        match limited_reader.read_to_string(&mut buffer) {
            Ok(_) => {
                // Check if we hit the size limit
                if buffer.len() >= MAX_PIPE_SIZE {
                    eprintln!("⚠️  Piped input exceeds maximum size of {}MB and was truncated", 
                             MAX_PIPE_SIZE / (1024 * 1024));
                }
                
                if !buffer.trim().is_empty() {
                    Some(buffer)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    } else {
        None
    }
}

/// Generate a pipe cache entry and return the cache key
/// This function only generates the cache key, actual storage handled by caller
pub fn prepare_pipe_cache(
    content: &str,
    invalid_address: &str,
) -> (String, String) {
    let cache_key = generate_cache_key(content, invalid_address);
    (cache_key, content.to_string())
}

/// Handle pipe input when set command fails
/// Returns (cache_key, content) if piped input detected, None otherwise
pub fn detect_and_prepare_pipe_cache(
    invalid_address: &str,
) -> Option<(String, String)> {
    // Only handle pipe cache if we detect piped input
    if let Some(content) = detect_pipe_input() {
        let (cache_key, cached_content) = prepare_pipe_cache(&content, invalid_address);
        
        // Provide user feedback
        eprintln!("⚠️  Invalid address '{}' - content cached as: {}", invalid_address, cache_key);
        eprintln!("💡 Use: prontodb copy {} <proper.address>", cache_key);
        
        Some((cache_key, cached_content))
    } else {
        // No piped input
        None
    }
}

/// Check if a key looks like a pipe cache entry
#[allow(dead_code)]
pub fn is_pipe_cache_key(key: &str) -> bool {
    key.starts_with("pipe.cache.")
}

/// Suggest XStream format for cached content (progressive education)
pub fn suggest_xstream_format(cache_key: &str, target_address: &str) -> String {
    format!(
        "💡 XStream format: echo \"ns={}; key=$(cat {});\" | prontodb stream", 
        target_address.split('.').next().unwrap_or("project"),
        cache_key
    )
}

/// Detect if cached content looks like XStream tokens
pub fn detect_xstream_in_cache(content: &str) -> bool {
    // Simple heuristic: contains semicolons and equals signs typical of XStream
    let has_structure = content.contains(';') && content.contains('=');
    let has_namespace = content.contains("ns=") || content.contains("meta:") || content.contains("sec:");
    has_structure && (has_namespace || content.matches('=').count() >= 2)
}

/// Enhanced pipe cache preparation with XStream education
pub fn prepare_pipe_cache_with_education(
    content: &str,
    invalid_address: &str,
) -> (String, String, Option<String>) {
    let (cache_key, cached_content) = prepare_pipe_cache(content, invalid_address);
    
    // Check if content might be XStream format
    let education_suggestion = if detect_xstream_in_cache(content) {
        Some(format!(
            "🎓 Detected structured data! Try XStream format:\n   echo \"{}\" | prontodb stream",
            content.trim()
        ))
    } else {
        None
    };
    
    (cache_key, cached_content, education_suggestion)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_cache_key() {
        let content = "test content";
        let address = "invalid.address";
        let key = generate_cache_key(content, address);
        
        assert!(key.starts_with("pipe.cache."));
        assert!(key.contains("invalid_address"));
        assert!(key.len() > 20); // Should have timestamp + hash
    }

    #[test]
    fn test_is_pipe_cache_key() {
        assert!(is_pipe_cache_key("pipe.cache.1234_abcd_test"));
        assert!(!is_pipe_cache_key("regular.key"));
        assert!(!is_pipe_cache_key("cache.pipe.test"));
    }

    #[test]
    fn test_suggest_xstream_format() {
        let suggestion = suggest_xstream_format("pipe.cache.123_abc", "project.namespace.key");
        assert!(suggestion.contains("ns=project"));
        assert!(suggestion.contains("pipe.cache.123_abc"));
    }
}