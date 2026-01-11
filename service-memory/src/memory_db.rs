use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use heleny_proto::ChatRole;
use heleny_proto::DisplayMessage;
use heleny_proto::MemoryContent;
use heleny_proto::MemoryEntry;
use sqlx::Pool;
use sqlx::Row;
use sqlx::Sqlite;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
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

    pub async fn save_entry(&self, entry: MemoryEntry) -> anyhow::Result<i64> {
        self.save(
            entry.role.to_str(),
            entry.time,
            &serde_json::to_string(&entry.content)?,
            None,
        )
        .await
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

    /// 旧->新
    pub async fn get_chat_messages(&self, n: i64) -> anyhow::Result<Vec<MemoryEntry>> {
        // 1. 执行查询
        // 使用 DESC 排序获取物理上最后存入的 n 条
        let rows = sqlx::query("SELECT role, time, content FROM memories ORDER BY id DESC LIMIT ?")
            .bind(n)
            .fetch_all(&self.pool)
            .await?;

        // 2. 解析数据
        let mut entries = Vec::new();
        for row in rows {
            // 从数据库读取原始数据
            let role_str: String = row.get("role");
            let time: DateTime<Local> = row.get("time");
            let content_json: String = row.get("content");

            // 将数据库里的 JSON 字符串解析回 MemoryContent 枚举
            let content: MemoryContent = serde_json::from_str(&content_json)
                .map_err(|e| anyhow::anyhow!("解析内存 JSON 失败: {}", e))?;

            // 还原 ChatRole (假设你实现了从 String 到 Enum 的转换，或者简单匹配)
            let role = ChatRole::from(&role_str);

            entries.push(MemoryEntry {
                role,
                time,
                content,
            });
        }
        entries.reverse();

        Ok(entries)
    }

    pub async fn get_display_messages(
        &self,
        id_upper_bound: i64,
        n: i64,
    ) -> anyhow::Result<Vec<DisplayMessage>> {
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

            entries.push(DisplayMessage {
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

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
