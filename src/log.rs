use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub struct LogMetadata {
    parent_resource_id: String
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct LogStructure {
    level:          String,
    message:        String,
    resource_id:    String,
    timestamp:      String,
    trace_id:       String,
    span_id:        String,
    commit:         String,
    metadata:       LogMetadata
}
