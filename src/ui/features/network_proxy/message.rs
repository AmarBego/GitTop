#[derive(Debug, Clone)]
pub enum ProxyMessage {
    ToggleEnabled(bool),
    UrlChanged(String),
    UsernameChanged(String),
    PasswordChanged(String),
    Save,
}
