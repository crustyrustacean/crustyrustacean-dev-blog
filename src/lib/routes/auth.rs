// src/lib/routes/auth.rs

// route handlers for authentication pages

// dependencies
use crate::errors::AppError;
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse};
use axum_macros::debug_handler;
use axum_template::RenderHtml;
use serde::Serialize;

// struct type to represent login page content
#[derive(Debug, Serialize)]
struct LoginPageContent {
    title: String,
    error: Option<String>,
}

// struct type to represent register page content
#[derive(Debug, Serialize)]
struct RegisterPageContent {
    title: String,
    error: Option<String>,
}

// handler which renders the login page template
#[debug_handler]
pub async fn get_login_page(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let login_content = LoginPageContent {
        title: "Login".to_string(),
        error: None,
    };

    Ok(RenderHtml("auth/login.html", state.engine, login_content))
}

// handler which renders the register page template
#[debug_handler]
pub async fn get_register_page(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let register_content = RegisterPageContent {
        title: "Register".to_string(),
        error: None,
    };

    Ok(RenderHtml("auth/register.html", state.engine, register_content))
}