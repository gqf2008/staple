//! Shared repository helpers: row reads, reference checks, dynamic UPDATEs.

use libsql::{Connection, Row};

/// Reads a TEXT column as `Option<String>` (`NULL` → `None`).
pub fn row_text(row: &Row, idx: i32) -> Result<Option<String>, libsql::Error> {
    let value = row.get_value(idx)?;
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(value.as_text().expect("TEXT column").clone()))
    }
}

/// Reads an INTEGER column.
pub fn row_i64(row: &Row, idx: i32) -> Result<i64, libsql::Error> {
    let value = row.get_value(idx)?;
    Ok(*value.as_integer().expect("INTEGER column"))
}

/// Whether a company exists.
pub async fn company_exists(conn: &Connection, company_id: &str) -> Result<bool, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT 1 FROM companies WHERE id = ?1",
            libsql::params![company_id],
        )
        .await?;
    Ok(rows.next().await?.is_some())
}

/// Whether a row exists in `table` (any table with an `id` column).
pub async fn find_row(conn: &Connection, table: &str, id: &str) -> Result<bool, libsql::Error> {
    let sql = format!("SELECT 1 FROM {table} WHERE id = ?1");
    let mut rows = conn.query(&sql, libsql::params![id]).await?;
    Ok(rows.next().await?.is_some())
}

/// Whether the row in `table` belongs to `company_id`.
pub async fn row_belongs_to_company(
    conn: &Connection,
    table: &str,
    id: &str,
    company_id: &str,
) -> Result<bool, libsql::Error> {
    let sql = format!("SELECT 1 FROM {table} WHERE id = ?1 AND company_id = ?2");
    let mut rows = conn.query(&sql, libsql::params![id, company_id]).await?;
    Ok(rows.next().await?.is_some())
}

/// The `company_id` of a row, or `None` when the row does not exist.
pub async fn row_company(
    conn: &Connection,
    table: &str,
    id: &str,
) -> Result<Option<String>, libsql::Error> {
    let sql = format!("SELECT company_id FROM {table} WHERE id = ?1");
    let mut rows = conn.query(&sql, libsql::params![id]).await?;
    match rows.next().await? {
        Some(row) => Ok(row_text(&row, 0)?),
        None => Ok(None),
    }
}

/// Builds `SET` fragments and bound values for a partial update.
///
/// Each entry is `(column, Option<Option<Value>>)`: `Some(Some(v))` binds a
/// value, `Some(None)` emits `column = NULL`, `None` skips the column.
pub fn build_update(
    fields: &[(&str, Option<Option<libsql::Value>>)],
) -> (Vec<String>, Vec<libsql::Value>) {
    let mut sets = Vec::new();
    let mut values = Vec::new();
    let mut param = 0usize;
    for (column, value) in fields {
        match value {
            Some(Some(value)) => {
                param += 1;
                sets.push(format!("{column} = ?{param}"));
                values.push(value.clone());
            }
            Some(None) => sets.push(format!("{column} = NULL")),
            None => {}
        }
    }
    (sets, values)
}

/// Executes `UPDATE table SET ... , updated_at = now WHERE id = ?N`.
pub async fn execute_update(
    conn: &Connection,
    table: &str,
    id: &str,
    sets: Vec<String>,
    mut values: Vec<libsql::Value>,
) -> Result<u64, libsql::Error> {
    let param = values.len() + 1;
    values.push(libsql::Value::from(id.to_owned()));
    let sql = format!(
        "UPDATE {table} SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
         WHERE id = ?{param}",
        sets.join(", ")
    );
    conn.execute(&sql, values).await
}
