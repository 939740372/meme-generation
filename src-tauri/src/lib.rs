use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    env,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppError {
    code: String,
    message: String,
}

type CommandResult<T> = Result<T, AppError>;

fn app_error(code: &str, message: impl Into<String>) -> AppError {
    AppError {
        code: code.to_string(),
        message: message.into(),
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("欢迎回来，{}。", name)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: String,
    platform: String,
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaInfo {
    path: String,
    file_name: String,
    format: String,
    duration_secs: f64,
    width: u32,
    height: u32,
    fps: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CropArea {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TextOverlay {
    text: String,
    x: u32,
    y: u32,
    size: u32,
    color: String,
    stroke_width: Option<u32>,
    stroke_color: Option<String>,
    use_box: Option<bool>,
    box_color: Option<String>,
    box_opacity: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageOverlay {
    path: String,
    x: u32,
    y: u32,
    width: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenderPlan {
    input_path: String,
    output_path: String,
    start_secs: f64,
    end_secs: f64,
    crop: Option<CropArea>,
    output_width: u32,
    fps: u32,
    quality: Option<String>,
    text_overlay: Option<TextOverlay>,
    image_overlay: Option<ImageOverlay>,
    #[serde(default)]
    format: Option<String>,
}

impl RenderPlan {
    fn is_webp(&self) -> bool {
        if let Some(fmt) = &self.format {
            if fmt.eq_ignore_ascii_case("webp") {
                return true;
            }
        }
        self.output_path.to_ascii_lowercase().ends_with(".webp")
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenderProgress {
    task_id: String,
    stage: String,
    progress: f32,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenderCompleted {
    task_id: String,
    output_path: String,
    output_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenderFailed {
    task_id: String,
    code: String,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenderCancelled {
    task_id: String,
    message: String,
}

#[derive(Clone)]
struct ManagedProcess {
    child: Arc<Mutex<Child>>,
    cancelled: Arc<AtomicBool>,
}

#[derive(Clone, Default)]
struct RenderState {
    processes: Arc<Mutex<HashMap<String, ManagedProcess>>>,
}

fn resolve_binary(name: &str) -> PathBuf {
    // 1. 优先使用环境变量指定的可执行文件
    let variable = format!("{}_BINARY", name.to_ascii_uppercase());
    if let Ok(value) = env::var(&variable) {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }

    // 2. 检查应用可执行文件同级或 resources/binaries 目录（生产打包环境）
    if let Ok(current_exe) = env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let direct = exe_dir.join(name);
            if direct.is_file() {
                return direct;
            }
            let direct_exe = exe_dir.join(format!("{}.exe", name));
            if direct_exe.is_file() {
                return direct_exe;
            }
            let res_candidate = exe_dir.join("../Resources/binaries").join(name);
            if res_candidate.is_file() {
                return res_candidate;
            }
        }
    }

    // 3. 检查开发环境下的 src-tauri/binaries/<name>-<target-triple>
    let target_triple = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        _ => "",
    };

    if !target_triple.is_empty() {
        let binary_filename = if cfg!(windows) {
            format!("{}-{}.exe", name, target_triple)
        } else {
            format!("{}-{}", name, target_triple)
        };

        let dev_candidates = [
            PathBuf::from("src-tauri").join("binaries").join(&binary_filename),
            PathBuf::from("binaries").join(&binary_filename),
            PathBuf::from("..").join("src-tauri").join("binaries").join(&binary_filename),
        ];

        for candidate in dev_candidates {
            if candidate.is_file() {
                return candidate;
            }
        }
    }

    // 4. 回退到系统的 PATH
    PathBuf::from(name)
}

fn detect_system_font() -> Option<String> {
    let candidates: &[&str] = if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Helvetica.ttc",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
        ]
    } else if cfg!(target_os = "windows") {
        &[
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ]
    } else {
        &[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        ]
    };

    for font in candidates {
        if Path::new(font).exists() {
            return Some((*font).to_string());
        }
    }
    None
}

fn path_to_string(path: &Path) -> CommandResult<String> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| app_error("INVALID_PATH", "文件路径包含无法处理的字符。"))
}

fn validate_input_file(path: &Path) -> CommandResult<()> {
    if !path.exists() {
        return Err(app_error("INPUT_NOT_FOUND", "找不到输入文件。"));
    }

    if !path.is_file() {
        return Err(app_error("INPUT_NOT_FILE", "输入路径不是文件。"));
    }

    Ok(())
}

fn json_number(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .or_else(|| value.and_then(Value::as_str).and_then(|text| text.parse().ok()))
}

fn parse_ratio(value: &str) -> Option<f64> {
    if let Some((numerator, denominator)) = value.split_once('/') {
        let numerator = numerator.parse::<f64>().ok()?;
        let denominator = denominator.parse::<f64>().ok()?;
        if denominator == 0.0 {
            return None;
        }
        return Some(numerator / denominator);
    }

    value.parse::<f64>().ok()
}

fn probe_media(path: &Path) -> CommandResult<MediaInfo> {
    validate_input_file(path)?;
    let path_string = path_to_string(path)?;
    let ffprobe_bin = resolve_binary("ffprobe");
    let output = Command::new(&ffprobe_bin)
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_type,width,height,avg_frame_rate,duration:format=format_name,duration",
            "-of",
            "json",
            &path_string,
        ])
        .output()
        .map_err(|error| {
            app_error(
                "FFPROBE_UNAVAILABLE",
                format!(
                    "无法启动 ffprobe（{}）：{}。请确保已下载 sidecar 或设置 FFPROBE_BINARY。",
                    ffprobe_bin.display(),
                    error
                ),
            )
        })?;

    if !output.status.success() {
        return Err(app_error(
            "MEDIA_PROBE_FAILED",
            "ffprobe 无法读取这个文件，可能是格式不支持或文件已损坏。",
        ));
    }

    let document: Value = serde_json::from_slice(&output.stdout)
        .map_err(|_| app_error("MEDIA_PROBE_INVALID", "ffprobe 返回了无法解析的媒体信息。"))?;
    let stream = document
        .get("streams")
        .and_then(Value::as_array)
        .and_then(|streams| streams.first())
        .ok_or_else(|| app_error("NO_VIDEO_STREAM", "输入文件没有可用的视频流。"))?;

    let width = stream
        .get("width")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| app_error("MEDIA_PROBE_INVALID", "无法读取视频宽度。"))?;
    let height = stream
        .get("height")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| app_error("MEDIA_PROBE_INVALID", "无法读取视频高度。"))?;
    let fps = stream
        .get("avg_frame_rate")
        .and_then(Value::as_str)
        .and_then(parse_ratio)
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(0.0);
    let duration_secs = json_number(stream.get("duration"))
        .or_else(|| {
            document
                .get("format")
                .and_then(|format| json_number(format.get("duration")))
        })
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| app_error("MEDIA_PROBE_INVALID", "无法读取视频时长。"))?;
    let format = document
        .get("format")
        .and_then(|format| format.get("format_name"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("video")
        .to_string();

    Ok(MediaInfo {
        path: path_string,
        file_name,
        format,
        duration_secs,
        width,
        height,
        fps,
    })
}

#[tauri::command]
fn media_probe(path: String) -> CommandResult<MediaInfo> {
    probe_media(Path::new(&path))
}

fn is_hex_color(value: &str) -> bool {
    (value.len() == 7 || value.len() == 9)
        && value.starts_with('#')
        && value[1..].chars().all(|character| character.is_ascii_hexdigit())
}

fn validate_render_plan(plan: &RenderPlan, media: &MediaInfo) -> CommandResult<()> {
    if !plan.start_secs.is_finite()
        || !plan.end_secs.is_finite()
        || plan.start_secs < 0.0
        || plan.end_secs <= plan.start_secs
        || plan.end_secs > media.duration_secs + 0.05
    {
        return Err(app_error(
            "INVALID_TIME_RANGE",
            format!("时间范围必须在 0 到 {:.2} 秒之间。", media.duration_secs),
        ));
    }

    if !(32..=1920).contains(&plan.output_width) {
        return Err(app_error("INVALID_OUTPUT_WIDTH", "输出宽度必须在 32 到 1920 之间。"));
    }

    if !(1..=60).contains(&plan.fps) {
        return Err(app_error("INVALID_FPS", "FPS 必须在 1 到 60 之间。"));
    }

    if let Some(crop) = &plan.crop {
        if crop.width == 0
            || crop.height == 0
            || crop.x >= media.width
            || crop.y >= media.height
            || crop.x.saturating_add(crop.width) > media.width
            || crop.y.saturating_add(crop.height) > media.height
        {
            return Err(app_error("INVALID_CROP", "裁切区域必须完整落在原视频画面内。"));
        }
    }

    if let Some(text) = &plan.text_overlay {
        if text.text.trim().is_empty() {
            return Err(app_error("INVALID_TEXT", "文字叠加内容不能为空。"));
        }
        if text.text.chars().any(char::is_control) {
            return Err(app_error("INVALID_TEXT", "文字叠加不能包含控制字符。"));
        }
        if !(8..=192).contains(&text.size) || !is_hex_color(&text.color) {
            return Err(app_error("INVALID_TEXT_STYLE", "文字字号或颜色格式无效。"));
        }
    }

    if let Some(image) = &plan.image_overlay {
        let image_path = Path::new(&image.path);
        validate_input_file(image_path)?;
        if image_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase() != "png")
            .unwrap_or(true)
        {
            return Err(app_error("INVALID_IMAGE", "图案叠加目前只支持 PNG 文件。"));
        }
        if image.width == 0 || image.width > 1920 {
            return Err(app_error("INVALID_IMAGE_SIZE", "图案宽度必须在 1 到 1920 之间。"));
        }
    }

    let output_path = Path::new(&plan.output_path);
    let ext = output_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase());
    match ext.as_deref() {
        Some("gif") | Some("webp") => {}
        _ => {
            return Err(app_error(
                "INVALID_OUTPUT",
                "输出文件必须使用 .gif 或 .webp 扩展名。",
            ));
        }
    }
    if let Some(fmt) = &plan.format {
        let fmt_lower = fmt.to_ascii_lowercase();
        if (fmt_lower == "webp" && ext.as_deref() != Some("webp"))
            || (fmt_lower == "gif" && ext.as_deref() != Some("gif"))
        {
            return Err(app_error(
                "INVALID_OUTPUT_FORMAT",
                "输出格式设置与文件扩展名不一致。",
            ));
        }
    }
    if output_path.exists() {
        return Err(app_error("OUTPUT_EXISTS", "输出文件已存在，请选择另一个路径。"));
    }
    let parent = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.exists() || !parent.is_dir() {
        return Err(app_error("OUTPUT_DIRECTORY_INVALID", "输出目录不存在或不可用。"));
    }

    Ok(())
}

fn format_secs(value: f64) -> String {
    format!("{value:.3}")
}

fn escape_drawtext(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\\' => "\\\\".to_string(),
            '\'' => "\\\\'".to_string(),
            ':' => "\\:".to_string(),
            ',' => "\\,".to_string(),
            '%' => "\\%".to_string(),
            other => other.to_string(),
        })
        .collect()
}

fn color_for_filter(value: &str) -> String {
    if let Some(hex) = value.strip_prefix('#') {
        format!("0x{}", hex)
    } else {
        value.to_string()
    }
}

fn build_filter_graph(plan: &RenderPlan) -> String {
    let mut source = String::new();
    if let Some(crop) = &plan.crop {
        source.push_str(&format!("crop={}:{}:{}:{},", crop.width, crop.height, crop.x, crop.y));
    }
    source.push_str(&format!(
        "scale={}:{}:flags=lanczos",
        plan.output_width, -1
    ));
    source.push_str(&format!(",fps={}", plan.fps));

    if let Some(text) = &plan.text_overlay {
        let mut drawtext_parts = Vec::new();
        if let Some(fontfile) = detect_system_font() {
            drawtext_parts.push(format!("fontfile='{}'", escape_drawtext(&fontfile)));
        }
        drawtext_parts.push(format!("text='{}'", escape_drawtext(&text.text)));
        drawtext_parts.push(format!("fontcolor={}", color_for_filter(&text.color)));
        drawtext_parts.push(format!("fontsize={}", text.size));
        drawtext_parts.push(format!("x={}", text.x));
        drawtext_parts.push(format!("y={}", text.y));

        let stroke_w = text.stroke_width.unwrap_or(0);
        if stroke_w > 0 {
            let stroke_c = text.stroke_color.as_deref().unwrap_or("#000000");
            drawtext_parts.push(format!("borderw={}", stroke_w));
            drawtext_parts.push(format!("bordercolor={}", color_for_filter(stroke_c)));
        }

        if text.use_box.unwrap_or(false) {
            let box_c = text.box_color.as_deref().unwrap_or("#000000");
            let box_op = text.box_opacity.unwrap_or(0.60);
            drawtext_parts.push(format!(
                "box=1:boxcolor={}@{:.2}:boxborderw=8",
                color_for_filter(box_c),
                box_op
            ));
        } else {
            drawtext_parts.push("box=0".to_string());
        }

        source.push_str(&format!(",drawtext={}", drawtext_parts.join(":")));
    }

    if plan.is_webp() {
        if let Some(image) = &plan.image_overlay {
            format!(
                "[0:v]{}[base];[1:v]scale={}:{}:flags=lanczos[sticker];[base][sticker]overlay={}:{}:eof_action=pass[vout]",
                source, image.width, -1, image.x, image.y
            )
        } else {
            format!("[0:v]{}[vout]", source)
        }
    } else {
        let mut graph = format!("[0:v]{}[base]", source);
        let final_label = if let Some(image) = &plan.image_overlay {
            graph.push_str(&format!(
                ";[1:v]scale={}:{}:flags=lanczos[sticker];[base][sticker]overlay={}:{}:eof_action=pass[composed]",
                image.width, -1, image.x, image.y
            ));
            "[composed]"
        } else {
            "[base]"
        };
        let dither = match plan.quality.as_deref() {
            Some("low") => "bayer:bayer_scale=3",
            Some("high") => "sierra2_4a",
            Some("none") => "none",
            _ => "sierra2_4a",
        };
        graph.push_str(&format!(
            ";{}split[gif][palette];[palette]palettegen=stats_mode=diff[p];[gif][p]paletteuse=dither={}[vout]",
            final_label, dither
        ));
        graph
    }
}

fn build_ffmpeg_args(plan: &RenderPlan) -> Vec<String> {
    let duration = plan.end_secs - plan.start_secs;
    let filter_graph = build_filter_graph(plan);
    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-nostats".to_string(),
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-n".to_string(),
        "-ss".to_string(),
        format_secs(plan.start_secs),
        "-t".to_string(),
        format_secs(duration),
        "-i".to_string(),
        plan.input_path.clone(),
    ];

    if let Some(image) = &plan.image_overlay {
        args.extend([
            "-loop".to_string(),
            "1".to_string(),
            "-i".to_string(),
            image.path.clone(),
        ]);
    }

    if plan.is_webp() {
        let q_val = match plan.quality.as_deref() {
            Some("low") => "55",
            Some("high") => "90",
            _ => "75",
        };
        args.extend([
            "-filter_complex".to_string(),
            filter_graph,
            "-map".to_string(),
            "[vout]".to_string(),
            "-c:v".to_string(),
            "libwebp".to_string(),
            "-q:v".to_string(),
            q_val.to_string(),
            "-loop".to_string(),
            "0".to_string(),
            "-an".to_string(),
            plan.output_path.clone(),
        ]);
    } else {
        args.extend([
            "-filter_complex".to_string(),
            filter_graph,
            "-map".to_string(),
            "[vout]".to_string(),
            "-an".to_string(),
            "-loop".to_string(),
            "0".to_string(),
            "-f".to_string(),
            "gif".to_string(),
            plan.output_path.clone(),
        ]);
    }
    args
}

fn ffmpeg_supports_filter(filter_name: &str) -> Option<bool> {
    let ffmpeg_bin = resolve_binary("ffmpeg");
    let output = Command::new(&ffmpeg_bin)
        .args(["-hide_banner", "-filters"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Some(
        stdout
            .lines()
            .chain(stderr.lines())
            .any(|line| line.split_whitespace().any(|token| token == filter_name)),
    )
}

fn task_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("render-{nanos}")
}

fn emit_progress(app: &AppHandle, task_id: &str, stage: &str, progress: f32, message: &str) {
    let _ = app.emit(
        "render_progress",
        RenderProgress {
            task_id: task_id.to_string(),
            stage: stage.to_string(),
            progress: progress.clamp(0.0, 100.0),
            message: message.to_string(),
        },
    );
}

fn progress_seconds(line: &str) -> Option<f64> {
    let value = line.strip_prefix("out_time_ms=")?.parse::<f64>().ok()?;
    Some(value / 1_000_000.0)
}

fn trim_error(message: String) -> String {
    let message = message.trim().to_string();
    if message.is_empty() {
        return "FFmpeg 没有返回具体错误信息。".to_string();
    }
    message.chars().take(260).collect()
}

fn run_render(
    app: AppHandle,
    state: RenderState,
    task_id: String,
    process: ManagedProcess,
    stdout: Option<std::process::ChildStdout>,
    stderr: Option<std::process::ChildStderr>,
    output_path: PathBuf,
    duration_secs: f64,
) {
    let progress_task = stdout.map(|stdout| {
        let app = app.clone();
        let task_id = task_id.clone();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Some(seconds) = progress_seconds(&line) {
                    let progress = 6.0 + ((seconds / duration_secs).clamp(0.0, 1.0) as f32 * 88.0);
                    emit_progress(&app, &task_id, "rendering", progress, "正在编码 GIF…");
                } else if line == "progress=end" {
                    emit_progress(&app, &task_id, "finalizing", 96.0, "正在验证输出…");
                }
            }
        })
    });
    let error_task = stderr.map(|stderr| {
        thread::spawn(move || {
            let mut last_line = String::new();
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    last_line = line;
                }
            }
            last_line
        })
    });

    let status = loop {
        let poll = match process.child.lock() {
            Ok(mut child) => child.try_wait().map_err(|_| ()),
            Err(_) => Err(()),
        };
        match poll {
            Ok(Some(status)) => break Some(status),
            Ok(None) => thread::sleep(Duration::from_millis(60)),
            Err(_) => break None,
        }
    };

    if let Some(progress_task) = progress_task {
        let _ = progress_task.join();
    }
    let error_message = error_task
        .and_then(|error_task| error_task.join().ok())
        .unwrap_or_default();
    if let Ok(mut processes) = state.processes.lock() {
        processes.remove(&task_id);
    }

    if process.cancelled.load(Ordering::SeqCst) {
        let _ = app.emit(
            "render_cancelled",
            RenderCancelled {
                task_id,
                message: "任务已取消，未保留不完整的输出文件。".to_string(),
            },
        );
        let _ = fs::remove_file(output_path);
        return;
    }

    let Some(status) = status else {
        let _ = app.emit(
            "render_failed",
            RenderFailed {
                task_id,
                code: "PROCESS_STATUS_UNKNOWN".to_string(),
                message: "无法读取 FFmpeg 进程状态。".to_string(),
            },
        );
        return;
    };

    if !status.success() {
        let _ = app.emit(
            "render_failed",
            RenderFailed {
                task_id,
                code: "FFMPEG_FAILED".to_string(),
                message: trim_error(error_message),
            },
        );
        let _ = fs::remove_file(output_path);
        return;
    }

    let output_bytes = match fs::metadata(&output_path) {
        Ok(metadata) if metadata.len() > 0 => metadata.len(),
        _ => {
            let _ = app.emit(
                "render_failed",
                RenderFailed {
                    task_id,
                    code: "OUTPUT_INVALID".to_string(),
                    message: "FFmpeg 完成了，但没有生成可用的 GIF 文件。".to_string(),
                },
            );
            let _ = fs::remove_file(output_path);
            return;
        }
    };

    if let Err(error) = probe_media(&output_path) {
        let _ = app.emit(
            "render_failed",
            RenderFailed {
                task_id,
                code: "OUTPUT_NOT_READABLE".to_string(),
                message: error.message,
            },
        );
        let _ = fs::remove_file(output_path);
        return;
    }

    emit_progress(&app, &task_id, "finalizing", 100.0, "GIF 导出完成。");
    let _ = app.emit(
        "render_completed",
        RenderCompleted {
            task_id,
            output_path: output_path.to_string_lossy().to_string(),
            output_bytes,
        },
    );
}

#[tauri::command]
fn render_gif(
    app: AppHandle,
    state: State<'_, RenderState>,
    plan: RenderPlan,
) -> CommandResult<String> {
    let media = probe_media(Path::new(&plan.input_path))?;
    validate_render_plan(&plan, &media)?;
    if plan.text_overlay.is_some() && ffmpeg_supports_filter("drawtext") == Some(false) {
        return Err(app_error(
            "DRAW_TEXT_UNAVAILABLE",
            "当前 FFmpeg 构建不包含 drawtext，请换用带 libfreetype 的 FFmpeg。",
        ));
    }
    let args = build_ffmpeg_args(&plan);
    let ffmpeg_bin = resolve_binary("ffmpeg");
    let mut child = Command::new(&ffmpeg_bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            app_error(
                "FFMPEG_UNAVAILABLE",
                format!(
                    "无法启动 ffmpeg（{}）：{}。请确保已下载 sidecar 或设置 FFMPEG_BINARY。",
                    ffmpeg_bin.display(),
                    error
                ),
            )
        })?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let process = ManagedProcess {
        child: Arc::new(Mutex::new(child)),
        cancelled: Arc::new(AtomicBool::new(false)),
    };
    let task_id = task_id();
    let output_path = PathBuf::from(&plan.output_path);
    let duration_secs = plan.end_secs - plan.start_secs;
    state
        .inner()
        .processes
        .lock()
        .map_err(|_| app_error("TASK_STATE_UNAVAILABLE", "渲染任务状态暂时不可用。"))?
        .insert(task_id.clone(), process.clone());

    emit_progress(&app, &task_id, "starting", 2.0, "正在启动 FFmpeg…");
    let render_state = state.inner().clone();
    let render_app = app.clone();
    let render_task_id = task_id.clone();
    thread::spawn(move || {
        run_render(
            render_app,
            render_state,
            render_task_id,
            process,
            stdout,
            stderr,
            output_path,
            duration_secs,
        );
    });

    Ok(task_id)
}

#[tauri::command]
fn cancel_render(state: State<'_, RenderState>, task_id: String) -> CommandResult<()> {
    let process = state
        .inner()
        .processes
        .lock()
        .map_err(|_| app_error("TASK_STATE_UNAVAILABLE", "渲染任务状态暂时不可用。"))?
        .get(&task_id)
        .cloned()
        .ok_or_else(|| app_error("TASK_NOT_FOUND", "渲染任务不存在或已经结束。"))?;
    process.cancelled.store(true, Ordering::SeqCst);
    let mut child = process
        .child
        .lock()
        .map_err(|_| app_error("PROCESS_UNAVAILABLE", "无法访问 FFmpeg 进程。"))?;
    match child
        .try_wait()
        .map_err(|error| app_error("PROCESS_UNAVAILABLE", format!("无法读取 FFmpeg 状态：{}", error)))?
    {
        Some(_) => return Ok(()),
        None => child
            .kill()
            .map_err(|error| app_error("CANCEL_FAILED", format!("无法取消 FFmpeg：{}", error)))?,
    }
    Ok(())
}

#[tauri::command]
fn copy_file_to_clipboard(path: String) -> CommandResult<()> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(app_error("FILE_NOT_FOUND", "文件不存在，无法复制。"));
    }
    #[cfg(target_os = "macos")]
    {
        let escaped = path.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!("set the clipboard to (POSIX file \"{}\")", escaped);
        let status = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .status();
        match status {
            Ok(s) if s.success() => return Ok(()),
            Ok(s) => return Err(app_error("CLIPBOARD_ERROR", format!("剪贴板复制失败，进程退出码: {:?}", s.code()))),
            Err(e) => return Err(app_error("CLIPBOARD_ERROR", format!("执行剪贴板命令失败: {}", e))),
        }
    }
    #[cfg(target_os = "windows")]
    {
        let escaped = path.replace('\'', "''");
        let script = format!("Set-Clipboard -Path '{}'", escaped);
        let status = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .status();
        match status {
            Ok(s) if s.success() => return Ok(()),
            Ok(s) => return Err(app_error("CLIPBOARD_ERROR", format!("剪贴板复制失败: {:?}", s.code()))),
            Err(e) => return Err(app_error("CLIPBOARD_ERROR", format!("执行剪贴板命令失败: {}", e))),
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Ok(())
    }
}

#[tauri::command]
fn read_file_as_data_url(path: String) -> CommandResult<String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(app_error("FILE_NOT_FOUND", "文件不存在。"));
    }
    let metadata = fs::metadata(p).map_err(|e| app_error("FILE_READ_ERROR", format!("无法读取文件信息: {}", e)))?;
    if metadata.len() > 15 * 1024 * 1024 {
        return Err(app_error("FILE_TOO_LARGE", "文件过大，无法生成缩略图。"));
    }
    let bytes = fs::read(p).map_err(|e| app_error("FILE_READ_ERROR", format!("读取文件失败: {}", e)))?;
    let encoded = BASE64.encode(&bytes);
    let lower = path.to_lowercase();
    let mime = if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "image/gif"
    };
    Ok(format!("data:{};base64,{}", mime, encoded))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RenderState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_info,
            media_probe,
            render_gif,
            cancel_render,
            copy_file_to_clipboard,
            read_file_as_data_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fractional_frame_rate() {
        assert!((parse_ratio("30000/1001").unwrap() - 29.970029).abs() < 0.0001);
        assert!(parse_ratio("0/0").is_none());
    }

    #[test]
    fn escapes_drawtext_filter_delimiters() {
        assert_eq!(escape_drawtext("a:b, 100%"), "a\\:b\\, 100\\%");
    }

    #[test]
    fn builds_palette_filter_graph() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            crop: None,
            output_width: 480,
            fps: 12,
            quality: None,
            text_overlay: None,
            image_overlay: None,
            format: None,
        };
        let graph = build_filter_graph(&plan);
        assert!(graph.contains("palettegen"));
        assert!(graph.contains("paletteuse=dither=sierra2_4a"));
        assert!(graph.contains("scale=480:-1"));
    }

    #[test]
    fn builds_drawtext_filter_graph_with_font() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            crop: None,
            output_width: 320,
            fps: 10,
            quality: Some("low".to_string()),
            text_overlay: Some(TextOverlay {
                text: "测试文字".to_string(),
                x: 10,
                y: 20,
                size: 24,
                color: "#ff0000".to_string(),
                stroke_width: Some(3),
                stroke_color: Some("#000000".to_string()),
                use_box: Some(true),
                box_color: Some("#111111".to_string()),
                box_opacity: Some(0.65),
            }),
            image_overlay: None,
            format: Some("gif".to_string()),
        };
        let graph = build_filter_graph(&plan);
        assert!(graph.contains("drawtext="));
        assert!(graph.contains("测试文字"));
        assert!(graph.contains("borderw=3"));
        assert!(graph.contains("bordercolor=0x000000"));
        assert!(graph.contains("box=1:boxcolor=0x111111@0.65"));
        assert!(graph.contains("paletteuse=dither=bayer:bayer_scale=3"));
    }

    #[test]
    fn builds_drawtext_filter_graph_without_box_and_stroke() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            crop: None,
            output_width: 320,
            fps: 10,
            quality: None,
            text_overlay: Some(TextOverlay {
                text: "无底框纯字".to_string(),
                x: 10,
                y: 20,
                size: 24,
                color: "#ffffff".to_string(),
                stroke_width: None,
                stroke_color: None,
                use_box: Some(false),
                box_color: None,
                box_opacity: None,
            }),
            image_overlay: None,
            format: None,
        };
        let graph = build_filter_graph(&plan);
        assert!(graph.contains("box=0"));
        assert!(!graph.contains("borderw="));
    }

    #[test]
    fn builds_webp_filter_graph_without_palette() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.webp".to_string(),
            start_secs: 0.0,
            end_secs: 2.0,
            crop: None,
            output_width: 480,
            fps: 15,
            quality: Some("high".to_string()),
            text_overlay: None,
            image_overlay: None,
            format: Some("webp".to_string()),
        };
        let graph = build_filter_graph(&plan);
        assert!(!graph.contains("palettegen"));
        assert!(!graph.contains("paletteuse"));
        assert!(graph.ends_with("[vout]"));

        let args = build_ffmpeg_args(&plan);
        assert!(args.iter().any(|arg| arg == "libwebp"));
        assert!(args.iter().any(|arg| arg == "-q:v"));
        assert!(args.iter().any(|arg| arg == "90"));
        assert!(!args.iter().any(|arg| arg == "-f"));
    }

    #[test]
    fn validates_render_plan_supports_gif_and_webp() {
        let media = MediaInfo {
            path: "in.mp4".to_string(),
            file_name: "in.mp4".to_string(),
            format: "mov,mp4,m4a,3gp,3g2,mj2".to_string(),
            duration_secs: 5.0,
            width: 1280,
            height: 720,
            fps: 30.0,
        };

        let temp_dir = env::temp_dir();
        let webp_out = temp_dir.join("test_valid.webp");
        let plan = RenderPlan {
            input_path: "in.mp4".to_string(),
            output_path: webp_out.to_string_lossy().to_string(),
            start_secs: 0.0,
            end_secs: 2.0,
            crop: None,
            output_width: 320,
            fps: 15,
            quality: None,
            text_overlay: None,
            image_overlay: None,
            format: Some("webp".to_string()),
        };
        assert!(validate_render_plan(&plan, &media).is_ok());

        // 测试格式不匹配
        let mismatch_plan = RenderPlan {
            output_path: webp_out.to_string_lossy().to_string(),
            format: Some("gif".to_string()),
            ..plan.clone()
        };
        assert!(validate_render_plan(&mismatch_plan, &media).is_err());
    }

    #[test]
    fn resolves_binary_with_env_override() {
        env::set_var("MOCK_BIN_BINARY", "/custom/bin/mock");
        let resolved = resolve_binary("mock_bin");
        assert_eq!(resolved, PathBuf::from("/custom/bin/mock"));
        env::remove_var("MOCK_BIN_BINARY");
    }

    #[test]
    fn read_file_as_data_url_fails_on_missing_file() {
        let res = read_file_as_data_url("/path/to/nonexistent/file.gif".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "FILE_NOT_FOUND");
    }

    #[test]
    fn read_file_as_data_url_encodes_valid_file() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("test_sample.gif");
        fs::write(&test_file, b"GIF89a_test_bytes").unwrap();
        let res = read_file_as_data_url(test_file.to_string_lossy().to_string());
        assert!(res.is_ok());
        let url = res.unwrap();
        assert!(url.starts_with("data:image/gif;base64,"));
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn copy_file_to_clipboard_fails_on_missing_file() {
        let res = copy_file_to_clipboard("/path/to/nonexistent/file.gif".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "FILE_NOT_FOUND");
    }
}
