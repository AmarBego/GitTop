use crate::github::SubjectType;

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    SelectType(Option<SubjectType>),
    SelectRepo(Option<String>),
    SwitchAccount(String),
    OpenSettings,
    Logout,
}
