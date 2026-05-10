# 上传附件 + 本地 Tesseract OCR 方案

## 背景

- **Tesseract 5.5.2** 已安装（`/opt/homebrew/bin/tesseract`）
- 中文语言包已就绪：`chi_sim`（简体）、`chi_tra`（繁体）
- 项目 Rust 后端使用 **axum**，已支持 multipart
- 当前 `possession-entry` 只有纯文本输入，无文件上传或 OCR 能力

## 方案选择：Tesseract CLI 本地 OCR

通过 `std::process::Command` 调用系统 Tesseract 命令行：

```bash
tesseract temp.png stdout -l chi_sim+eng --psm 3
```

| 参数 | 含义 |
|------|------|
| `stdout` | 输出到标准输出 |
| `-l chi_sim+eng` | 中英文混合识别 |
| `--psm 3` | 自动页面分割（默认） |

优点：零新增 Rust crate 依赖、中文识别成熟、完全离线。

---

## 实施计划（共 5 步）

### 第一步：Rust 后端 — 新增 OCR 模块

**新建文件：** `rust/api/src/ocr.rs`

```rust
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

pub fn ocr_image(data: &[u8], lang: &str) -> Result<String, String> {
    // 写入临时文件
    let mut tmp = NamedTempFile::new().map_err(|e| e.to_string())?;
    tmp.write_all(data).map_err(|e| e.to_string())?;
    let path = tmp.path().to_str().ok_or("invalid temp path")?;

    // 调用 tesseract
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
```

依赖：`tempfile` crate（已在 `api/Cargo.toml` 或 `foundation/Cargo.toml` 中添加）

---

### 第二步：Rust 后端 — 新增 OCR API 端点

**修改文件：** `rust/api/src/routes/possess.rs`

在 `router()` 中新增路由：
```rust
.route("/ocr", post(ocr_upload))
```

新增端点：
```
POST /api/v1/possess/ocr
```

请求：`multipart/form-data`，字段名 `files`（可多文件）

响应：
```json
{
  "results": [
    {
      "filename": "photo.jpg",
      "text": "识别出的文字...",
      "error": null
    }
  ]
}
```

实现要点：
- 用 `axum::extract::Multipart` 解析上传
- 单文件最大 10MB，最多 5 个文件
- 校验 MIME 类型（`image/png`、`image/jpeg`、`image/webp`、`image/gif`）
- 并发调用 `ocr_image()`（`tokio::task::spawn_blocking`，因为 `std::process::Command` 是阻塞的）
- 语言默认 `chi_sim+eng`

---

### 第三步：前端 — API 层

**修改文件：** `nextjs/lib/api.ts`

新增函数：
```typescript
interface OcrResult { filename: string; text: string; error: string | null; }

export async function ocrFiles(files: File[]): Promise<OcrResult[]> {
  const form = new FormData();
  files.forEach((f) => form.append("files", f));
  const res = await fetch(`${API_BASE}/possess/ocr`, {
    method: "POST",
    body: form,
  });
  if (!res.ok) throw new Error(`OCR failed: ${res.statusText}`);
  const data = await res.json();
  return data.results;
}
```

---

### 第四步：前端 — AttachmentUpload 组件

**新建文件：** `nextjs/components/attachment-upload.tsx`

功能：
- **三种上传方式**：拖拽、粘贴（Ctrl+V 截屏）、文件选择
- **预览**：缩略图网格（`grid-cols-2` 或 `grid-cols-3`）
- **OCR 进度**：每个文件有独立状态指示（pending → recognizing → done/error）
- **文字预览**：OCR 完成后显示前 80 字的摘要，可点击展开
- **删除**：点击 × 移除文件及其 OCR 结果
- **识别按钮**：「开始识别」触发批量 OCR

Props：
```typescript
interface AttachmentUploadProps {
  onOcrResults: (texts: string[]) => void; // OCR 完成后回调，传文字数组
}
```

交互流程：
```
拖入/粘贴/选择图片 → 显示缩略图
  ↓
点击「开始识别」→ 每个文件显示 spinner
  ↓
调用 POST /possess/ocr → 批量返回结果
  ↓
显示提取文字预览 → 通过 onOcrResults 回调传出
```

---

### 第五步：前端 — 集成到 PossessionEntry

**修改文件：** `nextjs/components/possession-entry.tsx`

改动：
1. 在 `<Textarea>` 上方添加 `<AttachmentUpload>`
2. 新增 `attachmentTexts` state（`string[]`）
3. OCR 完成后，将文字以 `--- [附件文字] ---\n{text}\n` 格式追加到 `task` textarea 开头
4. 最终调用 `analyzeTask` 时，`task` 已包含附件 OCR 文字 + 用户手写问题

---

### 涉及文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `rust/api/src/ocr.rs` | **新建** | OCR 模块，封装 Tesseract CLI |
| `rust/api/src/main.rs` | 修改 | 注册 `ocr` 模块 |
| `rust/api/src/routes/possess.rs` | 修改 | 新增 `POST /possess/ocr` + `ocr_upload` handler |
| `rust/api/Cargo.toml` | 修改 | 添加 `tempfile` 依赖 |
| `nextjs/lib/api.ts` | 修改 | 新增 `ocrFiles()` |
| `nextjs/components/attachment-upload.tsx` | **新建** | 附件上传 + OCR 组件 |
| `nextjs/components/possession-entry.tsx` | 修改 | 集成附件上传 |

## 风险与注意事项

1. **Tesseract 路径**：使用 `"tesseract"` 直接调用（依赖 PATH），或检测 `which tesseract`
2. **阻塞调用**：`std::process::Command` 是同步的，必须用 `tokio::task::spawn_blocking` 包装
3. **临时文件清理**：`NamedTempFile` 在 drop 时自动删除
4. **并发**：多文件用 `futures::join_all` + `spawn_blocking` 并发识别
5. **语言检测**：后续可自动检测中英文比例选择 `chi_sim` / `eng`，当前默认 `chi_sim+eng`
6. **Tesseract 依赖**：如果目标机器没有 Tesseract，启动时检测并给出明确错误提示
