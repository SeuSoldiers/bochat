use uuid::Uuid;

pub fn generate_user_id() -> String {
    format!("u_{}", Uuid::new_v4())
}

pub fn generate_bot_id() -> String {
    format!("b_{}", Uuid::new_v4())
}

pub fn generate_file_id() -> String {
    format!("f_{}", Uuid::new_v4())
}

pub fn generate_group_id() -> String {
    format!("g_{}", Uuid::new_v4())
}
