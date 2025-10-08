use axum::extract::{Json, State};
use serde::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub struct Message {
    pub email: String,
    pub content: String,
}
