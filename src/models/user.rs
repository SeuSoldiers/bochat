use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: String,
    pub name: String,
    pub id_number: String, // 身份证号
    pub phone: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub id_number: String, // 18位身份证号
    pub phone: String,
}

/// 登录请求
///
/// 使用身份证号和手机号进行身份验证
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub id_number: String, // 18位身份证号
    pub phone: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub user_id: String,
    pub name: String,
    pub id_number: String,
    pub phone: String,
    pub created_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            user_id: user.user_id,
            name: user.name,
            id_number: user.id_number,
            phone: user.phone,
            created_at: user.created_at,
        }
    }
}
