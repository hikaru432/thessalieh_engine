use serde_json::Value;
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::api::users::shared::E;
use axum::http::StatusCode;

pub(super) fn titling_officer_id_from_agents(agents: &Value) -> Option<String> {
    agents.as_array()?.iter().find_map(|a| {
        if a.get("role").and_then(|v| v.as_str()) == Some("titling-officer") {
            a.get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    })
}

pub(super) fn collect_direct_and_downline_seller_ids(agents: &Value, fallback_root: Option<&str>) -> HashSet<String> {
    let root = titling_officer_id_from_agents(agents)
        .or_else(|| fallback_root.map(|s| s.to_string()));
    let Some(root) = root else {
        return HashSet::new();
    };

    let mut by_parent: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(arr) = agents.as_array() {
        for a in arr {
            let Some(id) = a.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            if let Some(parent) = a.get("parentId").and_then(|v| v.as_str()) {
                by_parent
                    .entry(parent.to_string())
                    .or_default()
                    .push(id.to_string());
            }
        }
    }

    let mut out = HashSet::new();
    out.insert(root.clone());
    let mut q = VecDeque::new();
    q.push_back(root);
    while let Some(id) = q.pop_front() {
        if let Some(children) = by_parent.get(&id) {
            for child in children {
                if out.insert(child.clone()) {
                    q.push_back(child.clone());
                }
            }
        }
    }
    out
}

pub(super) async fn load_project_agents(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<(Value, Option<Uuid>), E> {
    let row = sqlx::query(
        "SELECT agents_json, titling_officer_roster_id FROM public.projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load project")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Project not found"))?;

    Ok((
        row.try_get("agents_json")
            .unwrap_or_else(|_| Value::Array(vec![])),
        row.try_get("titling_officer_roster_id").ok().flatten(),
    ))
}

pub(super) fn resolve_to_subject_id(agents: &Value, titling_officer_roster_id: Option<Uuid>) -> Option<String> {
    titling_officer_id_from_agents(agents)
        .or_else(|| titling_officer_roster_id.map(|id| id.to_string()))
}
