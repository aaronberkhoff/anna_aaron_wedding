use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Photo {
    pub id: Uuid,
    pub filename: String,
    pub caption: Option<String>,
    pub taken_at: Option<String>,
    pub uploaded_at: String,
    pub size_bytes: Option<u64>,
}
