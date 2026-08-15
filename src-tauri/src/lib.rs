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
    text_overlay: Option<TextOverlay>,
    image_overlay: Option<ImageOverlay>,
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

fn binary_from_env(name: &str) -> String {
    let variable = format!("{}_BINARY", name.to_ascii_uppercase());
    env::var(variable).unwrap_or_else(|_| name.to_string())
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
    let output = Command::new(binary_from_env("ffprobe"))
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
                format!("无法启动 ffprobe：{}。请安装 FFmpeg 或设置 FFPROBE_BINARY。", error),
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
    if output_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase() != "gif")
        .unwrap_or(true)
    {
        return Err(app_error("INVALID_OUTPUT", "输出文件必须使用 .gif 扩展名。"));
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
    format!("0x{}", &value[1..])
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
        source.push_str(&format!(
            ",drawtext=text='{}':fontcolor={}:fontsize={}:x={}:y={}:box=1:boxcolor=black@0.45:boxborderw=8",
            escape_drawtext(&text.text),
            color_for_filter(&text.color),
            text.size,
            text.x,
            text.y
        ));
    }

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
    graph.push_str(&format!(
        ";{}split[gif][palette];[palette]palettegen=stats_mode=diff[p];[gif][p]paletteuse=dither=sierra2_4a[vout]",
        final_label
    ));
    graph
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
    args
}

fn ffmpeg_supports_filter(filter_name: &str) -> Option<bool> {
    let output = Command::new(binary_from_env("ffmpeg"))
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
    let mut child = Command::new(binary_from_env("ffmpeg"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            app_error(
                "FFMPEG_UNAVAILABLE",
                format!("无法启动 ffmpeg：{}。请安装 FFmpeg 或设置 FFMPEG_BINARY。", error),
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
            cancel_render
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
            text_overlay: None,
            image_overlay: None,
        };
        let graph = build_filter_graph(&plan);
        assert!(graph.contains("palettegen"));
        assert!(graph.contains("paletteuse"));
        assert!(graph.contains("scale=480:-1"));
    }
}
