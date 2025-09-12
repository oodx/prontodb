//! XStream integration module with feature-gated compilation
//! 
//! This module provides streaming capabilities when the `streaming` feature is enabled.
//! When disabled, provides graceful error messages to users.

#[cfg(feature = "streaming")]
use std::io::{self, Read};

/// Maximum allowed streaming input size (10MB)
#[allow(dead_code)]  // Used in conditional compilation - false positive when streaming feature disabled
const MAX_STREAM_SIZE: usize = 10 * 1024 * 1024;

/// Check if streaming feature is enabled at runtime
pub fn is_streaming_enabled() -> bool {
    cfg!(feature = "streaming")
}

/// Main streaming command handler
pub fn handle_stream_command() -> Result<(), String> {
    #[cfg(feature = "streaming")]
    {
        handle_stream_with_xstream()
    }
    
    #[cfg(not(feature = "streaming"))]
    {
        Err("Streaming feature not enabled. Build with '--features streaming' to enable XStream integration.".to_string())
    }
}

#[cfg(feature = "streaming")]
fn handle_stream_with_xstream() -> Result<(), String> {
    use xstream::{tokenize_string, collect_tokens, BucketMode};
    
    // Read from stdin with memory protection
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut limited_reader = stdin.take(MAX_STREAM_SIZE as u64);
    
    limited_reader.read_to_string(&mut buffer)
        .map_err(|e| format!("IO error: {}", e))?;
    
    // Check if we hit the limit
    if buffer.len() >= MAX_STREAM_SIZE {
        return Err(format!("Input exceeds maximum size of {} bytes ({}MB)", 
                          MAX_STREAM_SIZE, MAX_STREAM_SIZE / (1024 * 1024)));
    }
    
    if buffer.trim().is_empty() {
        return Err("No input provided to stream command".to_string());
    }
    
    // Tokenize the input
    let tokens = tokenize_string(&buffer)
        .map_err(|e| format!("Failed to tokenize stream: {}", e))?;
    
    // Collect into bucket using Flat mode (namespace-like behavior)
    let bucket = collect_tokens(&tokens, BucketMode::Flat);
    
    // Process namespaces
    process_token_bucket(bucket)
}

#[cfg(feature = "streaming")]
fn process_token_bucket(bucket: xstream::TokenBucket) -> Result<(), String> {
    use crate::storage::Storage;
    use crate::xdg::XdgPaths;
    
    // Get paths and open default database
    let paths = XdgPaths::new();
    paths.ensure_dirs().map_err(|e| e.to_string())?;
    let db_path = paths.get_db_path_with_name("main");
    let storage = Storage::open(&db_path).map_err(|e| e.to_string())?;
    let mut processed_count = 0;
    
    // Process each namespace in the bucket
    for (namespace, values) in bucket.data.iter() {
        match namespace.as_str() {
            "meta" => {
                // Meta namespace contains directives
                for (key, value) in values {
                    println!("📋 Meta directive: {} = {}", key, value);
                    // Could implement special meta handling here
                }
            },
            "sec" => {
                // Security namespace (future implementation)
                for (key, value) in values {
                    println!("🔒 Security setting: {} = {}", key, value);
                    // Security directives would be processed here
                }
            },
            _ => {
                // Regular data namespace - store in database
                for (key, value) in values {
                    use crate::addressing::Address;
                    let address_str = format!("{}.{}", namespace, key);
                    let address = Address::parse(&address_str, ".")
                        .map_err(|e| format!("Invalid address {}: {}", address_str, e))?;
                    storage.set(&address, value, None)
                        .map_err(|e| format!("Failed to store {}: {}", address_str, e))?;
                    processed_count += 1;
                    println!("✅ Stored: {} = {}", address, value);
                }
            }
        }
    }
    
    println!("⚡ Stream processing complete: {} items stored", processed_count);
    Ok(())
}

/// Parse stream command with optional format argument
#[allow(dead_code)]
pub fn parse_stream_args(args: &[String]) -> Result<StreamFormat, String> {
    if args.is_empty() {
        return Ok(StreamFormat::Auto);
    }
    
    match args[0].as_str() {
        "xstream" | "tokens" => Ok(StreamFormat::XStream),
        "json" => Ok(StreamFormat::Json),
        "auto" => Ok(StreamFormat::Auto),
        _ => Err(format!("Unknown stream format: {}. Use 'xstream', 'json', or 'auto'", args[0]))
    }
}

/// Stream format options
#[derive(Debug, Clone)]
pub enum StreamFormat {
    Auto,     // Auto-detect format
    XStream,  // Token stream format
    Json,     // JSON format (future)
}

/// Detect if input looks like XStream format
#[allow(dead_code)]
pub fn detect_xstream_format(content: &str) -> bool {
    // Simple heuristic: contains semicolons and equals signs
    content.contains(';') && content.contains('=') && 
    (content.contains("ns=") || content.contains("meta:") || content.contains("sec:"))
}

/// Provide helpful format suggestion when streaming fails
#[allow(dead_code)]
pub fn suggest_stream_format(error_context: &str) -> String {
    format!(
        r#"Stream format not recognized. Try:
  XStream format: echo "ns=project; key=value; other=data;" | prontodb stream
  With meta:      echo "meta:ttl=300; ns=cache; data=content;" | prontodb stream
  With security:  echo "sec:auth=token123; ns=api; response=data;" | prontodb stream
  
Error context: {}"#,
        error_context
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_streaming_enabled_check() {
        #[cfg(feature = "streaming")]
        assert!(is_streaming_enabled());
        
        #[cfg(not(feature = "streaming"))]
        assert!(!is_streaming_enabled());
    }
    
    #[test]
    fn test_format_detection() {
        assert!(detect_xstream_format("ns=project; key=value;"));
        assert!(detect_xstream_format("meta:ttl=300; data=test;"));
        assert!(!detect_xstream_format("just plain text"));
        assert!(!detect_xstream_format("key:value")); // Missing equals
    }
    
    #[test]
    fn test_parse_stream_args() {
        let result = parse_stream_args(&["xstream".to_string()]);
        assert!(matches!(result.unwrap(), StreamFormat::XStream));
        
        let result = parse_stream_args(&[]);
        assert!(matches!(result.unwrap(), StreamFormat::Auto));
    }
}