use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    User(String),
    #[error("数据库操作失败：{0}")]
    Database(#[from] rusqlite::Error),
    #[error("文件操作失败：{0}")]
    Io(#[from] std::io::Error),
    #[error("PDF 处理失败：{0}")]
    Pdf(String),
    #[error("数据格式错误：{0}")]
    Serialization(#[from] serde_json::Error),
    #[error("应用路径不可用：{0}")]
    Path(String),
    #[error("未找到：{0}")]
    NotFound(String),
    #[error("操作冲突：{0}")]
    Conflict(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorPayload<'a> {
    code: &'a str,
    message: String,
    detail: Option<String>,
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::User(_) => "invalid_request",
            Self::Database(_) => "database_error",
            Self::Io(_) => "io_error",
            Self::Pdf(_) => "pdf_error",
            Self::Serialization(_) => "serialization_error",
            Self::Path(_) => "path_error",
            Self::NotFound(_) => "not_found",
            Self::Conflict(_) => "conflict",
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ErrorPayload {
            code: self.code(),
            message: self.to_string(),
            detail: None,
        }
        .serialize(serializer)
    }
}

pub type AppResult<T> = Result<T, AppError>;
