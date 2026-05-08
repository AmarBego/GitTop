use crate::settings::CredentialStorage;

#[derive(Debug, Clone)]
pub enum AccountMessage {
    TokenInputChanged(String),
    SubmitToken,
    TokenValidated(Result<String, String>),
    RemoveAccount(String),
    CredentialStorageChanged(CredentialStorage),
}
