use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use rusqlite::{
    backup::Backup, params, Connection, OptionalExtension, Row, Transaction, TransactionBehavior,
};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{
    AppSettings, ApplicationProject, CreateProjectInput, Material, ModuleFile, ProjectModule,
    ProjectTemplate, TemplateModule,
};

const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone)]
pub struct BlobRecord {
    pub id: String,
    pub relative_path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub page_count: u32,
}

#[derive(Debug, Clone)]
pub struct Storage {
    root: PathBuf,
    db_path: PathBuf,
    blobs_dir: PathBuf,
    incoming_dir: PathBuf,
    backups_dir: PathBuf,
}

impl Storage {
    pub fn initialize(root: PathBuf) -> AppResult<Self> {
        let storage = Self {
            db_path: root.join("data.db"),
            blobs_dir: root.join("blobs"),
            incoming_dir: root.join("incoming"),
            backups_dir: root.join("backups"),
            root,
        };
        fs::create_dir_all(&storage.root)?;
        fs::create_dir_all(&storage.blobs_dir)?;
        fs::create_dir_all(&storage.incoming_dir)?;
        fs::create_dir_all(&storage.backups_dir)?;
        storage.backup_before_migration()?;
        storage.migrate()?;
        storage.cleanup_incoming()?;
        Ok(storage)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn incoming_dir(&self) -> &Path {
        &self.incoming_dir
    }

    pub fn blob_destination(&self, sha256: &str) -> AppResult<(PathBuf, String)> {
        if sha256.len() < 2 || !sha256.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(AppError::Path("材料哈希格式无效".to_string()));
        }
        let prefix = &sha256[..2];
        let directory = self.blobs_dir.join(prefix);
        fs::create_dir_all(&directory)?;
        let relative = format!("blobs/{prefix}/{sha256}.pdf");
        Ok((self.root.join(&relative), relative))
    }

    fn connect(&self) -> AppResult<Connection> {
        let connection = Connection::open(&self.db_path)?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;
             PRAGMA busy_timeout = 5000;",
        )?;
        Ok(connection)
    }

    fn backup_before_migration(&self) -> AppResult<()> {
        if !self.db_path.exists() {
            return Ok(());
        }
        let connection = self.connect()?;
        let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version >= SCHEMA_VERSION {
            return Ok(());
        }
        let stamp = Utc::now().format("%Y%m%d-%H%M%S");
        let suffix = Uuid::new_v4();
        let backup_path = self
            .backups_dir
            .join(format!("data-v{version}-{stamp}-{suffix}.db"));
        let temporary_path = self
            .backups_dir
            .join(format!(".data-v{version}-{stamp}-{suffix}.part"));
        let result = (|| -> AppResult<()> {
            {
                let mut destination = Connection::open(&temporary_path)?;
                let backup = Backup::new(&connection, &mut destination)?;
                backup.run_to_completion(128, Duration::from_millis(10), None)?;
            }
            fs::rename(&temporary_path, &backup_path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }

    fn migrate(&self) -> AppResult<()> {
        let connection = self.connect()?;
        let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version > SCHEMA_VERSION {
            return Err(AppError::Database(rusqlite::Error::InvalidQuery));
        }
        if version == 0 {
            connection.execute_batch(
                "BEGIN IMMEDIATE;
                 CREATE TABLE blobs (
                   id TEXT PRIMARY KEY,
                   relative_path TEXT NOT NULL UNIQUE,
                   sha256 TEXT NOT NULL UNIQUE,
                   size_bytes INTEGER NOT NULL CHECK(size_bytes >= 0),
                   page_count INTEGER NOT NULL CHECK(page_count > 0),
                   created_at TEXT NOT NULL
                 );
                 CREATE TABLE materials (
                   id TEXT PRIMARY KEY,
                   name TEXT NOT NULL,
                   category TEXT NOT NULL,
                   original_name TEXT NOT NULL,
                   blob_id TEXT NOT NULL REFERENCES blobs(id) ON DELETE RESTRICT,
                   scope TEXT NOT NULL CHECK(scope IN ('library', 'project')),
                   scope_project_id TEXT,
                   status TEXT NOT NULL DEFAULT 'ready',
                   warning TEXT,
                   created_at TEXT NOT NULL,
                   updated_at TEXT NOT NULL
                 );
                 CREATE INDEX idx_materials_scope ON materials(scope, scope_project_id);
                 CREATE TABLE projects (
                   id TEXT PRIMARY KEY,
                   school TEXT NOT NULL DEFAULT '',
                   department TEXT NOT NULL DEFAULT '',
                   program TEXT NOT NULL DEFAULT '',
                   notice_url TEXT NOT NULL DEFAULT '',
                   deadline TEXT,
                   size_limit_mb REAL,
                   notes TEXT NOT NULL DEFAULT '',
                   created_at TEXT NOT NULL,
                   updated_at TEXT NOT NULL,
                   last_export_path TEXT,
                   last_exported_at TEXT
                 );
                 CREATE TABLE project_modules (
                   id TEXT PRIMARY KEY,
                   project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                   title TEXT NOT NULL,
                   category TEXT NOT NULL,
                   position INTEGER NOT NULL,
                   required INTEGER NOT NULL DEFAULT 0,
                   enabled INTEGER NOT NULL DEFAULT 1,
                   UNIQUE(project_id, position)
                 );
                 CREATE TABLE project_module_files (
                   id TEXT PRIMARY KEY,
                   module_id TEXT NOT NULL REFERENCES project_modules(id) ON DELETE CASCADE,
                   material_id TEXT NOT NULL REFERENCES materials(id) ON DELETE RESTRICT,
                   position INTEGER NOT NULL,
                   UNIQUE(module_id, position),
                   UNIQUE(module_id, material_id)
                 );
                 CREATE INDEX idx_project_files_material ON project_module_files(material_id);
                 CREATE TABLE templates (
                   id TEXT PRIMARY KEY,
                   name TEXT NOT NULL,
                   description TEXT NOT NULL DEFAULT '',
                   created_at TEXT NOT NULL,
                   updated_at TEXT NOT NULL
                 );
                 CREATE TABLE template_modules (
                   id TEXT PRIMARY KEY,
                   template_id TEXT NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
                   title TEXT NOT NULL,
                   category TEXT NOT NULL,
                   position INTEGER NOT NULL,
                   required INTEGER NOT NULL DEFAULT 0,
                   enabled INTEGER NOT NULL DEFAULT 1,
                   UNIQUE(template_id, position)
                 );
                 CREATE TABLE template_module_files (
                   id TEXT PRIMARY KEY,
                   module_id TEXT NOT NULL REFERENCES template_modules(id) ON DELETE CASCADE,
                   material_id TEXT REFERENCES materials(id) ON DELETE SET NULL,
                   placeholder_name TEXT,
                   position INTEGER NOT NULL,
                   UNIQUE(module_id, position)
                 );
                 CREATE TABLE settings (
                   key TEXT PRIMARY KEY,
                   value TEXT NOT NULL
                 );
                 PRAGMA user_version = 1;
                 COMMIT;",
            )?;
        }
        Ok(())
    }

    fn cleanup_incoming(&self) -> AppResult<()> {
        for entry in fs::read_dir(&self.incoming_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let _ = fs::remove_file(entry.path());
            }
        }
        Ok(())
    }

    pub fn settings(&self) -> AppResult<AppSettings> {
        let connection = self.connect()?;
        let value: Option<String> = connection
            .query_row(
                "SELECT value FROM settings WHERE key = 'app_settings'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        match value {
            Some(json) => Ok(serde_json::from_str(&json).unwrap_or_default()),
            None => Ok(AppSettings::default()),
        }
    }

    pub fn update_settings(&self, settings: &AppSettings) -> AppResult<()> {
        let connection = self.connect()?;
        let json = serde_json::to_string(settings)?;
        connection.execute(
            "INSERT INTO settings(key, value) VALUES('app_settings', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [json],
        )?;
        Ok(())
    }

    pub fn materials(&self) -> AppResult<Vec<Material>> {
        let connection = self.connect()?;
        let mut statement = connection.prepare(
            "SELECT m.id, m.name, m.category, m.original_name,
                    b.size_bytes, b.page_count, b.sha256,
                    m.scope, m.scope_project_id, m.created_at, m.updated_at,
                    (SELECT COUNT(*) FROM project_module_files f WHERE f.material_id = m.id),
                    m.status, m.warning
             FROM materials m
             JOIN blobs b ON b.id = m.blob_id
             ORDER BY m.updated_at DESC",
        )?;
        let rows = statement.query_map([], material_from_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn material(&self, id: &str) -> AppResult<Material> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT m.id, m.name, m.category, m.original_name,
                        b.size_bytes, b.page_count, b.sha256,
                        m.scope, m.scope_project_id, m.created_at, m.updated_at,
                        (SELECT COUNT(*) FROM project_module_files f WHERE f.material_id = m.id),
                        m.status, m.warning
                 FROM materials m JOIN blobs b ON b.id = m.blob_id WHERE m.id = ?1",
                [id],
                material_from_row,
            )
            .optional()?
            .ok_or_else(|| AppError::NotFound(format!("材料 {id}")))
    }

    pub fn find_blob_by_hash(&self, sha256: &str) -> AppResult<Option<BlobRecord>> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT id, relative_path, sha256, size_bytes, page_count FROM blobs WHERE sha256 = ?1",
                [sha256],
                |row| {
                    Ok(BlobRecord {
                        id: row.get(0)?,
                        relative_path: row.get(1)?,
                        sha256: row.get(2)?,
                        size_bytes: row.get::<_, i64>(3)? as u64,
                        page_count: row.get::<_, i64>(4)? as u32,
                    })
                },
            )
            .optional()
            .map_err(AppError::from)
    }

    pub fn insert_blob(&self, blob: &BlobRecord) -> AppResult<()> {
        let connection = self.connect()?;
        connection.execute(
            "INSERT OR IGNORE INTO blobs(id, relative_path, sha256, size_bytes, page_count, created_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                blob.id,
                blob.relative_path,
                blob.sha256,
                blob.size_bytes as i64,
                blob.page_count as i64,
                now()
            ],
        )?;
        Ok(())
    }

    pub fn remove_blob_if_unreferenced(&self, blob_id: &str) -> AppResult<Option<PathBuf>> {
        let mut connection = self.connect()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let blob: Option<String> = transaction
            .query_row(
                "SELECT relative_path FROM blobs
                 WHERE id = ?1 AND NOT EXISTS(SELECT 1 FROM materials WHERE blob_id = ?1)",
                [blob_id],
                |row| row.get(0),
            )
            .optional()?;
        if blob.is_some() {
            transaction.execute("DELETE FROM blobs WHERE id = ?1", [blob_id])?;
        }
        transaction.commit()?;
        Ok(blob.map(|relative| self.root.join(relative)))
    }

    pub fn insert_material(
        &self,
        name: &str,
        category: &str,
        original_name: &str,
        blob_id: &str,
        scope_project_id: Option<&str>,
    ) -> AppResult<Material> {
        let mut connection = self.connect()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        let scope = if let Some(project_id) = scope_project_id {
            let project_exists: i64 = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
                [project_id],
                |row| row.get(0),
            )?;
            if project_exists == 0 {
                return Err(AppError::NotFound(format!("申请项目 {project_id}")));
            }
            "project"
        } else {
            "library"
        };
        transaction.execute(
            "INSERT INTO materials(id, name, category, original_name, blob_id, scope, scope_project_id, status, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, 'ready', ?8, ?8)",
            params![id, name, category, original_name, blob_id, scope, scope_project_id, timestamp],
        )?;
        transaction.commit()?;
        self.material(&id)
    }

    pub fn resolve_material_path(&self, material_id: &str) -> AppResult<PathBuf> {
        let connection = self.connect()?;
        let relative: String = connection
            .query_row(
                "SELECT b.relative_path FROM materials m JOIN blobs b ON b.id = m.blob_id WHERE m.id = ?1",
                [material_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| AppError::NotFound(format!("材料文件 {material_id}")))?;
        let path = self.root.join(relative);
        if !path.exists() {
            return Err(AppError::NotFound(format!("材料文件已缺失：{material_id}")));
        }
        Ok(path)
    }

    pub fn rename_material(&self, material_id: &str, name: &str) -> AppResult<Material> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::User("材料名称不能为空".to_string()));
        }
        let connection = self.connect()?;
        let changed = connection.execute(
            "UPDATE materials SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![material_id, trimmed, now()],
        )?;
        if changed == 0 {
            return Err(AppError::NotFound(format!("材料 {material_id}")));
        }
        self.material(material_id)
    }

    pub fn delete_material(&self, material_id: &str) -> AppResult<Option<PathBuf>> {
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let refs: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM project_module_files WHERE material_id = ?1",
            [material_id],
            |row| row.get(0),
        )?;
        let template_refs: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM template_module_files WHERE material_id = ?1",
            [material_id],
            |row| row.get(0),
        )?;
        if refs > 0 || template_refs > 0 {
            return Err(AppError::Conflict(format!(
                "该材料仍被 {refs} 个项目模块和 {template_refs} 个模板条目使用，请先移除相关引用"
            )));
        }
        let blob: Option<(String, String)> = transaction
            .query_row(
                "SELECT b.id, b.relative_path FROM materials m JOIN blobs b ON b.id = m.blob_id WHERE m.id = ?1",
                [material_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((blob_id, relative)) = blob else {
            return Err(AppError::NotFound(format!("材料 {material_id}")));
        };
        transaction.execute("DELETE FROM materials WHERE id = ?1", [material_id])?;
        let remaining: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM materials WHERE blob_id = ?1",
            [&blob_id],
            |row| row.get(0),
        )?;
        let orphan = if remaining == 0 {
            transaction.execute("DELETE FROM blobs WHERE id = ?1", [&blob_id])?;
            Some(self.root.join(relative))
        } else {
            None
        };
        transaction.commit()?;
        Ok(orphan)
    }

    pub fn projects(&self) -> AppResult<Vec<ApplicationProject>> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare("SELECT id FROM projects ORDER BY updated_at DESC, created_at DESC")?;
        let ids = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        ids.into_iter()
            .map(|id| load_project(&connection, &id))
            .collect()
    }

    pub fn project(&self, project_id: &str) -> AppResult<ApplicationProject> {
        let connection = self.connect()?;
        load_project(&connection, project_id)
    }

    pub fn create_project(&self, input: &CreateProjectInput) -> AppResult<ApplicationProject> {
        validate_project_name(&input.school, &input.department, &input.program)?;
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let project_id = Uuid::new_v4().to_string();
        let timestamp = now();
        transaction.execute(
            "INSERT INTO projects(id, school, department, program, notice_url, deadline, size_limit_mb, notes, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![
                project_id,
                input.school.trim(),
                input.department.trim(),
                input.program.trim(),
                input.notice_url.trim(),
                input.deadline,
                input.size_limit_mb,
                input.notes.trim(),
                timestamp
            ],
        )?;
        if let Some(template_id) = input.template_id.as_deref() {
            copy_template_into_project(&transaction, template_id, &project_id)?;
        }
        transaction.commit()?;
        self.project(&project_id)
    }

    pub fn update_project(&self, project: &ApplicationProject) -> AppResult<ApplicationProject> {
        validate_project_name(&project.school, &project.department, &project.program)?;
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            "UPDATE projects SET school = ?2, department = ?3, program = ?4, notice_url = ?5,
                    deadline = ?6, size_limit_mb = ?7, notes = ?8, updated_at = ?9
             WHERE id = ?1",
            params![
                project.id,
                project.school.trim(),
                project.department.trim(),
                project.program.trim(),
                project.notice_url.trim(),
                project.deadline,
                project.size_limit_mb,
                project.notes,
                now()
            ],
        )?;
        if changed == 0 {
            return Err(AppError::NotFound(format!("申请项目 {}", project.id)));
        }
        transaction.execute(
            "DELETE FROM project_modules WHERE project_id = ?1",
            [&project.id],
        )?;
        insert_project_modules(&transaction, &project.id, &project.modules)?;
        transaction.commit()?;
        self.project(&project.id)
    }

    pub fn delete_project(&self, project_id: &str) -> AppResult<Vec<PathBuf>> {
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let scoped = {
            let mut statement = transaction.prepare(
                "SELECT m.id, b.id, b.relative_path
                 FROM materials m JOIN blobs b ON b.id = m.blob_id
                 WHERE m.scope = 'project' AND m.scope_project_id = ?1",
            )?;
            let result = statement
                .query_map([project_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            result
        };
        let changed = transaction.execute("DELETE FROM projects WHERE id = ?1", [project_id])?;
        if changed == 0 {
            return Err(AppError::NotFound(format!("申请项目 {project_id}")));
        }
        let mut orphan_paths = Vec::new();
        for (material_id, blob_id, relative) in scoped {
            let refs: i64 = transaction.query_row(
                "SELECT COUNT(*) FROM project_module_files WHERE material_id = ?1",
                [&material_id],
                |row| row.get(0),
            )?;
            if refs == 0 {
                transaction.execute("DELETE FROM materials WHERE id = ?1", [&material_id])?;
                let remaining: i64 = transaction.query_row(
                    "SELECT COUNT(*) FROM materials WHERE blob_id = ?1",
                    [&blob_id],
                    |row| row.get(0),
                )?;
                if remaining == 0 {
                    transaction.execute("DELETE FROM blobs WHERE id = ?1", [&blob_id])?;
                    orphan_paths.push(self.root.join(relative));
                }
            }
        }
        transaction.commit()?;
        Ok(orphan_paths)
    }

    pub fn duplicate_project(&self, project_id: &str) -> AppResult<ApplicationProject> {
        let source = self.project(project_id)?;
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let new_id = Uuid::new_v4().to_string();
        let timestamp = now();
        let program = if source.program.trim().is_empty() {
            "申请项目（副本）".to_string()
        } else {
            format!("{}（副本）", source.program)
        };
        transaction.execute(
            "INSERT INTO projects(id, school, department, program, notice_url, deadline, size_limit_mb, notes, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![new_id, source.school, source.department, program, source.notice_url, source.deadline, source.size_limit_mb, source.notes, timestamp],
        )?;
        let mut material_map = HashMap::<String, String>::new();
        for module in &source.modules {
            let new_module_id = Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO project_modules(id, project_id, title, category, position, required, enabled)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![new_module_id, new_id, module.title, module.category, module.position, module.required as i64, module.enabled as i64],
            )?;
            for file in &module.files {
                let target_material_id = if file.material.scope == "project" {
                    if let Some(mapped) = material_map.get(&file.material_id) {
                        mapped.clone()
                    } else {
                        let clone_id = Uuid::new_v4().to_string();
                        transaction.execute(
                            "INSERT INTO materials(id, name, category, original_name, blob_id, scope, scope_project_id, status, warning, created_at, updated_at)
                             SELECT ?1, name, category, original_name, blob_id, 'project', ?2, status, warning, ?3, ?3
                             FROM materials WHERE id = ?4",
                            params![clone_id, new_id, timestamp, file.material_id],
                        )?;
                        material_map.insert(file.material_id.clone(), clone_id.clone());
                        clone_id
                    }
                } else {
                    file.material_id.clone()
                };
                transaction.execute(
                    "INSERT INTO project_module_files(id, module_id, material_id, position) VALUES(?1, ?2, ?3, ?4)",
                    params![Uuid::new_v4().to_string(), new_module_id, target_material_id, file.position],
                )?;
            }
        }
        transaction.commit()?;
        self.project(&new_id)
    }

    pub fn templates(&self) -> AppResult<Vec<ProjectTemplate>> {
        let connection = self.connect()?;
        let mut statement = connection.prepare(
            "SELECT id, name, description, created_at, updated_at FROM templates ORDER BY updated_at DESC",
        )?;
        let templates = statement
            .query_map([], |row| {
                Ok(ProjectTemplate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    modules: Vec::new(),
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        templates
            .into_iter()
            .map(|mut template| {
                template.modules = load_template_modules(&connection, &template.id)?;
                Ok(template)
            })
            .collect()
    }

    pub fn save_project_as_template(
        &self,
        project_id: &str,
        name: &str,
        description: &str,
    ) -> AppResult<ProjectTemplate> {
        if name.trim().is_empty() {
            return Err(AppError::User("模板名称不能为空".to_string()));
        }
        let project = self.project(project_id)?;
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        transaction.execute(
            "INSERT INTO templates(id, name, description, created_at, updated_at) VALUES(?1, ?2, ?3, ?4, ?4)",
            params![id, name.trim(), description.trim(), timestamp],
        )?;
        for module in &project.modules {
            let module_id = Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO template_modules(id, template_id, title, category, position, required, enabled)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![module_id, id, module.title, module.category, module.position, module.required as i64, module.enabled as i64],
            )?;
            for file in &module.files {
                let material_id =
                    (file.material.scope == "library").then_some(file.material_id.as_str());
                let placeholder = Some(file.material.name.as_str());
                transaction.execute(
                    "INSERT INTO template_module_files(id, module_id, material_id, placeholder_name, position)
                     VALUES(?1, ?2, ?3, ?4, ?5)",
                    params![Uuid::new_v4().to_string(), module_id, material_id, placeholder, file.position],
                )?;
            }
        }
        transaction.commit()?;
        self.templates()?
            .into_iter()
            .find(|template| template.id == id)
            .ok_or_else(|| AppError::NotFound(format!("模板 {id}")))
    }

    pub fn delete_template(&self, template_id: &str) -> AppResult<()> {
        let connection = self.connect()?;
        let changed = connection.execute("DELETE FROM templates WHERE id = ?1", [template_id])?;
        if changed == 0 {
            return Err(AppError::NotFound(format!("模板 {template_id}")));
        }
        Ok(())
    }

    pub fn update_export_record(&self, project_id: &str, output_path: &str) -> AppResult<()> {
        let timestamp = now();
        let connection = self.connect()?;
        connection.execute(
            "UPDATE projects SET last_export_path = ?2, last_exported_at = ?3, updated_at = ?3 WHERE id = ?1",
            params![project_id, output_path, timestamp],
        )?;
        Ok(())
    }
}

fn material_from_row(row: &Row<'_>) -> rusqlite::Result<Material> {
    Ok(Material {
        id: row.get(0)?,
        name: row.get(1)?,
        category: row.get(2)?,
        original_name: row.get(3)?,
        size_bytes: row.get::<_, i64>(4)? as u64,
        page_count: row.get::<_, i64>(5)? as u32,
        sha256: row.get(6)?,
        scope: row.get(7)?,
        scope_project_id: row.get(8)?,
        stored_path: None,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        used_by_count: row.get::<_, i64>(11)? as u32,
        status: row.get(12)?,
        warning: row.get(13)?,
    })
}

fn load_project(connection: &Connection, project_id: &str) -> AppResult<ApplicationProject> {
    let mut project = connection
        .query_row(
            "SELECT id, school, department, program, notice_url, deadline, size_limit_mb, notes,
                    created_at, updated_at, last_export_path, last_exported_at
             FROM projects WHERE id = ?1",
            [project_id],
            |row| {
                Ok(ApplicationProject {
                    id: row.get(0)?,
                    school: row.get(1)?,
                    department: row.get(2)?,
                    program: row.get(3)?,
                    notice_url: row.get(4)?,
                    deadline: row.get(5)?,
                    size_limit_mb: row.get(6)?,
                    notes: row.get(7)?,
                    modules: Vec::new(),
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    last_export_path: row.get(10)?,
                    last_exported_at: row.get(11)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("申请项目 {project_id}")))?;

    let mut module_statement = connection.prepare(
        "SELECT id, title, category, position, required, enabled
         FROM project_modules WHERE project_id = ?1 ORDER BY position",
    )?;
    let mut modules = module_statement
        .query_map([project_id], |row| {
            Ok(ProjectModule {
                id: row.get(0)?,
                title: row.get(1)?,
                category: row.get(2)?,
                position: row.get(3)?,
                required: row.get::<_, i64>(4)? != 0,
                enabled: row.get::<_, i64>(5)? != 0,
                files: Vec::new(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for module in &mut modules {
        let mut file_statement = connection.prepare(
            "SELECT f.id, f.material_id, f.position,
                    m.id, m.name, m.category, m.original_name,
                    b.size_bytes, b.page_count, b.sha256,
                    m.scope, m.scope_project_id, m.created_at, m.updated_at,
                    (SELECT COUNT(*) FROM project_module_files f2 WHERE f2.material_id = m.id),
                    m.status, m.warning
             FROM project_module_files f
             JOIN materials m ON m.id = f.material_id
             JOIN blobs b ON b.id = m.blob_id
             WHERE f.module_id = ?1 ORDER BY f.position",
        )?;
        module.files = file_statement
            .query_map([&module.id], |row| {
                Ok(ModuleFile {
                    id: row.get(0)?,
                    material_id: row.get(1)?,
                    position: row.get(2)?,
                    material: Material {
                        id: row.get(3)?,
                        name: row.get(4)?,
                        category: row.get(5)?,
                        original_name: row.get(6)?,
                        size_bytes: row.get::<_, i64>(7)? as u64,
                        page_count: row.get::<_, i64>(8)? as u32,
                        sha256: row.get(9)?,
                        scope: row.get(10)?,
                        scope_project_id: row.get(11)?,
                        stored_path: None,
                        created_at: row.get(12)?,
                        updated_at: row.get(13)?,
                        used_by_count: row.get::<_, i64>(14)? as u32,
                        status: row.get(15)?,
                        warning: row.get(16)?,
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
    }
    project.modules = modules;
    Ok(project)
}

fn insert_project_modules(
    transaction: &Transaction<'_>,
    project_id: &str,
    modules: &[ProjectModule],
) -> AppResult<()> {
    for (module_index, module) in modules.iter().enumerate() {
        let module_id = if module.id.trim().is_empty() {
            Uuid::new_v4().to_string()
        } else {
            module.id.clone()
        };
        transaction.execute(
            "INSERT INTO project_modules(id, project_id, title, category, position, required, enabled)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![module_id, project_id, module.title.trim(), module.category.trim(), module_index as i64, module.required as i64, module.enabled as i64],
        )?;
        for (file_index, file) in module.files.iter().enumerate() {
            let allowed: i64 = transaction.query_row(
                "SELECT COUNT(*) FROM materials
                 WHERE id = ?1 AND (scope = 'library' OR scope_project_id = ?2)",
                params![file.material_id, project_id],
                |row| row.get(0),
            )?;
            if allowed == 0 {
                return Err(AppError::Conflict(format!(
                    "材料 {} 不属于材料库或当前项目",
                    file.material_id
                )));
            }
            transaction.execute(
                "INSERT INTO project_module_files(id, module_id, material_id, position) VALUES(?1, ?2, ?3, ?4)",
                params![
                    if file.id.trim().is_empty() { Uuid::new_v4().to_string() } else { file.id.clone() },
                    module_id,
                    file.material_id,
                    file_index as i64
                ],
            )?;
        }
    }
    Ok(())
}

fn copy_template_into_project(
    transaction: &Transaction<'_>,
    template_id: &str,
    project_id: &str,
) -> AppResult<()> {
    let template_exists: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM templates WHERE id = ?1",
        [template_id],
        |row| row.get(0),
    )?;
    if template_exists == 0 {
        return Err(AppError::NotFound(format!("模板 {template_id}")));
    }
    let template_modules = {
        let mut statement = transaction.prepare(
            "SELECT id, title, category, position, required, enabled
             FROM template_modules WHERE template_id = ?1 ORDER BY position",
        )?;
        let result = statement
            .query_map([template_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        result
    };
    for (template_module_id, title, category, position, required, enabled) in template_modules {
        let module_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO project_modules(id, project_id, title, category, position, required, enabled)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![module_id, project_id, title, category, position, required, enabled],
        )?;
        let material_ids = {
            let mut statement = transaction.prepare(
                "SELECT material_id, position FROM template_module_files
                 WHERE module_id = ?1 AND material_id IS NOT NULL ORDER BY position",
            )?;
            let result = statement
                .query_map([&template_module_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            result
        };
        for (material_id, file_position) in material_ids {
            transaction.execute(
                "INSERT INTO project_module_files(id, module_id, material_id, position) VALUES(?1, ?2, ?3, ?4)",
                params![Uuid::new_v4().to_string(), module_id, material_id, file_position],
            )?;
        }
    }
    Ok(())
}

fn load_template_modules(
    connection: &Connection,
    template_id: &str,
) -> AppResult<Vec<TemplateModule>> {
    let mut statement = connection.prepare(
        "SELECT id, title, category, position, required, enabled
         FROM template_modules WHERE template_id = ?1 ORDER BY position",
    )?;
    let mut modules = statement
        .query_map([template_id], |row| {
            Ok(TemplateModule {
                id: row.get(0)?,
                title: row.get(1)?,
                category: row.get(2)?,
                position: row.get(3)?,
                required: row.get::<_, i64>(4)? != 0,
                enabled: row.get::<_, i64>(5)? != 0,
                material_ids: Vec::new(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for module in &mut modules {
        let mut files = connection.prepare(
            "SELECT material_id FROM template_module_files
             WHERE module_id = ?1 AND material_id IS NOT NULL ORDER BY position",
        )?;
        module.material_ids = files
            .query_map([&module.id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
    }
    Ok(modules)
}

fn validate_project_name(school: &str, department: &str, program: &str) -> AppResult<()> {
    if school.trim().is_empty() && department.trim().is_empty() && program.trim().is_empty() {
        return Err(AppError::User(
            "学校、学院或项目名称至少填写一项".to_string(),
        ));
    }
    Ok(())
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn storage() -> (tempfile::TempDir, Storage) {
        let directory = tempfile::tempdir().expect("temporary directory");
        let storage = Storage::initialize(directory.path().join("data"))
            .expect("initialize temporary storage");
        (directory, storage)
    }

    fn create_project(storage: &Storage) -> ApplicationProject {
        storage
            .create_project(&CreateProjectInput {
                school: "测试大学".to_string(),
                department: "人工智能学院".to_string(),
                program: "预推免".to_string(),
                notice_url: "https://example.edu/notice".to_string(),
                deadline: None,
                size_limit_mb: Some(20.0),
                notes: String::new(),
                template_id: None,
            })
            .expect("create project")
    }

    fn create_material(storage: &Storage) -> Material {
        let blob = BlobRecord {
            id: Uuid::new_v4().to_string(),
            relative_path: "blobs/aa/test.pdf".to_string(),
            sha256: "aa00000000000000000000000000000000000000000000000000000000000000".to_string(),
            size_bytes: 128,
            page_count: 1,
        };
        storage.insert_blob(&blob).expect("insert blob");
        storage
            .insert_material("测试成绩单", "本科成绩单", "成绩单.pdf", &blob.id, None)
            .expect("insert material")
    }

    #[test]
    fn ordinary_project_save_preserves_export_record() {
        let (_directory, storage) = storage();
        let mut project = create_project(&storage);
        storage
            .update_export_record(&project.id, r"C:\Exports\申请材料.pdf")
            .expect("record export");
        project.notes = "导师志愿已确认".to_string();

        storage
            .update_project(&project)
            .expect("save project changes");
        let saved = storage.project(&project.id).expect("reload project");

        assert_eq!(saved.notes, "导师志愿已确认");
        assert_eq!(
            saved.last_export_path.as_deref(),
            Some(r"C:\Exports\申请材料.pdf")
        );
        assert!(saved.last_exported_at.is_some());
    }

    #[test]
    fn template_reference_blocks_material_deletion() {
        let (_directory, storage) = storage();
        let material = create_material(&storage);
        let mut project = create_project(&storage);
        project.modules = vec![ProjectModule {
            id: Uuid::new_v4().to_string(),
            title: "本科成绩单".to_string(),
            category: "本科成绩单".to_string(),
            position: 0,
            required: true,
            enabled: true,
            files: vec![ModuleFile {
                id: Uuid::new_v4().to_string(),
                material_id: material.id.clone(),
                position: 0,
                material: material.clone(),
            }],
        }];
        let mut project = storage.update_project(&project).expect("attach material");
        storage
            .save_project_as_template(&project.id, "标准材料", "回归测试")
            .expect("save template");
        project.modules.clear();
        storage
            .update_project(&project)
            .expect("remove project reference");

        let result = storage.delete_material(&material.id);

        assert!(matches!(result, Err(AppError::Conflict(_))));
        assert!(storage.material(&material.id).is_ok());
    }

    #[test]
    fn unreferenced_blob_cleanup_removes_database_record() {
        let (_directory, storage) = storage();
        let blob = BlobRecord {
            id: Uuid::new_v4().to_string(),
            relative_path: "blobs/bb/orphan.pdf".to_string(),
            sha256: "bb00000000000000000000000000000000000000000000000000000000000000".to_string(),
            size_bytes: 64,
            page_count: 1,
        };
        storage.insert_blob(&blob).expect("insert orphan blob");

        let path = storage
            .remove_blob_if_unreferenced(&blob.id)
            .expect("clean orphan blob");

        assert_eq!(path, Some(storage.root().join(&blob.relative_path)));
        assert!(storage
            .find_blob_by_hash(&blob.sha256)
            .expect("query blob")
            .is_none());
    }
}
