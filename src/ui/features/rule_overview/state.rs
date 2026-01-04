#[derive(Debug, Clone, Default)]
pub struct RuleOverviewState {
    pub explain_test_type: String,
}

impl RuleOverviewState {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            explain_test_type: "Mentioned".to_string(),
        }
    }
}
