#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfile {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    pub city: String,
    pub company_name: String,
}
