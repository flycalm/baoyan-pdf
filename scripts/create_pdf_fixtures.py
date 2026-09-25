from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.pagesizes import A4, landscape
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.cidfonts import UnicodeCIDFont
from reportlab.pdfgen import canvas
from reportlab.platypus import Table, TableStyle


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "tmp" / "pdfs"


def setup_font() -> str:
    name = "STSong-Light"
    pdfmetrics.registerFont(UnicodeCIDFont(name))
    return name


def draw_header(pdf: canvas.Canvas, title: str, subtitle: str, width: float, height: float, font: str) -> None:
    pdf.setFillColor(colors.HexColor("#245B96"))
    pdf.roundRect(42, height - 112, width - 84, 60, 10, fill=1, stroke=0)
    pdf.setFillColor(colors.white)
    pdf.setFont(font, 20)
    pdf.drawString(60, height - 82, title)
    pdf.setFont(font, 10)
    pdf.drawRightString(width - 60, height - 82, subtitle)


def create_resume(font: str) -> Path:
    path = OUTPUT / "01-resume-chinese.pdf"
    width, height = A4
    pdf = canvas.Canvas(str(path), pagesize=A4)
    for page in range(1, 3):
        draw_header(pdf, "个人简历", f"第 {page} 页 / 共 2 页", width, height, font)
        pdf.setFillColor(colors.HexColor("#172033"))
        pdf.setFont(font, 13)
        y = height - 150
        rows = [
            ("姓名", "测试申请人"),
            ("学校", "扬州大学"),
            ("研究方向", "大语言模型、多模态检索与智能体"),
            ("内容核验", f"RESUME-PAGE-{page}"),
        ]
        for label, value in rows:
            pdf.setFillColor(colors.HexColor("#EAF2FB"))
            pdf.roundRect(48, y - 8, 112, 28, 5, fill=1, stroke=0)
            pdf.setFillColor(colors.HexColor("#245B96"))
            pdf.drawString(60, y, label)
            pdf.setFillColor(colors.HexColor("#172033"))
            pdf.drawString(180, y, value)
            y -= 48
        pdf.setStrokeColor(colors.HexColor("#D7DEE8"))
        pdf.line(48, 70, width - 48, 70)
        pdf.setFont(font, 9)
        pdf.setFillColor(colors.HexColor("#667085"))
        pdf.drawString(48, 52, "保研材料助手自动化测试样本")
        pdf.showPage()
    pdf.save()
    return path


def create_transcript(font: str) -> Path:
    path = OUTPUT / "02-transcript-landscape.pdf"
    width, height = landscape(A4)
    pdf = canvas.Canvas(str(path), pagesize=(width, height))
    draw_header(pdf, "本科成绩单", "横向页面测试", width, height, font)
    data = [
        ["学期", "课程", "成绩", "学分", "核验标识"],
        ["2024 秋", "机器学习", "96", "3.0", "TRANSCRIPT-ROW-1"],
        ["2025 春", "自然语言处理", "94", "3.0", "TRANSCRIPT-ROW-2"],
        ["2025 秋", "计算机视觉", "92", "3.0", "TRANSCRIPT-ROW-3"],
    ]
    table = Table(data, colWidths=[90, 180, 70, 70, 190], rowHeights=34)
    table.setStyle(
        TableStyle(
            [
                ("FONTNAME", (0, 0), (-1, -1), font),
                ("FONTSIZE", (0, 0), (-1, -1), 11),
                ("BACKGROUND", (0, 0), (-1, 0), colors.HexColor("#EAF2FB")),
                ("TEXTCOLOR", (0, 0), (-1, 0), colors.HexColor("#245B96")),
                ("GRID", (0, 0), (-1, -1), 0.75, colors.HexColor("#AEB8C6")),
                ("ALIGN", (2, 1), (3, -1), "CENTER"),
                ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
                ("LEFTPADDING", (0, 0), (-1, -1), 10),
            ]
        )
    )
    table.wrapOn(pdf, width, height)
    table.drawOn(pdf, 70, height - 310)
    pdf.save()
    return path


def create_certificate(font: str) -> Path:
    path = OUTPUT / "03-certificate-rotated.pdf"
    width, height = 520, 520
    pdf = canvas.Canvas(str(path), pagesize=(width, height))
    pdf.setPageRotation(90)
    pdf.setStrokeColor(colors.HexColor("#B68A2A"))
    pdf.setLineWidth(8)
    pdf.roundRect(38, 38, width - 76, height - 76, 18, fill=0, stroke=1)
    pdf.setFont(font, 28)
    pdf.setFillColor(colors.HexColor("#7A5818"))
    pdf.drawCentredString(width / 2, 330, "获奖证书")
    pdf.setFont(font, 14)
    pdf.setFillColor(colors.HexColor("#172033"))
    pdf.drawCentredString(width / 2, 250, "复杂页面旋转与裁切框核验")
    pdf.drawCentredString(width / 2, 215, "CERTIFICATE-ROTATED-1")
    pdf.save()
    return path


def create_form(font: str) -> Path:
    path = OUTPUT / "04-filled-form.pdf"
    width, height = A4
    pdf = canvas.Canvas(str(path), pagesize=A4)
    draw_header(pdf, "申请信息表", "表单外观测试", width, height, font)
    pdf.setFont(font, 12)
    pdf.setFillColor(colors.HexColor("#172033"))
    pdf.drawString(64, height - 170, "申请编号")
    pdf.acroForm.textfield(
        name="application_id",
        value="FORM-2026-0001",
        x=150,
        y=height - 188,
        width=250,
        height=28,
        borderColor=colors.HexColor("#7B8A9E"),
        fillColor=colors.white,
        textColor=colors.HexColor("#172033"),
        forceBorder=True,
    )
    pdf.drawString(64, height - 225, "提交状态")
    pdf.acroForm.checkbox(
        name="confirmed",
        checked=True,
        x=150,
        y=height - 238,
        buttonStyle="check",
        borderColor=colors.HexColor("#7B8A9E"),
        fillColor=colors.white,
        textColor=colors.HexColor("#245B96"),
        forceBorder=True,
    )
    pdf.setFont(font, 10)
    pdf.drawString(64, 80, "FORM-APPEARANCE-CHECK")
    pdf.save()
    return path


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    font = setup_font()
    generated = [
        create_resume(font),
        create_transcript(font),
        create_certificate(font),
        create_form(font),
    ]
    for path in generated:
        print(path)


if __name__ == "__main__":
    main()
