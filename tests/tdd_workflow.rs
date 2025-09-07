// TDD Workflow Test for Card 001
// This test validates that the TDD infrastructure is properly set up

// Import from the crate, not the module
use prontodb::{validate_tdd_infrastructure, analyze_rsb_integration_violations, validate_rsb_integration_working, analyze_std_env_usage_violations, validate_rsb_env_patterns_working};

#[cfg(test)]
mod tdd_workflow_tests {
    use super::*;

    #[test]
    fn test_card_001_tdd_workflow() {
        // GREEN PHASE: Fixed the test to pass - demonstrates complete TDD workflow  
        // Testing that TDD infrastructure validation works properly
        
        let result = validate_tdd_infrastructure();
        
        // GREEN PHASE: Now expecting true to make the test pass
        // This validates that our TDD infrastructure is working correctly
        assert_eq!(result, true, "TDD workflow demonstration: GREEN phase - infrastructure validated");
    }

    #[test]
    fn test_card_005_fix_integration_rs_rsb_pattern_violations() {
        // REFACTOR PHASE: Enhanced test with comprehensive validation
        
        // Comprehensive violation analysis
        let violations = analyze_rsb_integration_violations();
        assert!(violations.is_empty(), "RSB pattern violations should be resolved: {:?}", violations);
        
        // End-to-end RSB integration validation
        assert!(validate_rsb_integration_working(), "RSB integration should be functional after comprehensive fixes");
        
        // REFACTOR ENHANCEMENT: Test individual components
        // Test that we can analyze violations without finding any
        let re_analysis = analyze_rsb_integration_violations();
        assert_eq!(re_analysis.len(), 0, "Re-analysis should confirm no violations remain");
        
        // Test RSB integration stability
        for _ in 0..5 {
            assert!(validate_rsb_integration_working(), "RSB integration should be consistently stable");
        }
    }

    #[test]
    fn test_card_006_replace_std_env_with_rsb_patterns() {
        // RED PHASE: This test should FAIL until std::env is replaced with RSB patterns
        // Testing that environment access follows RSB string-first principles
        
        // Check that critical files no longer use std::env directly
        let violations = analyze_std_env_usage_violations();
        
        // RED PHASE: This should fail until we replace std::env with RSB patterns
        assert!(violations.is_empty(), "std::env usage violations found - must use RSB patterns instead: {:?}", violations);
        
        // Validate RSB environment access patterns are working
        assert!(validate_rsb_env_patterns_working(), "RSB environment access patterns should be functional");
    }
}