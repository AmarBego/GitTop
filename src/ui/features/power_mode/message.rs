#[derive(Debug, Clone)]
pub enum PowerModeMessage {
    Toggle(bool),
    OpenRuleEngine,
}
