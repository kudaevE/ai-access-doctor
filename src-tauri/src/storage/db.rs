use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecordSummary {
    pub id: String,
    pub timestamp: String,
    pub verdict_state: String,
    pub primary_issue_code: String,
    pub title_ru: String,
    pub explanation_ru: String,
    pub passed_count: i64,
    pub warning_count: i64,
    pub failed_count: i64,
}

pub struct DiagnosticStorage {
    db_path: PathBuf,
}

impl DiagnosticStorage {
    pub fn new() -> Self {
        let appdata = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let db_dir = appdata.join("AI Access Doctor");
        let _ = std::fs::create_dir_all(&db_dir);
        let db_path = db_dir.join("ai_access_doctor.db");

        let storage = Self { db_path };
        let _ = storage.init_db();
        storage
    }

    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
    }

    pub fn init_db(&self) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS diagnostic_runs (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                verdict_state TEXT NOT NULL,
                primary_issue_code TEXT NOT NULL,
                title_ru TEXT NOT NULL,
                explanation_ru TEXT NOT NULL,
                passed_count INTEGER NOT NULL,
                warning_count INTEGER NOT NULL,
                failed_count INTEGER NOT NULL,
                raw_json TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn save_run(
        &self,
        id: &str,
        verdict_state: &str,
        issue_code: &str,
        title_ru: &str,
        explanation_ru: &str,
        passed: usize,
        warning: usize,
        failed: usize,
        raw_json: &str,
    ) -> Result<()> {
        let conn = self.get_connection()?;
        let now_str = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO diagnostic_runs (id, timestamp, verdict_state, primary_issue_code, title_ru, explanation_ru, passed_count, warning_count, failed_count, raw_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                now_str,
                verdict_state,
                issue_code,
                title_ru,
                explanation_ru,
                passed as i64,
                warning as i64,
                failed as i64,
                raw_json
            ],
        )?;

        Ok(())
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryRecordSummary>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, verdict_state, primary_issue_code, title_ru, explanation_ru, passed_count, warning_count, failed_count
             FROM diagnostic_runs
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(HistoryRecordSummary {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                verdict_state: row.get(2)?,
                primary_issue_code: row.get(3)?,
                title_ru: row.get(4)?,
                explanation_ru: row.get(5)?,
                passed_count: row.get(6)?,
                warning_count: row.get(7)?,
                failed_count: row.get(8)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(rec) = r {
                list.push(rec);
            }
        }

        Ok(list)
    }

    pub fn get_run_json(&self, id: &str) -> Result<Option<String>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT raw_json FROM diagnostic_runs WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            Ok(Some(json_str))
        } else {
            Ok(None)
        }
    }

    pub fn clear_history(&self) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM diagnostic_runs", [])?;
        Ok(())
    }
}

/// Redact personal usernames, computer names, and sensitive keys/tokens from report text
pub fn sanitize_text(input: &str) -> String {
    let mut sanitized = input.to_string();

    // Redact full UserProfile path if available
    if let Ok(profile) = std::env::var("USERPROFILE") {
        if !profile.is_empty() {
            sanitized = sanitized.replace(&profile, "%USERPROFILE%");
        }
    }

    // Redact current username if available
    if let Ok(user) = std::env::var("USERNAME") {
        if !user.is_empty() {
            sanitized = sanitized.replace(&user, "[USER]");
        }
    }

    // Redact computer name if available
    if let Ok(comp) = std::env::var("COMPUTERNAME") {
        if !comp.is_empty() {
            sanitized = sanitized.replace(&comp, "[PC]");
        }
    }

    // Redact Bearer tokens, API keys and secrets
    let words: Vec<&str> = sanitized.split_whitespace().collect();
    let mut output_words = Vec::new();
    let mut skip_next = false;

    for (i, &word) in words.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        let lower = word.to_lowercase();
        if (lower == "bearer" || lower == "token" || lower.starts_with("authorization:")) && i + 1 < words.len() {
            output_words.push(word);
            output_words.push("[REDACTED_SECRET]");
            skip_next = true;
        } else if word.starts_with("sk-")
            || word.starts_with("ghp_")
            || word.starts_with("gho_")
            || word.starts_with("github_pat_")
            || word.starts_with("AIzaSy")
            || (word.starts_with("ey") && word.len() > 30) // JWT header prefix 'ey...'
        {
            output_words.push("[REDACTED_KEY]");
        } else if lower.contains("api_key=") || lower.contains("apikey=") || lower.contains("secret=") {
            output_words.push("[REDACTED_PARAM]");
        } else {
            output_words.push(word);
        }
    }

    output_words.join(" ")
}
