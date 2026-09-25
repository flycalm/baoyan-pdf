# 保研材料助手

一款 Windows 桌面应用，用于整理保研申请 PDF、按院校要求编排材料，并导出合并后的 PDF。核心材料处理在本机完成；AI 整理功能可选，使用时需自行配置兼容的模型接口和 API Key。

**下载 v0.2.0：** [Windows 安装包](https://github.com/flycalm/baoyan-pdf/releases/download/v0.2.0/Baoyan-PDF-Setup-v0.2.0.exe) · [Windows 便携版](https://github.com/flycalm/baoyan-pdf/releases/download/v0.2.0/Baoyan-PDF-Portable-v0.2.0.zip) · [查看发布说明](https://github.com/flycalm/baoyan-pdf/releases/tag/v0.2.0)

## 界面预览

![申请项目、材料编排与 PDF 导出界面](docs/screenshots/home.jpg)

<p align="center">
  <img src="docs/screenshots/library.jpg" alt="材料库界面" width="49%" />
  <img src="docs/screenshots/ai-assistant.jpg" alt="AI 整理报名材料界面" width="49%" />
</p>

## 功能

- **材料库**：导入 PDF，按类别管理常用材料，查看页数和预览。
- **申请项目**：为不同学校建立材料清单，调整顺序、必需状态和备注；支持保存为模板。
- **检查与导出**：检查缺失材料和文件状态，按顺序合并 PDF，可选择压缩级别并提示大小限制。
- **AI 整理（可选）**：读取报名通知网页或粘贴的正文，提取要求及依据，在本地用材料名称和分类匹配材料库。应用会展示匹配结果供人工确认。

## 安装与使用

目前面向 **Windows**。如需从源码运行，请先安装 Node.js、pnpm、Rust 工具链（Rust 1.88 或更新版本，Windows 使用 MSVC 目标）和 Tauri 2 所需的 Windows 构建依赖。

```powershell
pnpm install --frozen-lockfile
pnpm tauri dev
```

构建安装包：

```powershell
pnpm tauri build
```

仅预览前端界面可运行 `pnpm dev`。浏览器预览使用示例数据，不会读取桌面应用中的真实材料。PDF 导入、预览和导出需要桌面应用及 QPDF 运行文件。

## 数据与隐私

- 导入的 PDF、项目、模板和设置存放在当前 Windows 用户的本地应用数据目录。可从应用的“设置”页打开该目录。请勿将其中的数据库或 PDF 提交到公开仓库。
- 未启用 AI 时，核心 PDF 功能不需要模型接口。启用 AI 后，通知链接、网页正文或粘贴的通知文本，以及最多三个补充网页或附件的提取文本会发送给所配置的模型服务。**材料库中的个人 PDF 不会发送给模型**；材料匹配在本机进行。
- API Key 由用户在应用内填写，**以明文保存在当前用户的本地应用数据中**。不要共享该目录，也不要把密钥写进代码、议题或日志。
- AI 提取和匹配结果可能有误，提交申请前应对照学校通知及最终 PDF 人工核对。

## 开发与测试

前端使用 Svelte 5、TypeScript、Vite；桌面端使用 Tauri 2 和 Rust，数据存储使用 SQLite，PDF 处理使用 QPDF。

```powershell
pnpm check
pnpm test
pnpm build
cd src-tauri
cargo test --locked
```

测试用 PDF 可通过 `python scripts/create_pdf_fixtures.py` 生成到被忽略的 `tmp/` 目录；该脚本需要 `reportlab`。请勿将真实申请材料用作公开测试样本。

## 第三方组件

仓库包含 QPDF 12.4.0 的 Windows 运行文件，供桌面应用打包和本地测试使用。其来源、校验值及第三方许可见 [`src-tauri/resources/qpdf/SOURCE.md`](src-tauri/resources/qpdf/SOURCE.md) 和 [`THIRD_PARTY_NOTICES.md`](src-tauri/resources/qpdf/THIRD_PARTY_NOTICES.md)。本项目的 MIT 许可证只适用于本项目代码；QPDF 及其依赖遵循各自许可证。

## 参与贡献

欢迎提交问题和改进。提交前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。
