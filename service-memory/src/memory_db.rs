use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use heleny_proto::ChatRole;
use heleny_proto::MemoryEntry;
use heleny_proto::MemoryContent;
use sqlx::Pool;
use sqlx::Row;
use sqlx::Sqlite;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

static INIT_SQL: &'static str = r#"
    CREATE TABLE IF NOT EXISTS memories (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        role TEXT NOT NULL,
        time DATETIME NOT NULL,
        content TEXT NOT NULL,
        embedding BLOB
    );
    CREATE INDEX IF NOT EXISTS idx_mem_time ON memories(time);
    CREATE INDEX IF NOT EXISTS idx_mem_role ON memories(role);
"#;

pub struct MemoryDb {
    pool: Pool<Sqlite>,
}

impl MemoryDb {
    pub async fn new(path: &Path) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        sqlx::query(INIT_SQL).execute(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn save_entry(&self, role: ChatRole,time: DateTime<Local>,content: MemoryContent) -> anyhow::Result<i64> {
        self.save(
            role.to_str(),
            time,
            &serde_json::to_string(&content)?,
            None,
        )
        .await
    }

    pub async fn delete_entry(&self, id: i64) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM memories WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn save(
        &self,
        role: &str,
        time: DateTime<Local>,
        content: &str,
        vec: Option<Vec<u8>>,
    ) -> anyhow::Result<i64> {
        let result = sqlx::query(
            "INSERT INTO memories (role, time, content, embedding) VALUES (?, ?, ?, ?)",
        )
        .bind(role)
        .bind(time)
        .bind(content)
        .bind(vec)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_display_messages(
        &self,
        id_upper_bound: i64,
        n: i64,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        // 1. 执行查询
        // 筛选 id 小于上限的记录，按 id 倒序排列取前 n 条
        let rows = sqlx::query(
            "SELECT id, role, time, content FROM memories \
            WHERE id < ? \
            ORDER BY id DESC LIMIT ?",
        )
        .bind(id_upper_bound)
        .bind(n)
        .fetch_all(&self.pool)
        .await?;

        // 2. 解析数据
        let mut entries = Vec::new();
        for row in rows {
            let id: i64 = row.get("id");
            let role_str: String = row.get("role");
            let time: DateTime<Local> = row.get("time");
            let content_json: String = row.get("content");

            // 解析 JSON 内容
            let content: MemoryContent = serde_json::from_str(&content_json)
                .map_err(|e| anyhow::anyhow!("解析内存 JSON 失败: {}", e))?;

            // 转换 Role
            let role = ChatRole::from(&role_str);

            entries.push(MemoryEntry {
                id,
                role,
                time,
                content,
            });
        }

        // 3. 翻转顺序
        // 数据库取出的是 [99, 98, 97...]，前端展示需要 [97, 98, 99...]
        entries.reverse();

        Ok(entries)
    }

    pub async fn get_entries_by_ids(
        &self,
        ids: &HashSet<i64>,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut placeholders = String::new();
        for i in 0..ids.len() {
            if i > 0 {
                placeholders.push_str(", ");
            }
            placeholders.push('?');
        }

        let query = format!(
            "SELECT id, role, time, content FROM memories WHERE id IN ({}) ORDER BY id ASC",
            placeholders
        );
        let mut stmt = sqlx::query(&query);
        for id in ids {
            stmt = stmt.bind(id);
        }

        let rows = stmt.fetch_all(&self.pool).await?;
        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            let id: i64 = row.get("id");
            let role_str: String = row.get("role");
            let time: DateTime<Local> = row.get("time");
            let content_json: String = row.get("content");

            let content: MemoryContent = serde_json::from_str(&content_json)
                .map_err(|e| anyhow::anyhow!("解析成 MemoryContent 失败: {}", e))?;
            let role = ChatRole::from(&role_str);

            entries.push(MemoryEntry {
                id,
                role,
                time,
                content,
            });
        }

        Ok(entries)
    }

    pub async fn get_content_not_in_ids(
        &self,
        ids: &HashSet<i64>,
    ) -> anyhow::Result<HashMap<i64,String>> {
        let mut query = String::from("SELECT id, role, time, content FROM memories");
        if !ids.is_empty() {
            let mut placeholders = String::new();
            for i in 0..ids.len() {
                if i > 0 {
                    placeholders.push_str(", ");
                }
                placeholders.push('?');
            }
            query.push_str(&format!(" WHERE id NOT IN ({})", placeholders));
        }
        query.push_str(" ORDER BY id ASC");

        let mut stmt = sqlx::query(&query);
        for id in ids {
            stmt = stmt.bind(id);
        }

        let rows = stmt.fetch_all(&self.pool).await?;
        let mut entries = HashMap::with_capacity(rows.len());
        for row in rows {
            let id: i64 = row.get("id");
            let content_json: String = row.get("content");

            let content: MemoryContent = serde_json::from_str(&content_json)
                .map_err(|e| anyhow::anyhow!("解析成 MemoryContent 失败: {}", e))?;
            if let MemoryContent::Text(content)=content {
                entries.insert(id,content);
            }
            
        }

        Ok(entries)
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
