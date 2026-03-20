use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: String,
    pub name: String,
    pub id_number: String, // 身份证号
    pub phone: String,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub id_number: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
}

/// 登录请求
///
/// 使用身份证号和手机号进行身份验证
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    #[serde(default)]
    pub id_number: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub name: String,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            name: user.name,
            phone: normalize_optional_field(user.phone),
            avatar_url: user.avatar_url,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

fn normalize_optional_field(value: String) -> Option<String> {
    if value.is_empty() || value.starts_with("_none_") {
        None
    } else {
        Some(value)
    }
}
