#[derive(Debug, Clone, Default)]
pub enum SubmissionStatus {
    #[default]
    Idle,
    Validating,
    Success(String),
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct AccountManagementState {
    pub token_input: String,
    pub status: SubmissionStatus,
}
