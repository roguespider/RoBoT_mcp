mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve_exception() {
        let mut tracker = ExceptionTracker::new();
        let exp_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        
        let exception = Exception::new(
            exp_id,
            pattern_id,
            0.5,
            "Test deviation".to_string(),
        );
        
        tracker.add_exception(exception);
        
        let exceptions = tracker.get_for_pattern(&pattern_id);
        assert_eq!(exceptions.len(), 1);
    }

    #[test]
    fn test_exception_threshold() {
        let mut tracker = ExceptionTracker::with_threshold(0.5);
        let exp_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        
        // Below threshold
        let low_exception = Exception::new(exp_id, pattern_id, 0.3, "Low deviation".to_string());
        tracker.add_exception(low_exception);
        
        assert_eq!(tracker.count(), 0);
        
        // Above threshold
        let high_exception = Exception::new(exp_id, pattern_id, 0.6, "High deviation".to_string());
        tracker.add_exception(high_exception);
        
        assert_eq!(tracker.count(), 1);
    }
