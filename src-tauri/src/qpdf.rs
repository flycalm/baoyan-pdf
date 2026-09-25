use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use tauri::{AppHandle, Manager};
use wait_timeout::ChildExt;

use crate::error::{AppError, AppResult};
use crate::models::PdfCompressionLevel;

const QPDF_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone)]
pub struct QpdfEngine {
    executable: Option<PathBuf>,
    version: Option<String>,
}

#[derive(Debug)]
struct ProcessOutput {
    success: bool,
    warning: bool,
    stdout: String,
    stderr: String,
}

impl QpdfEngine {
    pub fn discover(app: &AppHandle) -> Self {
        let mut candidates = Vec::new();
        if let Ok(path) = std::env::var("BAOYAN_QPDF_PATH") {
            candidates.push(PathBuf::from(path));
        }
        if let Ok(resource_dir) = app.path().resource_dir() {
            candidates.push(resource_dir.join("qpdf").join("bin").join("qpdf.exe"));
            candidates.push(resource_dir.join("qpdf").join("qpdf.exe"));
        }
        candidates.push(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join("qpdf")
                .join("bin")
                .join("qpdf.exe"),
        );

        let executable = candidates.into_iter().find(|path| path.is_file());
        let version = executable.as_ref().and_then(|path| {
            run_process(
                path,
                vec![OsString::from("--version")],
                Duration::from_secs(15),
            )
            .ok()
            .filter(|output| output.success || output.warning)
            .map(|output| {
                output
                    .stdout
                    .lines()
                    .next()
                    .unwrap_or("qpdf")
                    .trim()
                    .to_string()
            })
        });
        Self {
            executable,
            version,
        }
    }

    pub fn version(&self) -> Option<String> {
        self.version.clone()
    }

    pub fn page_count(&self, path: &Path) -> AppResult<u32> {
        let executable = self.require_executable()?;
        let output = run_process(
            executable,
            vec![OsString::from("--show-npages"), path.as_os_str().to_owned()],
            QPDF_TIMEOUT,
        )?;
        // qpdf exit code 3 means the operation succeeded after recovering from
        // a structural warning (for example, rebuilding a broken xref table).
        // The parsed page count is still required below, so a warning cannot
        // turn an unreadable file into a successful import.
        if !output.success && !output.warning {
            return Err(classify_qpdf_error(&output.stderr));
        }
        output
            .stdout
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|pages| *pages > 0)
            .ok_or_else(|| AppError::Pdf("无法读取 PDF 页数".to_string()))
    }

    pub fn merge(&self, inputs: &[PathBuf], output: &Path) -> AppResult<()> {
        if inputs.is_empty() {
            return Err(AppError::User("没有可合并的 PDF 材料".to_string()));
        }
        let executable = self.require_executable()?;
        let mut args = Vec::<OsString>::with_capacity(inputs.len() * 2 + 4);
        args.push(OsString::from("--empty"));
        args.push(OsString::from("--pages"));
        for input in inputs {
            args.push(input.as_os_str().to_owned());
            args.push(OsString::from("1-z"));
        }
        args.push(OsString::from("--"));
        args.push(output.as_os_str().to_owned());
        let output_result = run_process(executable, args, QPDF_TIMEOUT)?;
        // A warning exit still represents a completed qpdf operation. The
        // caller verifies the generated PDF's exact page count before export.
        if !output_result.success && !output_result.warning {
            return Err(classify_qpdf_error(&output_result.stderr));
        }
        if !output.is_file() {
            return Err(AppError::Pdf("PDF 引擎未生成输出文件".to_string()));
        }
        Ok(())
    }

    pub fn optimize(
        &self,
        input: &Path,
        output: &Path,
        level: &PdfCompressionLevel,
    ) -> AppResult<()> {
        if matches!(level, PdfCompressionLevel::None) {
            return Err(AppError::User("未选择 PDF 压缩".to_string()));
        }
        let executable = self.require_executable()?;
        let mut args = vec![
            OsString::from("--object-streams=generate"),
            OsString::from("--compress-streams=y"),
            OsString::from("--recompress-flate"),
            OsString::from("--compression-level=9"),
        ];
        match level {
            PdfCompressionLevel::Standard => {
                args.push(OsString::from("--optimize-images"));
                args.push(OsString::from("--jpeg-quality=82"));
            }
            PdfCompressionLevel::Strong => {
                args.push(OsString::from("--optimize-images"));
                args.push(OsString::from("--jpeg-quality=60"));
            }
            PdfCompressionLevel::Lossless => {}
            PdfCompressionLevel::None => unreachable!(),
        }
        args.push(input.as_os_str().to_owned());
        args.push(output.as_os_str().to_owned());
        let output_result = run_process(executable, args, QPDF_TIMEOUT)?;
        if !output_result.success && !output_result.warning {
            return Err(classify_qpdf_error(&output_result.stderr));
        }
        if !output.is_file() {
            return Err(AppError::Pdf("PDF 压缩引擎未生成输出文件".to_string()));
        }
        Ok(())
    }

    fn require_executable(&self) -> AppResult<&Path> {
        self.executable.as_deref().ok_or_else(|| {
            AppError::Pdf("PDF 引擎尚未安装或资源文件缺失，请重新安装应用".to_string())
        })
    }
}

fn classify_qpdf_error(stderr: &str) -> AppError {
    let normalized = stderr.to_ascii_lowercase();
    if normalized.contains("invalid password") || normalized.contains("password") {
        AppError::Pdf("此 PDF 已加密，首版暂不支持需要密码的文件".to_string())
    } else if normalized.contains("not a pdf") || normalized.contains("pdf header") {
        AppError::Pdf("文件不是有效的 PDF".to_string())
    } else {
        let detail = stderr.trim();
        AppError::Pdf(if detail.is_empty() {
            "PDF 文件无法读取或结构已损坏".to_string()
        } else {
            format!(
                "PDF 文件无法处理：{}",
                detail.lines().last().unwrap_or(detail)
            )
        })
    }
}

fn run_process(
    executable: &Path,
    args: Vec<OsString>,
    timeout: Duration,
) -> AppResult<ProcessOutput> {
    let mut command = Command::new(executable);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let mut child = command.spawn().map_err(|error| {
        AppError::Pdf(format!(
            "无法启动 PDF 引擎（{}）：{error}",
            executable.display()
        ))
    })?;

    #[cfg(windows)]
    let _job = match WindowsJob::attach(&child) {
        Ok(job) => job,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Pdf("无法读取 PDF 引擎输出".to_string()))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::Pdf("无法读取 PDF 引擎错误信息".to_string()))?;
    let stdout_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stdout.read_to_end(&mut bytes);
        bytes
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.read_to_end(&mut bytes);
        bytes
    });

    let status = match child.wait_timeout(timeout)? {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(AppError::Pdf(format!(
                "PDF 处理超过 {} 秒，已安全终止",
                timeout.as_secs()
            )));
        }
    };

    let stdout_bytes = stdout_reader.join().unwrap_or_default();
    let stderr_bytes = stderr_reader.join().unwrap_or_default();
    Ok(ProcessOutput {
        success: status.code() == Some(0),
        warning: status.code() == Some(3),
        stdout: String::from_utf8_lossy(&stdout_bytes).into_owned(),
        stderr: String::from_utf8_lossy(&stderr_bytes).into_owned(),
    })
}

pub fn looks_like_pdf(path: &Path) -> AppResult<bool> {
    let mut file = File::open(path)?;
    let mut prefix = vec![0_u8; 1024];
    let read = file.read(&mut prefix)?;
    prefix.truncate(read);
    Ok(prefix.windows(5).any(|window| window == b"%PDF-"))
}

#[cfg(windows)]
struct WindowsJob(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl WindowsJob {
    fn attach(child: &std::process::Child) -> AppResult<Self> {
        use std::mem::size_of;
        use std::os::windows::io::AsRawHandle;
        use std::ptr::null;
        use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOB_OBJECT_LIMIT_PROCESS_MEMORY,
        };

        let job = unsafe { CreateJobObjectW(null(), null()) };
        if job.is_null() {
            return Err(AppError::Pdf("无法创建 PDF 进程安全边界".to_string()));
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            | JOB_OBJECT_LIMIT_PROCESS_MEMORY
            | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
        info.BasicLimitInformation.ActiveProcessLimit = 1;
        info.ProcessMemoryLimit = 1024 * 1024 * 1024;
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const std::ffi::c_void,
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        let process_handle = child.as_raw_handle() as HANDLE;
        let assigned =
            configured != 0 && unsafe { AssignProcessToJobObject(job, process_handle) } != 0;
        if !assigned {
            unsafe { CloseHandle(job) };
            return Err(AppError::Pdf("无法限制 PDF 子进程资源".to_string()));
        }
        Ok(Self(job))
    }
}

#[cfg(windows)]
impl Drop for WindowsJob {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::io::Write;

    fn bundled_engine() -> QpdfEngine {
        let executable = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("qpdf")
            .join("bin")
            .join("qpdf.exe");
        assert!(executable.is_file(), "bundled qpdf is missing");
        QpdfEngine {
            executable: Some(executable),
            version: None,
        }
    }

    fn write_minimal_pdf(path: &Path, marker: &str) {
        let content = format!("BT /F1 18 Tf 72 720 Td ({marker}) Tj ET");
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>".to_string(),
            format!("<< /Length {} >>\nstream\n{}\nendstream", content.len(), content),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        ];
        let mut bytes = b"%PDF-1.4\n".to_vec();
        let mut offsets = vec![0_usize];
        for (index, object) in objects.iter().enumerate() {
            offsets.push(bytes.len());
            write!(bytes, "{} 0 obj\n{}\nendobj\n", index + 1, object).expect("write PDF object");
        }
        let xref = bytes.len();
        write!(bytes, "xref\n0 {}\n", objects.len() + 1).expect("write xref header");
        bytes.extend_from_slice(b"0000000000 65535 f \n");
        for offset in offsets.into_iter().skip(1) {
            writeln!(bytes, "{offset:010} 00000 n ").expect("write xref entry");
        }
        write!(
            bytes,
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref
        )
        .expect("write trailer");
        std::fs::write(path, bytes).expect("write test PDF");
    }

    fn damage_xref_table(path: &Path) {
        let mut bytes = std::fs::read(path).expect("read test PDF");
        let marker = b"xref\n";
        let offset = bytes
            .windows(marker.len())
            .position(|window| window == marker)
            .expect("find xref table");
        bytes[offset + 1] = b'!';
        std::fs::write(path, bytes).expect("damage xref table");
    }

    #[test]
    fn bundled_qpdf_counts_and_merges_pages() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let first = directory.path().join("first.pdf");
        let second = directory.path().join("second.pdf");
        let merged = directory.path().join("merged.pdf");
        write_minimal_pdf(&first, "FIRST");
        write_minimal_pdf(&second, "SECOND");
        let engine = bundled_engine();

        assert_eq!(engine.page_count(&first).expect("count first PDF"), 1);
        engine
            .merge(&[first, second], &merged)
            .expect("merge valid PDFs");
        assert_eq!(engine.page_count(&merged).expect("count merged PDF"), 2);
    }

    #[test]
    fn pdf_header_detection_rejects_disguised_text() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let fake = directory.path().join("fake.pdf");
        std::fs::write(&fake, b"this is not a PDF").expect("write fake PDF");

        assert!(!looks_like_pdf(&fake).expect("inspect fake PDF"));
    }

    #[test]
    fn recoverable_xref_warning_can_be_counted_and_merged() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let damaged = directory.path().join("damaged-xref.pdf");
        let merged = directory.path().join("repaired-output.pdf");
        write_minimal_pdf(&damaged, "RECOVERABLE");
        damage_xref_table(&damaged);
        let engine = bundled_engine();

        let raw_output = run_process(
            engine.executable.as_deref().expect("qpdf path"),
            vec![
                OsString::from("--show-npages"),
                damaged.as_os_str().to_owned(),
            ],
            QPDF_TIMEOUT,
        )
        .expect("run qpdf against damaged xref");
        assert!(raw_output.warning, "fixture must produce a qpdf warning");

        assert_eq!(engine.page_count(&damaged).expect("recover page count"), 1);
        engine
            .merge(&[damaged], &merged)
            .expect("merge recoverable PDF");
        assert_eq!(
            engine.page_count(&merged).expect("count repaired output"),
            1
        );
    }

    #[test]
    fn compression_levels_preserve_page_count() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let source = directory.path().join("source.pdf");
        write_minimal_pdf(&source, "COMPRESS");
        let engine = bundled_engine();

        for (name, level) in [
            ("lossless", PdfCompressionLevel::Lossless),
            ("standard", PdfCompressionLevel::Standard),
            ("strong", PdfCompressionLevel::Strong),
        ] {
            let output = directory.path().join(format!("{name}.pdf"));
            engine
                .optimize(&source, &output, &level)
                .expect("compress test PDF");
            assert!(output.is_file(), "compression must create an output PDF");
            assert_eq!(engine.page_count(&output).expect("count compressed PDF"), 1);
        }
    }
}
