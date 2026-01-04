#[derive(Debug, Clone)]
pub enum OrgMessage {
    Toggle(String, bool),
    Delete(String),
    Duplicate(String),
}
