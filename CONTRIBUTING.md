# 参与贡献

欢迎报告问题、改进界面和完善 PDF 处理。提交议题时，请说明复现步骤、预期结果、实际结果和应用版本。

请先删除截图或示例 PDF 中的姓名、学校账号、联系方式、成绩、证件信息和本地路径。不要在议题、提交或拉取请求中放入 API Key 或真实申请材料；如需复现 PDF 问题，可运行 `python scripts/create_pdf_fixtures.py` 生成测试样本。

提交代码前运行：

```powershell
pnpm check
pnpm test
pnpm build
cd src-tauri
cargo test --locked
```

请在拉取请求中简要说明改动、验证方式，以及对用户数据或 PDF 输出的影响。
