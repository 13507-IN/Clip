use rusqlite::{params, Connection, Result as SqlResult};

use crate::model::UrlEntry;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn new(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL")?;
        conn.execute_batch("PRAGMA busy_timeout = 5000")?;
        conn.execute_batch("PRAGMA synchronous = NORMAL")?;
        conn.execute_batch("PRAGMA cache_size = -20000")?;

        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS urls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                short_code TEXT NOT NULL UNIQUE,
                original TEXT NOT NULL,
                created_at TEXT NOT NULL,
                clicks INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_short_code ON urls(short_code);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_original ON urls(original);",
        )
    }

    pub fn insert(&self, short_code: &str, original: &str) -> SqlResult<UrlEntry> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO urls (short_code, original, created_at) VALUES (?1, ?2, ?3)",
            params![short_code, original, now],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(UrlEntry {
            id,
            short_code: short_code.to_string(),
            original: original.to_string(),
            created_at: now,
            clicks: 0,
        })
    }

    pub fn get_by_short_code(&self, code: &str) -> SqlResult<Option<UrlEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, short_code, original, created_at, clicks FROM urls WHERE short_code = ?1",
        )?;
        let mut rows = stmt.query(params![code])?;
        match rows.next()? {
            Some(row) => Ok(Some(UrlEntry {
                id: row.get(0)?,
                short_code: row.get(1)?,
                original: row.get(2)?,
                created_at: row.get(3)?,
                clicks: row.get(4)?,
            })),
            None => Ok(None),
        }
    }

    pub fn get_by_original(&self, original: &str) -> SqlResult<Option<UrlEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, short_code, original, created_at, clicks FROM urls WHERE original = ?1",
        )?;
        let mut rows = stmt.query(params![original])?;
        match rows.next()? {
            Some(row) => Ok(Some(UrlEntry {
                id: row.get(0)?,
                short_code: row.get(1)?,
                original: row.get(2)?,
                created_at: row.get(3)?,
                clicks: row.get(4)?,
            })),
            None => Ok(None),
        }
    }

    pub fn increment_clicks(&self, code: &str) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE urls SET clicks = clicks + 1 WHERE short_code = ?1",
            params![code],
        )?;
        Ok(())
    }
}
