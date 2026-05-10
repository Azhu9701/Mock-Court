use std::io::Write;
use std::process::Command;

pub fn ocr_image(data: &[u8], lang: &str) -> Result<String, String> {
    let mut tmp = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    tmp.write_all(data).map_err(|e| e.to_string())?;
    let path = tmp.path().to_str().ok_or("invalid temp path")?;

    let output = Command::new("tesseract")
        .arg(path)
        .arg("stdout")
        .arg("-l")
        .arg(lang)
        .arg("--psm")
        .arg("3")
        .output()
        .map_err(|e| format!("tesseract 调用失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("OCR 失败: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
