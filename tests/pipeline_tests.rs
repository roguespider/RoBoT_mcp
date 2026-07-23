mod tests {
    use super::*;

    #[test]
    fn test_start_pipeline() {
        let mut pipeline = LearningPipeline::new(100);
        let source_id = Uuid::new_v4();
        
        let record_id = pipeline.start_from_input(source_id, "Test input");
        
        let record = pipeline.get(&record_id).unwrap();
        assert_eq!(record.current_stage, PipelineStage::Input);
        assert_eq!(record.completed_stages.len(), 0);
    }

    #[test]
    fn test_advance_stage() {
        let mut pipeline = LearningPipeline::new(100);
        let source_id = Uuid::new_v4();
        
        let record_id = pipeline.start_from_input(source_id, "Test input");
        pipeline.advance_stage(&record_id, PipelineStage::Observation, "Observation made", Some(0.8));
        
        let record = pipeline.get(&record_id).unwrap();
        assert_eq!(record.current_stage, PipelineStage::Observation);
        assert!(record.completed_stages.contains(&PipelineStage::Input));
    }

    #[test]
    fn test_pipeline_stats() {
        let mut pipeline = LearningPipeline::new(100);
        
        let id1 = pipeline.start_from_input(Uuid::new_v4(), "Input 1");
        pipeline.advance_stage(&id1, PipelineStage::Observation, "Obs 1", None);
        
        let _id2 = pipeline.start_from_input(Uuid::new_v4(), "Input 2");
        
        let stats = pipeline.stats();
        assert_eq!(stats.total_records, 2);
        // id1 moved to Observation, id2 still in Input
        assert_eq!(stats.stage_counts.get(&PipelineStage::Input), Some(&1));
        assert_eq!(stats.stage_counts.get(&PipelineStage::Observation), Some(&1));
    }
