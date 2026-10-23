use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;

use crate::{database::work_orders::structs::WorkOrderStatus, error::AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum RecentItemKind {
    Client,
    Car,
    WorkOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RecentItem {
    pub kind: RecentItemKind,
    pub id: String,
    pub label: String,
    pub subtitle: Option<String>,
    pub status: Option<WorkOrderStatus>,
}

#[tauri::command]
#[specta::specta]
pub async fn record_recent_item(
    state: State<'_, crate::AppState>,
    kind: RecentItemKind,
    id: String,
) -> AppResult<()> {
    let exists: i64 = match kind {
        RecentItemKind::Client | RecentItemKind::Car => {
            let Ok(numeric_id) = id.parse::<i32>() else {
                return Ok(());
            };
            let table = match kind {
                RecentItemKind::Client => "clients",
                RecentItemKind::Car => "cars",
                RecentItemKind::WorkOrder => unreachable!(),
            };
            let sql = format!("SELECT COUNT(*) FROM {table} WHERE id = ? AND archived_at IS NULL");
            sqlx::query_scalar(&sql)
                .bind(numeric_id)
                .fetch_one(&state.db_pool)
                .await?
        }
        RecentItemKind::WorkOrder => {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM work_orders WHERE id = ? AND archived_at IS NULL",
            )
            .bind(&id)
            .fetch_one(&state.db_pool)
            .await?
        }
    };

    if exists == 0 {
        return Ok(());
    }

    sqlx::query("DELETE FROM recent_items WHERE kind = ? AND item_id = ?")
        .bind(kind)
        .bind(&id)
        .execute(&state.db_pool)
        .await?;
    sqlx::query(
        "INSERT INTO recent_items (kind, item_id, viewed_at)
         VALUES (?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
    )
    .bind(kind)
    .bind(&id)
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_recent_items(state: State<'_, crate::AppState>) -> AppResult<Vec<RecentItem>> {
    let items = sqlx::query_as::<_, RecentItem>(
        r#"
        SELECT kind, id, label, subtitle, status
        FROM (
            SELECT
                'client' AS kind,
                CAST(c.id AS TEXT) AS id,
                c.name AS label,
                NULL AS subtitle,
                NULL AS status,
                r.viewed_at
            FROM recent_items r
            JOIN clients c ON c.id = CAST(r.item_id AS INTEGER)
            WHERE r.kind = 'client' AND c.archived_at IS NULL

            UNION ALL

            SELECT
                'car' AS kind,
                CAST(car.id AS TEXT) AS id,
                b.name || ' ' || car.model AS label,
                NULL AS subtitle,
                NULL AS status,
                r.viewed_at
            FROM recent_items r
            JOIN cars car ON car.id = CAST(r.item_id AS INTEGER)
            JOIN car_brands b ON b.id = car.brand_id
            WHERE r.kind = 'car' AND car.archived_at IS NULL

            UNION ALL

            SELECT
                'work_order' AS kind,
                wo.id AS id,
                wo.id AS label,
                b.name || ' ' || car.model AS subtitle,
                wo.status AS status,
                r.viewed_at
            FROM recent_items r
            JOIN work_orders wo ON wo.id = r.item_id
            JOIN cars car ON car.id = wo.car_id
            JOIN car_brands b ON b.id = car.brand_id
            WHERE r.kind = 'work_order' AND wo.archived_at IS NULL
        )
        ORDER BY viewed_at DESC
        LIMIT 20
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(items)
}

#[tauri::command]
#[specta::specta]
pub async fn clear_recent_items(state: State<'_, crate::AppState>) -> AppResult<()> {
    sqlx::query("DELETE FROM recent_items")
        .execute(&state.db_pool)
        .await?;
    Ok(())
}
