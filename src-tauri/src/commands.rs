use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sha2::{Digest, Sha256};
use tauri::State;
use uuid::Uuid;

use crate::ai;
use crate::db::{BlobRecord, Storage};
use crate::error::{AppError, AppResult};
use crate::models::{
    AiConnectionResult, AiProviderConfig, AnalyzeRegistrationNoticeInput,
    AnalyzeRegistrationNoticeResult, AppSettings, AppSnapshot, ApplicationProject,
    CreateProjectInput, ExportProjectInput, ExportResult, ImportFailure, ImportMaterialsInput,
    ImportResult, Material, ProjectTemplate,
};
use crate::qpdf::{looks_like_pdf, QpdfEngine};

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    pub qpdf: Arc<QpdfEngine>,
    pub app_version: String,
}

#[tauri::command]
pub fn get_app_snapshot(state: State<'_, AppState>) -> AppResult<AppSnapshot> {
    Ok(AppSnapshot {
        materials: state.storage.materials()?,
        projects: state.storage.projects()?,
        templates: state.storage.templates()?,
        settings: state.storage.settings()?,
        app_version: state.app_version.clone(),
        data_directory: state.storage.root().to_string_lossy().into_owned(),
        qpdf_version: state.qpdf.version(),
    })
}

#[tauri::command]
pub fn create_project(
    input: CreateProjectInput,
    state: State<'_, AppState>,
) -> AppResult<ApplicationProject> {
    state.storage.create_project(&input)
}

#[tauri::command]
pub fn update_project(
    project: ApplicationProject,
    state: State<'_, AppState>,
) -> AppResult<ApplicationProject> {
    state.storage.update_project(&project)
}

#[tauri::command]
pub fn delete_project(project_id: String, state: State<'_, AppState>) -> AppResult<()> {
    let orphans = state.storage.delete_project(&project_id)?;
    for path in orphans {
        let _ = fs::remove_file(path);
    }
    Ok(())
}

#[tauri::command]
pub fn duplicate_project(
    project_id: String,
    state: State<'_, AppState>,
) -> AppResult<ApplicationProject> {
    state.storage.duplicate_project(&project_id)
}

#[tauri::command]
pub async fn import_materials(
    input: ImportMaterialsInput,
    state: State<'_, AppState>,
) -> AppResult<ImportResult> {
    let storage = Arc::clone(&state.storage);
    let qpdf = Arc::clone(&state.qpdf);
    tauri::async_runtime::spawn_blocking(move || import_materials_blocking(&storage, &qpdf, input))
        .await
        .map_err(|error| AppError::User(format!("导入任务意外终止：{error}")))?
}

#[tauri::command]
pub fn rename_material(
    material_id: String,
    name: String,
    state: State<'_, AppState>,
) -> AppResult<Material> {
    state.storage.rename_material(&material_id, &name)
}

#[tauri::command]
pub fn delete_material(material_id: String, state: State<'_, AppState>) -> AppResult<()> {
    if let Some(orphan) = state.storage.delete_material(&material_id)? {
        let _ = fs::remove_file(orphan);
    }
    Ok(())
}

#[tauri::command]
pub fn save_project_as_template(
    project_id: String,
    name: String,
    description: String,
    state: State<'_, AppState>,
) -> AppResult<ProjectTemplate> {
    state
        .storage
        .save_project_as_template(&project_id, &name, &description)
}

#[tauri::command]
pub fn delete_template(template_id: String, state: State<'_, AppState>) -> AppResult<()> {
    state.storage.delete_template(&template_id)
}

#[tauri::command]
pub async fn export_project(
    input: ExportProjectInput,
    state: State<'_, AppState>,
) -> AppResult<ExportResult> {
    let storage = Arc::clone(&state.storage);
    let qpdf = Arc::clone(&state.qpdf);
    tauri::async_runtime::spawn_blocking(move || export_project_blocking(&storage, &qpdf, input))
        .await
        .map_err(|error| AppError::User(format!("导出任务意外终止：{error}")))?
}

#[tauri::command]
pub fn get_material_preview_path(
    material_id: String,
    state: State<'_, AppState>,
) -> AppResult<String> {
    Ok(state
        .storage
        .resolve_material_path(&material_id)?
        .to_string_lossy()
        .into_owned())
}

#[tauri::command]
pub fn update_settings(settings: AppSettings, state: State<'_, AppState>) -> AppResult<()> {
    state.storage.update_settings(&settings)
}

#[tauri::command]
pub async fn test_ai_connection(config: AiProviderConfig) -> AiConnectionResult {
    ai::test_connection(config).await
}

#[tauri::command]
pub async fn analyze_registration_notice(
    input: AnalyzeRegistrationNoticeInput,
) -> AppResult<AnalyzeRegistrationNoticeResult> {
    ai::analyze_registration_notice(input)
        .await
        .map_err(|error| AppError::User(error.to_string()))
}

#[tauri::command]
pub fn cancel_ai_analysis(request_id: String) -> bool {
    ai::cancel_analysis(&request_id)
}

fn import_materials_blocking(
    storage: &Storage,
    qpdf: &QpdfEngine,
    input: ImportMaterialsInput,
) -> AppResult<ImportResult> {
    if input.paths.is_empty() {
        return Err(AppError::User("请选择至少一个 PDF 文件".to_string()));
    }
    if input.category.trim().is_empty() {
        return Err(AppError::User("请选择材料分类".to_string()));
    }
    if input.category.chars().count() > 40 {
        return Err(AppError::User("材料名称不能超过 40 个字符".to_string()));
    }
    if input.category.chars().any(char::is_control) {
        return Err(AppError::User("材料名称包含不支持的字符".to_string()));
    }
    if let Some(project_id) = input.scope_project_id.as_deref() {
        storage.project(project_id)?;
    }
    let mut imported = Vec::new();
    let mut failures = Vec::new();
    for raw_path in input.paths {
        let path = PathBuf::from(&raw_path);
        match import_one(
            storage,
            qpdf,
            &path,
            &input.category,
            input.scope_project_id.as_deref(),
        ) {
            Ok(material) => imported.push(material),
            Err(error) => failures.push(ImportFailure {
                path: raw_path,
                reason: error.to_string(),
            }),
        }
    }
    Ok(ImportResult {
        imported,
        failures,
        duplicates: Vec::new(),
    })
}

fn import_one(
    storage: &Storage,
    qpdf: &QpdfEngine,
    source: &Path,
    category: &str,
    scope_project_id: Option<&str>,
) -> AppResult<Material> {
    let metadata_before = source
        .metadata()
        .map_err(|error| AppError::User(format!("无法读取 {}：{error}", source.display())))?;
    if !metadata_before.is_file() || metadata_before.len() == 0 {
        return Err(AppError::User("文件为空或不是普通文件".to_string()));
    }
    if source
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| !value.eq_ignore_ascii_case("pdf"))
        .unwrap_or(true)
    {
        return Err(AppError::User("仅支持 PDF 文件".to_string()));
    }
    if !looks_like_pdf(source)? {
        return Err(AppError::Pdf(
            "文件扩展名为 PDF，但内容不是有效 PDF".to_string(),
        ));
    }

    let temp = storage
        .incoming_dir()
        .join(format!("{}.part", Uuid::new_v4()));
    let copy_result = copy_and_hash(source, &temp);
    let (sha256, copied_size) = match copy_result {
        Ok(result) => result,
        Err(error) => {
            let _ = fs::remove_file(&temp);
            return Err(error);
        }
    };
    let metadata_after = source.metadata()?;
    if metadata_after.len() != metadata_before.len()
        || metadata_after.modified().ok() != metadata_before.modified().ok()
    {
        let _ = fs::remove_file(&temp);
        return Err(AppError::Conflict(
            "导入过程中源文件发生变化，请保存后重新导入".to_string(),
        ));
    }

    let (blob, created_blob) = if let Some(existing) = storage.find_blob_by_hash(&sha256)? {
        let _ = fs::remove_file(&temp);
        (existing, false)
    } else {
        let page_count = match qpdf.page_count(&temp) {
            Ok(value) => value,
            Err(error) => {
                let _ = fs::remove_file(&temp);
                return Err(error);
            }
        };
        let (destination, relative_path) = storage.blob_destination(&sha256)?;
        if !destination.exists() {
            fs::rename(&temp, &destination)?;
        } else {
            let _ = fs::remove_file(&temp);
        }
        let blob = BlobRecord {
            id: Uuid::new_v4().to_string(),
            relative_path,
            sha256,
            size_bytes: copied_size,
            page_count,
        };
        storage.insert_blob(&blob)?;
        let stored = storage
            .find_blob_by_hash(&blob.sha256)?
            .ok_or_else(|| AppError::Database(rusqlite::Error::QueryReturnedNoRows))?;
        let created = stored.id == blob.id;
        (stored, created)
    };

    let original_name = source
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("材料.pdf");
    let display_name = source
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("未命名材料");
    let inserted = storage.insert_material(
        display_name,
        category.trim(),
        original_name,
        &blob.id,
        scope_project_id,
    );
    if inserted.is_err() && created_blob {
        if let Ok(Some(path)) = storage.remove_blob_if_unreferenced(&blob.id) {
            let _ = fs::remove_file(path);
        }
    }
    inserted
}

fn copy_and_hash(source: &Path, target: &Path) -> AppResult<(String, u64)> {
    let source_file = File::open(source)?;
    let target_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    let mut reader = BufReader::with_capacity(1024 * 1024, source_file);
    let mut writer = BufWriter::with_capacity(1024 * 1024, target_file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    let mut total = 0_u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        writer.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
        total += read as u64;
    }
    writer.flush()?;
    writer.get_ref().sync_all()?;
    Ok((hex::encode(hasher.finalize()), total))
}

fn export_project_blocking(
    storage: &Storage,
    qpdf: &QpdfEngine,
    input: ExportProjectInput,
) -> AppResult<ExportResult> {
    let project = storage.project(&input.project_id)?;
    let mut paths = Vec::new();
    let mut expected_pages = 0_u32;
    let mut warnings = Vec::new();
    let missing_required = project
        .modules
        .iter()
        .filter(|module| module.required && (!module.enabled || module.files.is_empty()))
        .map(|module| module.title.clone())
        .collect::<Vec<_>>();
    if !missing_required.is_empty() {
        warnings.push(format!("缺少必需模块：{}", missing_required.join("、")));
    }
    for module in project.modules.iter().filter(|module| module.enabled) {
        for file in &module.files {
            if file.material.status != "ready" {
                return Err(AppError::Pdf(format!(
                    "材料“{}”当前不可用",
                    file.material.name
                )));
            }
            paths.push(storage.resolve_material_path(&file.material_id)?);
            expected_pages = expected_pages.saturating_add(file.material.page_count);
        }
    }
    if paths.is_empty() {
        return Err(AppError::User("当前项目没有启用的 PDF 材料".to_string()));
    }
    if !warnings.is_empty() && !input.allow_warnings {
        return Err(AppError::Conflict(warnings.join("；")));
    }

    let output = PathBuf::from(&input.output_path);
    let extension_ok = output
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false);
    if !extension_ok {
        return Err(AppError::User("输出文件必须使用 .pdf 扩展名".to_string()));
    }
    let parent = output
        .parent()
        .ok_or_else(|| AppError::Path("请选择有效的输出目录".to_string()))?;
    if !parent.is_dir() {
        return Err(AppError::Path("输出目录不存在".to_string()));
    }
    let managed_root = storage.root().canonicalize()?;
    let output_parent = parent.canonicalize()?;
    if output_parent.starts_with(&managed_root) {
        return Err(AppError::Path(
            "不能把导出文件保存到软件的数据目录，请选择“文档”或其他文件夹".to_string(),
        ));
    }
    if output.exists() && !output.is_file() {
        return Err(AppError::Path("输出位置不是普通文件".to_string()));
    }
    if output.exists() && !input.overwrite {
        return Err(AppError::Conflict(
            "目标文件已存在，请更换文件名或确认覆盖".to_string(),
        ));
    }
    let file_name = output
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("申请材料.pdf");
    let operation_id = Uuid::new_v4();
    let merged_temp = parent.join(format!(".{file_name}.{operation_id}.merge.tmp.pdf"));
    let compressed_temp = parent.join(format!(".{file_name}.{operation_id}.compress.tmp.pdf"));
    let result = (|| -> AppResult<ExportResult> {
        qpdf.merge(&paths, &merged_temp)?;
        let actual_pages = qpdf.page_count(&merged_temp)?;
        if actual_pages != expected_pages {
            return Err(AppError::Pdf(format!(
                "合并后页数校验失败：预计 {expected_pages} 页，实际 {actual_pages} 页"
            )));
        }
        let source_size_bytes = merged_temp.metadata()?.len();
        let (candidate, size_bytes, compression_applied) = if matches!(
            input.compression_level,
            crate::models::PdfCompressionLevel::None
        ) {
            (&merged_temp, source_size_bytes, false)
        } else {
            qpdf.optimize(&merged_temp, &compressed_temp, &input.compression_level)?;
            let compressed_pages = qpdf.page_count(&compressed_temp)?;
            if compressed_pages != expected_pages {
                return Err(AppError::Pdf(format!(
                    "压缩后页数校验失败：预计 {expected_pages} 页，实际 {compressed_pages} 页"
                )));
            }
            let compressed_size = compressed_temp.metadata()?.len();
            if compressed_size < source_size_bytes {
                (&compressed_temp, compressed_size, true)
            } else {
                (&merged_temp, source_size_bytes, false)
            }
        };
        commit_export(candidate, &output, input.overwrite)?;
        let exceeded_limit = project
            .size_limit_mb
            .map(|limit| size_bytes as f64 > limit * 1024.0 * 1024.0)
            .unwrap_or(false);
        if exceeded_limit {
            warnings.push(format!(
                "最终文件超过 {:.0}MB 的项目限制",
                project.size_limit_mb.unwrap_or_default()
            ));
        }
        storage.update_export_record(&project.id, &output.to_string_lossy())?;
        Ok(ExportResult {
            output_path: output.to_string_lossy().into_owned(),
            page_count: actual_pages,
            source_size_bytes,
            size_bytes,
            compression_applied,
            exceeded_limit,
            warnings,
        })
    })();
    let _ = fs::remove_file(&merged_temp);
    let _ = fs::remove_file(&compressed_temp);
    result
}

fn commit_export(temp: &Path, output: &Path, overwrite: bool) -> AppResult<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(temp)?
        .sync_all()?;

    if output.exists() {
        if !overwrite {
            return Err(AppError::Conflict(
                "目标文件已存在，请更换文件名或确认覆盖".to_string(),
            ));
        }
        if !output.is_file() {
            return Err(AppError::Path("输出位置不是普通文件".to_string()));
        }
        return replace_existing_export(temp, output);
    }

    // On Windows, rename fails if another process creates the target between the
    // existence check and this call, so a non-overwrite export never silently wins a race.
    fs::rename(temp, output)?;
    Ok(())
}

#[cfg(windows)]
fn replace_existing_export(temp: &Path, output: &Path) -> AppResult<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null;
    use windows_sys::Win32::Storage::FileSystem::{ReplaceFileW, REPLACEFILE_WRITE_THROUGH};

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    let parent = output
        .parent()
        .ok_or_else(|| AppError::Path("请选择有效的输出目录".to_string()))?;
    let file_name = output
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("申请材料.pdf");
    let backup = parent.join(format!(".{file_name}.{}.backup", Uuid::new_v4()));
    let output_wide = wide(output);
    let temp_wide = wide(temp);
    let backup_wide = wide(&backup);
    let replaced = unsafe {
        ReplaceFileW(
            output_wide.as_ptr(),
            temp_wide.as_ptr(),
            backup_wide.as_ptr(),
            REPLACEFILE_WRITE_THROUGH,
            null(),
            null(),
        )
    };
    if replaced == 0 {
        let error = std::io::Error::last_os_error();
        if !output.exists() && backup.exists() {
            fs::rename(&backup, output).map_err(|restore_error| {
                AppError::Path(format!(
                    "替换失败，旧文件保存在 {}。自动恢复也失败：{}",
                    backup.display(),
                    restore_error
                ))
            })?;
        }
        return Err(AppError::Io(error));
    }
    let _ = fs::remove_file(backup);
    Ok(())
}

#[cfg(not(windows))]
fn replace_existing_export(temp: &Path, output: &Path) -> AppResult<()> {
    let parent = output
        .parent()
        .ok_or_else(|| AppError::Path("请选择有效的输出目录".to_string()))?;
    let backup = parent.join(format!(".export-{}.backup", Uuid::new_v4()));
    fs::rename(output, &backup)?;
    match fs::rename(temp, output) {
        Ok(()) => {
            let _ = fs::remove_file(backup);
            Ok(())
        }
        Err(error) => {
            fs::rename(&backup, output)?;
            Err(AppError::Io(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_export_refuses_silent_overwrite() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let temp = directory.path().join("new.tmp.pdf");
        let output = directory.path().join("application.pdf");
        fs::write(&temp, b"new output").expect("write temporary output");
        fs::write(&output, b"old output").expect("write existing output");

        let result = commit_export(&temp, &output, false);

        assert!(matches!(result, Err(AppError::Conflict(_))));
        assert_eq!(fs::read(&output).expect("read original"), b"old output");
        assert_eq!(fs::read(&temp).expect("read temporary"), b"new output");
    }

    #[test]
    fn commit_export_creates_new_file() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let temp = directory.path().join("new.tmp.pdf");
        let output = directory.path().join("application.pdf");
        fs::write(&temp, b"new output").expect("write temporary output");

        commit_export(&temp, &output, false).expect("commit new output");

        assert_eq!(
            fs::read(&output).expect("read committed output"),
            b"new output"
        );
        assert!(!temp.exists());
    }

    #[cfg(windows)]
    #[test]
    fn commit_export_replaces_existing_file_atomically() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let temp = directory.path().join("new.tmp.pdf");
        let output = directory.path().join("application.pdf");
        fs::write(&temp, b"new output").expect("write temporary output");
        fs::write(&output, b"old output").expect("write existing output");

        commit_export(&temp, &output, true).expect("replace existing output");

        assert_eq!(fs::read(&output).expect("read replacement"), b"new output");
        assert!(!temp.exists());
    }
}
