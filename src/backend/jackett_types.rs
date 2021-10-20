#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ConfigResponse {
 api_key: String,
 app_version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JackettResults {
 results: Vec<JackettResult>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JackettResult {
 tracker: Option<String>,
 tracker_id: Option<String>,
 category_desc: Option<String>,
 title: Option<String>,
 guid: Option<String>,
 link: Option<String>,
 details: Option<String>,
 publish_date: Option<String>,
 category: Option<Vec<i64>>,
 size: Option<i64>,
 seeders: Option<i64>,
 peers: Option<i64>,
 gain: Option<f64>,
}
