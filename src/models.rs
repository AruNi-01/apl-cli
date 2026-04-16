use serde::{Deserialize, Serialize};

// ── Responses ──────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct EnvCluster {
    pub env: String,
    pub clusters: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceInfo {
    pub app_id: String,
    pub cluster_name: String,
    pub namespace_name: String,
    pub comment: Option<String>,
    pub format: String,
    pub is_public: bool,
    #[serde(default)]
    pub items: Vec<ConfigItem>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigItem {
    #[serde(default)]
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub comment: Option<String>,
    pub data_change_created_by: Option<String>,
    pub data_change_last_modified_by: Option<String>,
    pub data_change_created_time: Option<String>,
    pub data_change_last_modified_time: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInfo {
    pub app_id: String,
    pub cluster_name: String,
    pub namespace_name: String,
    pub name: String,
    pub configurations: serde_json::Value,
    pub comment: Option<String>,
}

// ── Requests ───────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateItemRequest {
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub data_change_created_by: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateItemRequest {
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub data_change_last_modified_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_change_created_by: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishRequest {
    pub release_title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_comment: Option<String>,
    pub released_by: String,
}
