use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub struct LogMetadata {
    pub parent_resource_id: String
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct LogStructure {
    pub level:          String,
    pub message:        String,
    pub resource_id:    String,
    pub timestamp:      DateTime<Utc>,
    pub trace_id:       String,
    pub span_id:        String,
    pub commit:         String,
    pub metadata:       LogMetadata
}
