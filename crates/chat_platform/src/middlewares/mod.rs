use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};

use crate::{
    error::{AppError, AppResult},
    http::{require_bot_bearer_token, require_user_bearer_token, token_bot_id, token_user_id},
    repositories::BotRepository,
    services::authz::ensure_user_exists,
    utils::{verify_token, verify_user_token},
    AppState,
};

#[derive(Clone, Debug)]
pub struct UserAuth {
    pub user_id: String,
}

#[derive(Clone, Debug)]
pub struct BotAuth {
    pub bot_id: String,
    pub owner_id: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

pub async fn require_user_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let auth = authenticate_user_headers(&state, req.headers()).await?;
    req.extensions_mut().insert(auth);
    Ok(next.run(req).await)
}

pub async fn require_bot_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let auth = authenticate_bot_headers(&state, req.headers()).await?;
    req.extensions_mut().insert(auth);
    Ok(next.run(req).await)
}

pub async fn authenticate_user_headers(state: &AppState, headers: &HeaderMap) -> AppResult<UserAuth> {
    let token = require_user_bearer_token(headers)?;
    let user_id = token_user_id(&token)
        .map(str::to_string)
        .map_err(|_| AppError::InvalidUserToken)?;

    verify_user_token(
        &token,
        &state.config.security.jwt_secret,
        state.config.security.token_expiry_secs,
    )
    .map_err(|_| AppError::InvalidUserToken)?;
    ensure_user_exists(&state.pool, &user_id).await?;

    Ok(UserAuth { user_id })
}

pub async fn authenticate_bot_headers(state: &AppState, headers: &HeaderMap) -> AppResult<BotAuth> {
    let token = require_bot_bearer_token(headers)?;
    let bot_id = token_bot_id(&token)
        .map(str::to_string)
        .map_err(|_| AppError::InvalidBotToken)?;

    let bot = BotRepository::find_by_id(&state.pool, &bot_id)
        .await?
        .ok_or(AppError::InvalidBotToken)?;

    verify_token(&token, &bot.secret, state.config.security.token_expiry_secs)
        .map_err(|_| AppError::InvalidBotToken)?;

    if bot.status != "active" {
        return Err(AppError::BotInactive);
    }

    Ok(BotAuth {
        bot_id: bot.bot_id,
        owner_id: bot.owner_id,
        name: bot.name,
        avatar_url: bot.avatar_url,
    })
}
