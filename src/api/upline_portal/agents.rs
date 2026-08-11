use serde_json::Value;
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::api::users::shared::E;
use axum::http::StatusCode;

pub(super) fn root_id_from_agents(agents: &Value, role_slug: &str) -> Option<String> {
    agents.as_array()?.iter().find_map(|a| {
        if a.get("role").and_then(|v| v.as_str()) == Some(role_slug) {
            a.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
        } else {
            None
        }
    })
}

pub(super) fn collect_direct_and_downline_seller_ids(
    agents: &Value,
    role_slug: &str,
    fallback_root: Option<&str>,
) -> HashSet<String> {
    let root =
        root_id_from_agents(agents, role_slug).or_else(|| fallback_root.map(|s| s.to_string()));
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

/// Legacy per-role project columns only exist for lead-broker/titling-officer; any
/// other role resolves purely from agents_json.
pub(super) async fn load_project_agents(
    pool: &PgPool,
    project_id: Uuid,
    role_slug: &str,
) -> Result<(Value, Option<Uuid>), E> {
    let column = match role_slug {
        "lead-broker" => "lead_broker_roster_id",
        "titling-officer" => "titling_officer_roster_id",
        _ => "NULL",
    };
    let row = sqlx::query(&format!(
        "SELECT agents_json, {column} AS legacy_root_id FROM public.projects WHERE id = $1",
    ))
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
        row.try_get("legacy_root_id").ok().flatten(),
    ))
}
