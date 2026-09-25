use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Material {
    pub id: String,
    pub name: String,
    pub category: String,
    pub original_name: String,
    pub size_bytes: u64,
    pub page_count: u32,
    pub sha256: String,
    pub scope: String,
    pub scope_project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stored_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub used_by_count: u32,
    pub status: String,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleFile {
    pub id: String,
    pub material_id: String,
    pub position: i64,
    pub material: Material,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectModule {
    pub id: String,
    pub title: String,
    pub category: String,
    pub position: i64,
    pub required: bool,
    pub enabled: bool,
    pub files: Vec<ModuleFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationProject {
    pub id: String,
    pub school: String,
    pub department: String,
    pub program: String,
    pub notice_url: String,
    pub deadline: Option<String>,
    pub size_limit_mb: Option<f64>,
    pub notes: String,
    pub modules: Vec<ProjectModule>,
    pub created_at: String,
    pub updated_at: String,
    pub last_export_path: Option<String>,
    pub last_exported_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateModule {
    pub id: String,
    pub title: String,
    pub category: String,
    pub position: i64,
    pub required: bool,
    pub enabled: bool,
    pub material_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub modules: Vec<TemplateModule>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub last_page: String,
    pub last_project_id: Option<String>,
    pub default_export_directory: Option<String>,
    pub confirm_before_overwrite: bool,
    pub ai_enabled: bool,
    pub ai_base_url: String,
    pub ai_api_key: String,
    pub ai_model: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            last_page: "projects".to_string(),
            last_project_id: None,
            default_export_directory: None,
            confirm_before_overwrite: true,
            ai_enabled: false,
            ai_base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            ai_api_key: String::new(),
            ai_model: "glm-5.3-flash".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub materials: Vec<Material>,
    pub projects: Vec<ApplicationProject>,
    pub templates: Vec<ProjectTemplate>,
    pub settings: AppSettings,
    pub app_version: String,
    pub data_directory: String,
    pub qpdf_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub school: String,
    pub department: String,
    pub program: String,
    pub notice_url: String,
    pub deadline: Option<String>,
    pub size_limit_mb: Option<f64>,
    pub notes: String,
    pub template_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportMaterialsInput {
    pub paths: Vec<String>,
    pub category: String,
    pub scope_project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFailure {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDuplicate {
    pub path: String,
    pub existing_material_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: Vec<Material>,
    pub failures: Vec<ImportFailure>,
    pub duplicates: Vec<ImportDuplicate>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PdfCompressionLevel {
    None,
    Lossless,
    Standard,
    Strong,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProjectInput {
    pub project_id: String,
    pub output_path: String,
    pub overwrite: bool,
    pub allow_warnings: bool,
    pub compression_level: PdfCompressionLevel,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub output_path: String,
    pub page_count: u32,
    pub source_size_bytes: u64,
    pub size_bytes: u64,
    pub compression_applied: bool,
    pub exceeded_limit: bool,
    pub warnings: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeRegistrationNoticeInput {
    pub notice_url: Option<String>,
    pub notice_text: Option<String>,
    pub request_id: Option<String>,
    pub config: AiProviderConfig,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectionResult {
    pub ok: bool,
    pub model: String,
    pub latency_ms: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RegistrationRequirement {
    pub name: String,
    pub category: String,
    pub required: bool,
    pub details: String,
    pub format: Option<String>,
    pub copies: Option<u32>,
    pub deadline: Option<String>,
    pub evidence_ids: Vec<String>,
}

impl Default for RegistrationRequirement {
    fn default() -> Self {
        Self {
            name: String::new(),
            category: "其他材料".to_string(),
            required: true,
            details: String::new(),
            format: None,
            copies: None,
            deadline: None,
            evidence_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AiEvidence {
    pub id: String,
    pub source_url: String,
    pub quote: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSource {
    pub url: String,
    pub title: Option<String>,
    pub content_type: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiToolLogEntry {
    pub round: u8,
    pub url: String,
    pub status: String,
    pub content_type: Option<String>,
    pub bytes: Option<u64>,
    pub extracted_chars: Option<u64>,
    pub links_found: Option<u32>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeRegistrationNoticeResult {
    pub summary: String,
    pub requirements: Vec<RegistrationRequirement>,
    pub evidence: Vec<AiEvidence>,
    pub source: Vec<AiSource>,
    pub confidence: f64,
    pub warnings: Vec<String>,
    pub tool_log: Vec<AiToolLogEntry>,
}
