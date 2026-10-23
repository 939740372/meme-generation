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
    #[serde(default)]
    has_audio: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
struct BgmOverlay {
    path: String,
    #[serde(default)]
    volume: Option<f64>,
    #[serde(default)]
    orig_volume: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BuiltinSfx {
    id: String,
    name: String,
    category: String,
    path: String,
    data_url: String,
    duration: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CropArea {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TextOverlay {
    #[allow(dead_code)]
    #[serde(default)]
    id: Option<String>,
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
    #[serde(default)]
    visible: Option<bool>,
    pub rotation: Option<i32>,
    pub arc_radius: Option<i32>,
    pub is_italic: Option<bool>,
    pub shadow_offset: Option<u32>,
    pub shadow_color: Option<String>,
}

impl TextOverlay {
    pub fn has_advanced_transform(&self) -> bool {
        self.rotation.unwrap_or(0) != 0
            || self.is_italic.unwrap_or(false)
            || self.arc_radius.unwrap_or(0) != 0
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ImageOverlay {
    path: String,
    x: u32,
    y: u32,
    width: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ImageAdjustments {
    #[serde(default)]
    brightness: Option<i32>,
    #[serde(default)]
    contrast: Option<i32>,
    #[serde(default)]
    saturation: Option<i32>,
    #[serde(default)]
    sharpen: Option<u32>,
    #[serde(default)]
    blur: Option<u32>,
    #[serde(default)]
    hue: Option<i32>,
    #[serde(default)]
    vignette: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
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
    #[serde(default)]
    text_layers: Option<Vec<TextOverlay>>,
    image_overlay: Option<ImageOverlay>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    video_filter: Option<String>,
    #[serde(default)]
    image_adjustments: Option<ImageAdjustments>,
    #[serde(default)]
    include_audio: Option<bool>,
    #[serde(default)]
    hw_accel: Option<String>,
    #[serde(default)]
    has_audio: Option<bool>,
    #[serde(default)]
    bgm_overlay: Option<BgmOverlay>,
    #[serde(default)]
    source_width: Option<u32>,
    #[serde(default)]
    source_height: Option<u32>,
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

    fn is_avif(&self) -> bool {
        if let Some(fmt) = &self.format {
            if fmt.eq_ignore_ascii_case("avif") {
                return true;
            }
        }
        self.output_path.to_ascii_lowercase().ends_with(".avif")
    }

    fn is_mp4(&self) -> bool {
        if let Some(fmt) = &self.format {
            if fmt.eq_ignore_ascii_case("mp4") {
                return true;
            }
        }
        self.output_path.to_ascii_lowercase().ends_with(".mp4")
    }

    fn active_text_layers(&self) -> Vec<&TextOverlay> {
        if let Some(layers) = &self.text_layers {
            if !layers.is_empty() {
                return layers
                    .iter()
                    .filter(|l| l.visible.unwrap_or(true) && !l.text.trim().is_empty())
                    .collect();
            }
        }
        if let Some(single) = &self.text_overlay {
            if single.visible.unwrap_or(true) && !single.text.trim().is_empty() {
                return vec![single];
            }
        }
        Vec::new()
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

    let target_triple = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        _ => "",
    };

    let binary_filename = if cfg!(windows) {
        if target_triple.is_empty() {
            format!("{}.exe", name)
        } else {
            format!("{}-{}.exe", name, target_triple)
        }
    } else if target_triple.is_empty() {
        name.to_string()
    } else {
        format!("{}-{}", name, target_triple)
    };

    // 2. 检查应用可执行文件同级或 Resources 目录（生产打包环境）
    if let Ok(current_exe) = env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let prod_candidates = [
                exe_dir.join(name),
                exe_dir.join(&binary_filename),
                exe_dir.join(format!("{}.exe", name)),
                exe_dir.join("../Resources").join(name),
                exe_dir.join("../Resources").join(&binary_filename),
                exe_dir.join("../Resources/binaries").join(name),
                exe_dir.join("../Resources/binaries").join(&binary_filename),
            ];
            for candidate in prod_candidates {
                if candidate.is_file() {
                    return candidate;
                }
            }
        }
    }

    // 3. 检查开发环境下的 src-tauri/binaries 候选路径
    let dev_candidates = [
        PathBuf::from("src-tauri").join("binaries").join(&binary_filename),
        PathBuf::from("binaries").join(&binary_filename),
        PathBuf::from("..").join("src-tauri").join("binaries").join(&binary_filename),
        PathBuf::from("src-tauri").join("binaries").join(name),
        PathBuf::from("binaries").join(name),
        PathBuf::from("..").join("src-tauri").join("binaries").join(name),
    ];

    for candidate in dev_candidates {
        if candidate.is_file() {
            return candidate;
        }
    }

    // 4. 回退到系统的 PATH
    PathBuf::from(name)
}

fn new_hidden_command<S: AsRef<std::ffi::OsStr>>(program: S) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

fn detect_system_font() -> Option<String> {
    let candidates: &[&str] = if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Supplemental/Songti.ttc",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            "/Library/Fonts/Arial Unicode.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
        ]
    } else if cfg!(target_os = "windows") {
        &[
            "C:/Windows/Fonts/msyh.ttc",
            "C:/Windows/Fonts/msyhbd.ttc",
            "C:/Windows/Fonts/simhei.ttf",
            "C:/Windows/Fonts/simsun.ttc",
            "C:/Windows/Fonts/arial.ttf",
        ]
    } else {
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ]
    };

    for font in candidates {
        if Path::new(font).exists() {
            return Some((*font).replace('\\', "/"));
        }
    }
    None
}

fn path_to_string(path: &Path) -> CommandResult<String> {
    path.to_str()
        .map(|s| s.strip_prefix(r"\\?\").unwrap_or(s).to_string())
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
    let output = new_hidden_command(&ffprobe_bin)
        .args([
            "-v",
            "error",
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
    let streams = document
        .get("streams")
        .and_then(Value::as_array)
        .ok_or_else(|| app_error("NO_VIDEO_STREAM", "输入文件没有可用的视频流。"))?;

    let stream = streams
        .iter()
        .find(|s| s.get("codec_type").and_then(Value::as_str) == Some("video"))
        .ok_or_else(|| app_error("NO_VIDEO_STREAM", "输入文件没有可用的视频流。"))?;

    let has_audio = streams
        .iter()
        .any(|s| s.get("codec_type").and_then(Value::as_str) == Some("audio"));

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
        has_audio,
    })
}

fn check_has_audio_stream(ffprobe_bin: &Path, path: &str) -> bool {
    let output = new_hidden_command(ffprobe_bin)
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "json",
            path,
        ])
        .output();
    if let Ok(out) = output {
        if let Ok(val) = serde_json::from_slice::<Value>(&out.stdout) {
            if let Some(arr) = val.get("streams").and_then(|s| s.as_array()) {
                return !arr.is_empty();
            }
        }
    }
    false
}

fn check_file_has_audio(path: &Path) -> bool {
    let ffprobe_bin = resolve_binary("ffprobe");
    if let Ok(path_str) = path_to_string(path) {
        check_has_audio_stream(&ffprobe_bin, &path_str)
    } else {
        false
    }
}

fn input_has_audio(plan: &RenderPlan) -> bool {
    if let Some(ha) = plan.has_audio {
        return ha;
    }
    let p = Path::new(&plan.input_path);
    if p.exists() {
        check_file_has_audio(p)
    } else {
        true
    }
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

    let active_layers = plan.active_text_layers();
    if active_layers.len() > 6 {
        return Err(app_error("INVALID_TEXT_LAYERS", "文字图层最多支持 6 个。"));
    }
    for text in &active_layers {
        if text.text.trim().is_empty() {
            return Err(app_error("INVALID_TEXT", "文字叠加内容不能为空。"));
        }
        if text.text.chars().any(char::is_control) {
            return Err(app_error("INVALID_TEXT", "文字叠加不能包含控制字符。"));
        }
        if !(8..=512).contains(&text.size) || !is_hex_color(&text.color) {
            return Err(app_error("INVALID_TEXT_STYLE", "文字字号或颜色格式无效。"));
        }
        if let Some(stroke_c) = &text.stroke_color {
            if !is_hex_color(stroke_c) {
                return Err(app_error("INVALID_TEXT_STYLE", "文字描边颜色格式无效。"));
            }
        }
        if let Some(box_c) = &text.box_color {
            if !is_hex_color(box_c) {
                return Err(app_error("INVALID_TEXT_STYLE", "文字底框颜色格式无效。"));
            }
        }
        if let Some(rot) = text.rotation {
            if !(-360..=360).contains(&rot) {
                return Err(app_error("INVALID_TEXT_STYLE", "文字旋转角度必须在 -360° 到 360° 之间。"));
            }
        }
        if let Some(arc) = text.arc_radius {
            if !(-100..=100).contains(&arc) {
                return Err(app_error("INVALID_TEXT_STYLE", "文字弧度必须在 -100 到 100 之间。"));
            }
        }
        if let Some(shadow_off) = text.shadow_offset {
            if shadow_off > 100 {
                return Err(app_error("INVALID_TEXT_STYLE", "文字阴影偏移不能超过 100。"));
            }
        }
        if let Some(shadow_c) = &text.shadow_color {
            if !is_hex_color(shadow_c) {
                return Err(app_error("INVALID_TEXT_STYLE", "文字阴影颜色格式无效。"));
            }
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
        if image.width == 0 || image.width > 3840 {
            return Err(app_error("INVALID_IMAGE_SIZE", "图案宽度必须在 1 到 3840 之间。"));
        }
    }

    if let Some(adj) = &plan.image_adjustments {
        if let Some(b) = adj.brightness {
            if !(-100..=100).contains(&b) {
                return Err(app_error("INVALID_IMAGE_ADJUSTMENTS", "亮度调节范围必须在 -100 到 100 之间。"));
            }
        }
        if let Some(c) = adj.contrast {
            if !(-100..=100).contains(&c) {
                return Err(app_error("INVALID_IMAGE_ADJUSTMENTS", "对比度调节范围必须在 -100 到 100 之间。"));
            }
        }
        if let Some(s) = adj.saturation {
            if !(-100..=100).contains(&s) {
                return Err(app_error("INVALID_IMAGE_ADJUSTMENTS", "饱和度调节范围必须在 -100 到 100 之间。"));
            }
        }
        if let Some(sh) = adj.sharpen {
            if sh > 100 {
                return Err(app_error("INVALID_IMAGE_ADJUSTMENTS", "锐化强度必须在 0 到 100 之间。"));
            }
        }
        if let Some(bl) = adj.blur {
            if bl > 100 {
                return Err(app_error("INVALID_IMAGE_ADJUSTMENTS", "模糊强度必须在 0 到 100 之间。"));
            }
        }
        if let Some(h) = adj.hue {
            if !(-180..=180).contains(&h) {
                return Err(app_error("INVALID_IMAGE_ADJUSTMENTS", "色相偏移必须在 -180° 到 180° 之间。"));
            }
        }
    }

    if let Some(bgm) = &plan.bgm_overlay {
        if !bgm.path.trim().is_empty() {
            let bgm_path = Path::new(&bgm.path);
            validate_input_file(bgm_path)?;
            if let Some(vol) = bgm.volume {
                if !vol.is_finite() || !(0.0..=10.0).contains(&vol) {
                    return Err(app_error("INVALID_BGM_VOLUME", "背景音乐音量必须在 0 到 1000% 之间。"));
                }
            }
            if let Some(orig_vol) = bgm.orig_volume {
                if !orig_vol.is_finite() || !(0.0..=10.0).contains(&orig_vol) {
                    return Err(app_error("INVALID_ORIG_VOLUME", "原声音量必须在 0 到 1000% 之间。"));
                }
            }
        }
    }

    let output_path = Path::new(&plan.output_path);
    let ext = output_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase());
    match ext.as_deref() {
        Some("gif") | Some("webp") | Some("avif") | Some("mp4") => {}
        _ => {
            return Err(app_error(
                "INVALID_OUTPUT",
                "输出文件必须使用 .gif、.webp、.avif 或 .mp4 扩展名。",
            ));
        }
    }
    if let Some(fmt) = &plan.format {
        let fmt_lower = fmt.to_ascii_lowercase();
        if (fmt_lower == "webp" && ext.as_deref() != Some("webp"))
            || (fmt_lower == "gif" && ext.as_deref() != Some("gif"))
            || (fmt_lower == "avif" && ext.as_deref() != Some("avif"))
            || (fmt_lower == "mp4" && ext.as_deref() != Some("mp4"))
        {
            return Err(app_error(
                "INVALID_OUTPUT_FORMAT",
                "输出格式设置与文件扩展名不一致。",
            ));
        }
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

fn parse_numbered_stem(stem: &str) -> Option<(String, usize)> {
    if !stem.ends_with(')') {
        return None;
    }
    let open_paren = stem.rfind(" (")?;
    let inner_num_str = &stem[open_paren + 2..stem.len() - 1];
    let num = inner_num_str.parse::<usize>().ok()?;
    if num == 0 {
        return None;
    }
    let base_stem = stem[..open_paren].to_string();
    Some((base_stem, num + 1))
}

pub fn resolve_unique_output_path(target: &Path) -> PathBuf {
    if !target.exists() {
        return target.to_path_buf();
    }

    let parent = target.parent().unwrap_or_else(|| Path::new(""));
    let ext = target.extension().and_then(|e| e.to_str()).unwrap_or("");
    let stem = target.file_stem().and_then(|s| s.to_str()).unwrap_or("output");

    let (base_stem, start_num) = if let Some(caps) = parse_numbered_stem(stem) {
        caps
    } else {
        (stem.to_string(), 1)
    };

    let mut counter = start_num;
    while counter < 10000 {
        let new_file_name = if ext.is_empty() {
            format!("{} ({})", base_stem, counter)
        } else {
            format!("{} ({}).{}", base_stem, counter, ext)
        };

        let candidate = if parent.as_os_str().is_empty() {
            PathBuf::from(new_file_name)
        } else {
            parent.join(new_file_name)
        };

        if !candidate.exists() {
            return candidate;
        }

        counter += 1;
    }

    target.to_path_buf()
}

fn format_secs(value: f64) -> String {
    format!("{value:.3}")
}

fn escape_drawtext(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 16);
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("'\\''"),
            ':' => out.push_str("\\:"),
            ',' => out.push_str("\\,"),
            '%' => out.push_str("\\%"),
            '\r' => {}
            other => out.push(other),
        }
    }
    out
}

fn color_for_filter(value: &str) -> String {
    if let Some(hex) = value.strip_prefix('#') {
        format!("0x{}", hex)
    } else {
        value.to_string()
    }
}

fn map_video_filter(key: &str) -> Option<&'static str> {
    match key.trim().to_ascii_lowercase().as_str() {
        "grayscale" => Some("hue=s=0"),
        "contrast" => Some("eq=contrast=1.6:saturation=1.4"),
        "vintage" => Some("colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131"),
        "invert" => Some("negate"),
        "fast2x" => Some("setpts=0.5*PTS"),
        "slow05x" => Some("setpts=2.0*PTS"),
        "reverse" => Some("reverse"),
        _ => None,
    }
}

fn build_image_adjustments_filter(adj: &ImageAdjustments) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();

    let brightness = adj.brightness.unwrap_or(0);
    let contrast = adj.contrast.unwrap_or(0);
    let saturation = adj.saturation.unwrap_or(0);
    if brightness != 0 || contrast != 0 || saturation != 0 {
        let mut eq_params: Vec<String> = Vec::new();
        if brightness != 0 {
            let b_val = (brightness as f64 * 0.005).clamp(-0.50, 0.50);
            eq_params.push(format!("brightness={:.3}", b_val));
        }
        if contrast != 0 {
            let c_val = (1.0 + contrast as f64 * 0.01).clamp(0.0, 2.0);
            eq_params.push(format!("contrast={:.2}", c_val));
        }
        if saturation != 0 {
            let s_val = if saturation >= 0 {
                (1.0 + saturation as f64 * 0.015).clamp(1.0, 2.5)
            } else {
                (1.0 + saturation as f64 * 0.01).clamp(0.0, 1.0)
            };
            eq_params.push(format!("saturation={:.2}", s_val));
        }
        parts.push(format!("eq={}", eq_params.join(":")));
    }

    let hue = adj.hue.unwrap_or(0);
    if hue != 0 {
        parts.push(format!("hue=h={}", hue.clamp(-180, 180)));
    }

    let sharpen = adj.sharpen.unwrap_or(0);
    if sharpen > 0 {
        let amount = ((sharpen.min(100) as f64 / 100.0) * 2.50).clamp(0.03, 2.50);
        parts.push(format!("unsharp=5:5:{:.2}:5:5:0.0", amount));
    }

    let blur = adj.blur.unwrap_or(0);
    if blur > 0 {
        let sigma = ((blur.min(100) as f64 / 100.0) * 8.00).clamp(0.08, 8.00);
        parts.push(format!("gblur=sigma={:.2}", sigma));
    }

    if adj.vignette.unwrap_or(false) {
        parts.push("vignette=PI/4".to_string());
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(","))
    }
}

fn scale_text_overlay(text: &TextOverlay, crop_x: u32, crop_y: u32, scale_factor: f64) -> TextOverlay {
    if (scale_factor - 1.0).abs() < 1e-6 && crop_x == 0 && crop_y == 0 {
        return text.clone();
    }
    let rel_x = (text.x as f64 - crop_x as f64) * scale_factor;
    let rel_y = (text.y as f64 - crop_y as f64) * scale_factor;
    let scaled_x = rel_x.round().max(0.0) as u32;
    let scaled_y = rel_y.round().max(0.0) as u32;
    let scaled_size = ((text.size as f64) * scale_factor).round().clamp(8.0, 512.0) as u32;
    let scaled_stroke = text.stroke_width.map(|sw| {
        if sw == 0 {
            0
        } else {
            ((sw as f64) * scale_factor).round().clamp(1.0, 32.0) as u32
        }
    });
    let scaled_shadow = text.shadow_offset.map(|so| {
        if so == 0 {
            0
        } else {
            ((so as f64) * scale_factor).round().clamp(1.0, 64.0) as u32
        }
    });
    TextOverlay {
        id: text.id.clone(),
        text: text.text.clone(),
        x: scaled_x,
        y: scaled_y,
        size: scaled_size,
        color: text.color.clone(),
        stroke_width: scaled_stroke,
        stroke_color: text.stroke_color.clone(),
        use_box: text.use_box,
        box_color: text.box_color.clone(),
        box_opacity: text.box_opacity,
        visible: text.visible,
        rotation: text.rotation,
        arc_radius: text.arc_radius,
        is_italic: text.is_italic,
        shadow_offset: scaled_shadow,
        shadow_color: text.shadow_color.clone(),
    }
}

fn scale_image_overlay(image: &ImageOverlay, crop_x: u32, crop_y: u32, scale_factor: f64) -> ImageOverlay {
    if (scale_factor - 1.0).abs() < 1e-6 && crop_x == 0 && crop_y == 0 {
        return image.clone();
    }
    let rel_x = (image.x as f64 - crop_x as f64) * scale_factor;
    let rel_y = (image.y as f64 - crop_y as f64) * scale_factor;
    let scaled_x = rel_x.round().max(0.0) as u32;
    let scaled_y = rel_y.round().max(0.0) as u32;
    let scaled_width = ((image.width as f64) * scale_factor).round().clamp(8.0, 1920.0) as u32;
    ImageOverlay {
        path: image.path.clone(),
        x: scaled_x,
        y: scaled_y,
        width: scaled_width,
    }
}

fn build_filter_graph(plan: &RenderPlan) -> String {
    let crop_x = plan.crop.as_ref().map(|c| c.x).unwrap_or(0);
    let crop_y = plan.crop.as_ref().map(|c| c.y).unwrap_or(0);
    let base_width = plan
        .crop
        .as_ref()
        .map(|c| c.width)
        .or(plan.source_width)
        .unwrap_or(plan.output_width)
        .max(1);
    let scale_factor = (plan.output_width.max(1) as f64) / (base_width as f64);

    let active_layers: Vec<TextOverlay> = plan
        .active_text_layers()
        .into_iter()
        .map(|l| scale_text_overlay(l, crop_x, crop_y, scale_factor))
        .collect();
    let scaled_image_overlay = plan
        .image_overlay
        .as_ref()
        .map(|img| scale_image_overlay(img, crop_x, crop_y, scale_factor));
    let has_advanced = active_layers.iter().any(|l| l.has_advanced_transform());

    if !has_advanced {
        let mut source = String::new();
        if let Some(crop) = &plan.crop {
            source.push_str(&format!("crop={}:{}:{}:{},", crop.width, crop.height, crop.x, crop.y));
        }
        let scale_h = if plan.is_mp4() || plan.is_avif() { -2 } else { -1 };
        source.push_str(&format!(
            "scale={}:{}:flags=lanczos",
            plan.output_width, scale_h
        ));
        source.push_str(&format!(",fps={}", plan.fps));

        if let Some(filter_key) = &plan.video_filter {
            if let Some(expr) = map_video_filter(filter_key) {
                source.push_str(&format!(",{}", expr));
            }
        }
        if let Some(adj) = &plan.image_adjustments {
            if let Some(adj_expr) = build_image_adjustments_filter(adj) {
                source.push_str(&format!(",{}", adj_expr));
            }
        }

        let fontfile_opt = detect_system_font();
        for text in &active_layers {
            let mut drawtext_parts = Vec::new();
            if let Some(fontfile) = &fontfile_opt {
                drawtext_parts.push(format!("fontfile='{}'", escape_drawtext(fontfile)));
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

            if let Some(offset) = text.shadow_offset {
                if offset > 0 {
                    let shadow_c = text.shadow_color.as_deref().unwrap_or("#000000");
                    drawtext_parts.push(format!(
                        "shadowx={}:shadowy={}:shadowcolor={}",
                        offset,
                        offset,
                        color_for_filter(shadow_c)
                    ));
                }
            }

            source.push_str(&format!(",drawtext={}", drawtext_parts.join(":")));
        }

        if plan.is_webp() {
            if let Some(image) = &scaled_image_overlay {
                format!(
                    "[0:v]{}[base];[1:v]scale={}:{}:flags=lanczos[sticker];[base][sticker]overlay={}:{}:eof_action=pass[vout]",
                    source, image.width, -1, image.x, image.y
                )
            } else {
                format!("[0:v]{}[vout]", source)
            }
        } else if plan.is_avif() {
            if let Some(image) = &scaled_image_overlay {
                format!(
                    "[0:v]{}[base];[1:v]scale={}:{}:flags=lanczos[sticker];[base][sticker]overlay={}:{}:eof_action=pass,format=yuv420p[vout]",
                    source, image.width, -1, image.x, image.y
                )
            } else {
                format!("[0:v]{},format=yuv420p[vout]", source)
            }
        } else if plan.is_mp4() {
            if let Some(image) = &scaled_image_overlay {
                format!(
                    "[0:v]{}[base];[1:v]scale={}:{}:flags=lanczos[sticker];[base][sticker]overlay={}:{}:eof_action=pass,format=yuv420p[vout]",
                    source, image.width, -1, image.x, image.y
                )
            } else {
                format!("[0:v]{},format=yuv420p[vout]", source)
            }
        } else {
            let mut graph = format!("[0:v]{}[base]", source);
            let final_label = if let Some(image) = &scaled_image_overlay {
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
    } else {
        // Advanced text pipeline (rotation, shear, arc curvature)
        let mut graph_parts: Vec<String> = Vec::new();
        let mut base_source = String::new();
        if let Some(crop) = &plan.crop {
            base_source.push_str(&format!("crop={}:{}:{}:{},", crop.width, crop.height, crop.x, crop.y));
        }
        let scale_h = if plan.is_mp4() || plan.is_avif() { -2 } else { -1 };
        base_source.push_str(&format!(
            "scale={}:{}:flags=lanczos",
            plan.output_width, scale_h
        ));
        base_source.push_str(&format!(",fps={}", plan.fps));

        if let Some(filter_key) = &plan.video_filter {
            if let Some(expr) = map_video_filter(filter_key) {
                base_source.push_str(&format!(",{}", expr));
            }
        }
        if let Some(adj) = &plan.image_adjustments {
            if let Some(adj_expr) = build_image_adjustments_filter(adj) {
                base_source.push_str(&format!(",{}", adj_expr));
            }
        }

        graph_parts.push(format!("[0:v]{}[v0]", base_source));
        let mut current_stream = "[v0]".to_string();
        let fontfile_opt = detect_system_font();
        let base_duration = (plan.end_secs - plan.start_secs).max(0.1);
        let duration = if plan
            .video_filter
            .as_deref()
            .map(|s| s.trim().eq_ignore_ascii_case("slow05x"))
            .unwrap_or(false)
        {
            base_duration * 2.0
        } else {
            base_duration
        };

        for (layer_idx, text) in active_layers.iter().enumerate() {
            if !text.has_advanced_transform() {
                let next_stream = format!("[v_{}]", layer_idx + 1);
                let mut drawtext_parts = Vec::new();
                if let Some(fontfile) = &fontfile_opt {
                    drawtext_parts.push(format!("fontfile='{}'", escape_drawtext(fontfile)));
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

                if let Some(offset) = text.shadow_offset {
                    if offset > 0 {
                        let shadow_c = text.shadow_color.as_deref().unwrap_or("#000000");
                        drawtext_parts.push(format!(
                            "shadowx={}:shadowy={}:shadowcolor={}",
                            offset,
                            offset,
                            color_for_filter(shadow_c)
                        ));
                    }
                }

                graph_parts.push(format!(
                    "{}drawtext={}{}",
                    current_stream,
                    drawtext_parts.join(":"),
                    next_stream
                ));
                current_stream = next_stream;
            } else if text.arc_radius.unwrap_or(0) == 0 {
                // Rotated and/or sheared single layer
                let est_w = (text.text.chars().map(|c| if c.is_ascii() { 0.6 } else { 1.0 }).sum::<f64>() * text.size as f64).round() as u32;
                let est_h = text.size;
                let pad_w = ((est_w * 2 + 120).max(200) + 1) & !1;
                let pad_h = ((est_h * 3 + 120).max(120) + 1) & !1;

                let mut drawtext_parts = Vec::new();
                if let Some(fontfile) = &fontfile_opt {
                    drawtext_parts.push(format!("fontfile='{}'", escape_drawtext(fontfile)));
                }
                drawtext_parts.push(format!("text='{}'", escape_drawtext(&text.text)));
                drawtext_parts.push(format!("fontcolor={}", color_for_filter(&text.color)));
                drawtext_parts.push(format!("fontsize={}", text.size));
                drawtext_parts.push("x=(w-text_w)/2".to_string());
                drawtext_parts.push("y=(h-text_h)/2".to_string());

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

                if let Some(offset) = text.shadow_offset {
                    if offset > 0 {
                        let shadow_c = text.shadow_color.as_deref().unwrap_or("#000000");
                        drawtext_parts.push(format!(
                            "shadowx={}:shadowy={}:shadowcolor={}",
                            offset,
                            offset,
                            color_for_filter(shadow_c)
                        ));
                    }
                }

                let mut pad_chain = format!(
                    "color=c=black@0.0:s={}x{}:d={:.2},format=rgba,drawtext={}",
                    pad_w, pad_h, duration, drawtext_parts.join(":")
                );

                if text.is_italic.unwrap_or(false) {
                    pad_chain.push_str(",shear=shx=-0.22:c=none");
                }

                let rot = text.rotation.unwrap_or(0);
                if rot != 0 {
                    pad_chain.push_str(&format!(
                        ",rotate={}*PI/180:c=none:ow=rotw({}*PI/180):oh=roth({}*PI/180)",
                        rot, rot, rot
                    ));
                }

                let pad_label = format!("[t_{}]", layer_idx + 1);
                let next_stream = format!("[v_{}]", layer_idx + 1);
                graph_parts.push(format!("{}{}", pad_chain, pad_label));

                let cx = text.x as f64 + est_w as f64 / 2.0;
                let cy = text.y as f64 + est_h as f64 / 2.0;
                graph_parts.push(format!(
                    "{}{}overlay={:.0}-w/2:{:.0}-h/2:eof_action=pass{}",
                    current_stream, pad_label, cx, cy, next_stream
                ));
                current_stream = next_stream;
            } else {
                // Arc curvature layer
                let arc = text.arc_radius.unwrap_or(0);
                let chars: Vec<char> = text.text.chars().collect();
                let n = chars.len();
                let est_w = (text.text.chars().map(|c| if c.is_ascii() { 0.6 } else { 1.0 }).sum::<f64>() * text.size as f64).round() as u32;
                let est_h = text.size;
                let cx_base = text.x as f64 + est_w as f64 / 2.0;
                let cy_base = text.y as f64 + est_h as f64 / 2.0;

                if n <= 1 {
                    // Single char: render rotated
                    let pad_w = ((est_w * 2 + 120).max(200) + 1) & !1;
                    let pad_h = ((est_h * 3 + 120).max(120) + 1) & !1;

                    let mut drawtext_parts = Vec::new();
                    if let Some(fontfile) = &fontfile_opt {
                        drawtext_parts.push(format!("fontfile='{}'", escape_drawtext(fontfile)));
                    }
                    drawtext_parts.push(format!("text='{}'", escape_drawtext(&text.text)));
                    drawtext_parts.push(format!("fontcolor={}", color_for_filter(&text.color)));
                    drawtext_parts.push(format!("fontsize={}", text.size));
                    drawtext_parts.push("x=(w-text_w)/2".to_string());
                    drawtext_parts.push("y=(h-text_h)/2".to_string());

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

                    if let Some(offset) = text.shadow_offset {
                        if offset > 0 {
                            let shadow_c = text.shadow_color.as_deref().unwrap_or("#000000");
                            drawtext_parts.push(format!(
                                "shadowx={}:shadowy={}:shadowcolor={}",
                                offset,
                                offset,
                                color_for_filter(shadow_c)
                            ));
                        }
                    }

                    let mut pad_chain = format!(
                        "color=c=black@0.0:s={}x{}:d={:.2},format=rgba,drawtext={}",
                        pad_w, pad_h, duration, drawtext_parts.join(":")
                    );

                    if text.is_italic.unwrap_or(false) {
                        pad_chain.push_str(",shear=shx=-0.22:c=none");
                    }

                    let rot = text.rotation.unwrap_or(0);
                    if rot != 0 {
                        pad_chain.push_str(&format!(
                            ",rotate={}*PI/180:c=none:ow=rotw({}*PI/180):oh=roth({}*PI/180)",
                            rot, rot, rot
                        ));
                    }

                    let pad_label = format!("[t_{}]", layer_idx + 1);
                    let next_stream = format!("[v_{}]", layer_idx + 1);
                    graph_parts.push(format!("{}{}", pad_chain, pad_label));
                    graph_parts.push(format!(
                        "{}{}overlay={:.0}-w/2:{:.0}-h/2:eof_action=pass{}",
                        current_stream, pad_label, cx_base, cy_base, next_stream
                    ));
                    current_stream = next_stream;
                } else {
                    let char_pad_size = ((text.size * 2 + 40).max(64) + 1) & !1;
                    let max_angle_span = 70.0 * (arc as f64 / 100.0);
                    let arc_amplitude = (arc as f64 / 100.0) * 0.22 * est_w as f64;

                    for (char_idx, &ch) in chars.iter().enumerate() {
                        if ch.is_whitespace() {
                            continue;
                        }
                        let t_k = (char_idx as f64 - (n - 1) as f64 / 2.0) / ((n - 1) as f64 / 2.0);
                        let delta_theta = t_k * (max_angle_span / 2.0);
                        let total_rot = text.rotation.unwrap_or(0) as f64 + delta_theta;
                        let delta_x = t_k * (est_w as f64 / 2.0) * (delta_theta * std::f64::consts::PI / 180.0).cos();
                        let delta_y = arc_amplitude * t_k * t_k;
                        let char_cx = cx_base + delta_x;
                        let char_cy = cy_base + delta_y;

                        let mut drawtext_parts = Vec::new();
                        if let Some(fontfile) = &fontfile_opt {
                            drawtext_parts.push(format!("fontfile='{}'", escape_drawtext(fontfile)));
                        }
                        drawtext_parts.push(format!("text='{}'", escape_drawtext(&ch.to_string())));
                        drawtext_parts.push(format!("fontcolor={}", color_for_filter(&text.color)));
                        drawtext_parts.push(format!("fontsize={}", text.size));
                        drawtext_parts.push("x=(w-text_w)/2".to_string());
                        drawtext_parts.push("y=(h-text_h)/2".to_string());

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

                        if let Some(offset) = text.shadow_offset {
                            if offset > 0 {
                                let shadow_c = text.shadow_color.as_deref().unwrap_or("#000000");
                                drawtext_parts.push(format!(
                                    "shadowx={}:shadowy={}:shadowcolor={}",
                                    offset,
                                    offset,
                                    color_for_filter(shadow_c)
                                ));
                            }
                        }

                        let mut pad_chain = format!(
                            "color=c=black@0.0:s={}x{}:d={:.2},format=rgba,drawtext={}",
                            char_pad_size, char_pad_size, duration, drawtext_parts.join(":")
                        );

                        if text.is_italic.unwrap_or(false) {
                            pad_chain.push_str(",shear=shx=-0.22:c=none");
                        }

                        if total_rot.abs() > 0.1 {
                            pad_chain.push_str(&format!(
                                ",rotate={:.1}*PI/180:c=none:ow=rotw({:.1}*PI/180):oh=roth({:.1}*PI/180)",
                                total_rot, total_rot, total_rot
                            ));
                        }

                        let pad_label = format!("[tc_{}_{}]", layer_idx + 1, char_idx);
                        let next_stream = format!("[v_{}_{}]", layer_idx + 1, char_idx);
                        graph_parts.push(format!("{}{}", pad_chain, pad_label));
                        graph_parts.push(format!(
                            "{}{}overlay={:.0}-w/2:{:.0}-h/2:eof_action=pass{}",
                            current_stream, pad_label, char_cx, char_cy, next_stream
                        ));
                        current_stream = next_stream;
                    }
                }
            }
        }

        if let Some(image) = &scaled_image_overlay {
            graph_parts.push(format!(
                "[1:v]scale={}:{}:flags=lanczos[sticker];{}[sticker]overlay={}:{}:eof_action=pass[composed]",
                image.width, -1, current_stream, image.x, image.y
            ));
            current_stream = "[composed]".to_string();
        }

        if plan.is_webp() {
            graph_parts.push(format!("{}[vout]", current_stream));
        } else if plan.is_avif() || plan.is_mp4() {
            graph_parts.push(format!("{}format=yuv420p[vout]", current_stream));
        } else {
            let dither = match plan.quality.as_deref() {
                Some("low") => "bayer:bayer_scale=3",
                Some("high") => "sierra2_4a",
                Some("none") => "none",
                _ => "sierra2_4a",
            };
            graph_parts.push(format!(
                "{}split[gif][palette];[palette]palettegen=stats_mode=diff[p];[gif][p]paletteuse=dither={}[vout]",
                current_stream, dither
            ));
        }

        graph_parts.join(";")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HardwareAccelerationInfo {
    pub supported: bool,
    pub engine: String,
    pub encoder: String,
    pub description: String,
}

pub fn detect_hardware_acceleration(ffmpeg_bin: &Path) -> HardwareAccelerationInfo {
    let output = new_hidden_command(ffmpeg_bin)
        .args(["-hide_banner", "-encoders"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}\n{}", stdout, stderr);

            // 1. macOS VideoToolbox (Apple Silicon / Intel Mac GPU)
            if cfg!(target_os = "macos") && combined.contains("h264_videotoolbox") {
                return HardwareAccelerationInfo {
                    supported: true,
                    engine: "VideoToolbox".to_string(),
                    encoder: "h264_videotoolbox".to_string(),
                    description: "Apple VideoToolbox 硬件加速 (Media Engine)".to_string(),
                };
            }

            // 2. NVIDIA NVENC (Windows / Linux)
            if combined.contains("h264_nvenc") {
                return HardwareAccelerationInfo {
                    supported: true,
                    engine: "NVENC".to_string(),
                    encoder: "h264_nvenc".to_string(),
                    description: "NVIDIA NVENC 硬件加速".to_string(),
                };
            }

            // 3. Intel QuickSync (Windows / Linux)
            if combined.contains("h264_qsv") {
                return HardwareAccelerationInfo {
                    supported: true,
                    engine: "QuickSync".to_string(),
                    encoder: "h264_qsv".to_string(),
                    description: "Intel QuickSync 硬件加速".to_string(),
                };
            }

            // 4. Linux VAAPI
            if combined.contains("h264_vaapi") {
                return HardwareAccelerationInfo {
                    supported: true,
                    engine: "VAAPI".to_string(),
                    encoder: "h264_vaapi".to_string(),
                    description: "Linux VAAPI 硬件加速".to_string(),
                };
            }
        }
    }

    HardwareAccelerationInfo {
        supported: false,
        engine: "None".to_string(),
        encoder: "libx264".to_string(),
        description: "未检测到可用硬件编码器，使用 CPU 软件压制".to_string(),
    }
}

#[tauri::command]
fn get_hardware_acceleration_info() -> CommandResult<HardwareAccelerationInfo> {
    let ffmpeg_bin = resolve_binary("ffmpeg");
    Ok(detect_hardware_acceleration(&ffmpeg_bin))
}

fn build_ffmpeg_args_with_hw(plan: &RenderPlan, hw_info: &HardwareAccelerationInfo) -> Vec<String> {
    let duration = plan.end_secs - plan.start_secs;
    let filter_graph = build_filter_graph(plan);
    let hw_enabled = plan.hw_accel.as_deref() != Some("disabled") && hw_info.supported;

    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-nostats".to_string(),
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-n".to_string(),
    ];

    if hw_enabled {
        args.extend(["-hwaccel".to_string(), "auto".to_string()]);
    }

    args.extend([
        "-ss".to_string(),
        format_secs(plan.start_secs),
        "-t".to_string(),
        format_secs(duration),
        "-i".to_string(),
        plan.input_path.clone(),
    ]);

    if let Some(image) = &plan.image_overlay {
        args.extend([
            "-loop".to_string(),
            "1".to_string(),
            "-i".to_string(),
            image.path.clone(),
        ]);
    }

    if plan.is_mp4() {
        if let Some(bgm) = &plan.bgm_overlay {
            if !bgm.path.trim().is_empty() {
                args.extend(["-i".to_string(), bgm.path.clone()]);
            }
        }
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
    } else if plan.is_avif() {
        let crf_val = match plan.quality.as_deref() {
            Some("low") => "38",
            Some("high") => "26",
            _ => "32",
        };
        args.extend([
            "-filter_complex".to_string(),
            filter_graph,
            "-map".to_string(),
            "[vout]".to_string(),
            "-c:v".to_string(),
            "libsvtav1".to_string(),
            "-preset".to_string(),
            "8".to_string(),
            "-crf".to_string(),
            crf_val.to_string(),
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
            "-an".to_string(),
            plan.output_path.clone(),
        ]);
    } else if plan.is_mp4() {
        let (vcodec, quality_args) = if hw_enabled && hw_info.encoder == "h264_videotoolbox" {
            let q_val = match plan.quality.as_deref() {
                Some("low") => "50",
                Some("high") => "75",
                _ => "65",
            };
            ("h264_videotoolbox", vec!["-q:v".to_string(), q_val.to_string()])
        } else if hw_enabled && hw_info.encoder == "h264_nvenc" {
            let q_val = match plan.quality.as_deref() {
                Some("low") => "28",
                Some("high") => "19",
                _ => "23",
            };
            ("h264_nvenc", vec!["-cq".to_string(), q_val.to_string()])
        } else if hw_enabled && hw_info.encoder == "h264_qsv" {
            let q_val = match plan.quality.as_deref() {
                Some("low") => "28",
                Some("high") => "19",
                _ => "23",
            };
            ("h264_qsv", vec!["-global_quality".to_string(), q_val.to_string()])
        } else {
            let crf_val = match plan.quality.as_deref() {
                Some("low") => "28",
                Some("high") => "18",
                _ => "23",
            };
            ("libx264", vec![
                "-preset".to_string(),
                "medium".to_string(),
                "-crf".to_string(),
                crf_val.to_string(),
            ])
        };

        let bgm_opt = plan.bgm_overlay.as_ref().filter(|b| !b.path.trim().is_empty());
        let (filter_graph_final, has_bgm_mixed) = if let Some(bgm) = bgm_opt {
            let bgm_vol = bgm.volume.unwrap_or(1.0);
            let orig_vol = bgm.orig_volume.unwrap_or(1.0);
            let bgm_idx = if plan.image_overlay.is_some() { 2 } else { 1 };
            let has_orig = input_has_audio(plan) && plan.include_audio.unwrap_or(true) && orig_vol > 0.001;
            let dur_str = format_secs(duration);

            let mut combined = filter_graph;
            if has_orig {
                let orig_tempo_part = match plan.video_filter.as_deref().map(|s| s.trim().to_ascii_lowercase()) {
                    Some(ref s) if s == "fast2x" => "atempo=2.0,",
                    Some(ref s) if s == "slow05x" => "atempo=0.5,",
                    Some(ref s) if s == "reverse" => "areverse,",
                    _ => "",
                };
                combined.push_str(&format!(
                    ";[0:a]{}volume={:.2}[a0];[{}:a]volume={:.2},atrim=0:{}[a1];[a0][a1]amix=inputs=2:duration=first:dropout_transition=0[aout]",
                    orig_tempo_part, orig_vol, bgm_idx, bgm_vol, dur_str
                ));
            } else {
                combined.push_str(&format!(
                    ";[{}:a]volume={:.2},atrim=0:{}[aout]",
                    bgm_idx, bgm_vol, dur_str
                ));
            }
            (combined, true)
        } else {
            (filter_graph, false)
        };

        args.extend([
            "-filter_complex".to_string(),
            filter_graph_final,
            "-map".to_string(),
            "[vout]".to_string(),
            "-c:v".to_string(),
            vcodec.to_string(),
        ]);
        args.extend(quality_args);
        args.extend([
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
            "-movflags".to_string(),
            "+faststart".to_string(),
        ]);

        if has_bgm_mixed {
            args.extend([
                "-map".to_string(),
                "[aout]".to_string(),
                "-c:a".to_string(),
                "aac".to_string(),
                "-b:a".to_string(),
                "128k".to_string(),
            ]);
        } else {
            let include_audio = plan.include_audio.unwrap_or(true);
            if include_audio {
                args.extend([
                    "-map".to_string(),
                    "0:a?".to_string(),
                    "-c:a".to_string(),
                    "aac".to_string(),
                    "-b:a".to_string(),
                    "128k".to_string(),
                ]);

                if let Some(vk) = &plan.video_filter {
                    match vk.trim().to_ascii_lowercase().as_str() {
                        "fast2x" => {
                            args.extend(["-af".to_string(), "atempo=2.0".to_string()]);
                        }
                        "slow05x" => {
                            args.extend(["-af".to_string(), "atempo=0.5".to_string()]);
                        }
                        "reverse" => {
                            args.extend(["-af".to_string(), "areverse".to_string()]);
                        }
                        _ => {}
                    }
                }
            } else {
                args.push("-an".to_string());
            }
        }

        args.push(plan.output_path.clone());
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

fn build_ffmpeg_args(plan: &RenderPlan) -> Vec<String> {
    let ffmpeg_bin = resolve_binary("ffmpeg");
    let hw_info = detect_hardware_acceleration(&ffmpeg_bin);
    build_ffmpeg_args_with_hw(plan, &hw_info)
}

fn ffmpeg_supports_filter(filter_name: &str) -> Option<bool> {
    let ffmpeg_bin = resolve_binary("ffmpeg");
    let output = new_hidden_command(&ffmpeg_bin)
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
    let format_label = match output_path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("mp4") => "MP4 短视频",
        Some("webp") => "WebP 动图",
        Some("avif") => "AVIF 动图",
        _ => "GIF 动图",
    };
    let rendering_msg = format!("正在编码 {}…", format_label);

    let progress_task = stdout.map(|stdout| {
        let app = app.clone();
        let task_id = task_id.clone();
        let msg = rendering_msg.clone();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Some(seconds) = progress_seconds(&line) {
                    let progress = 6.0 + ((seconds / duration_secs).clamp(0.0, 1.0) as f32 * 88.0);
                    emit_progress(&app, &task_id, "rendering", progress, &msg);
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
    mut plan: RenderPlan,
) -> CommandResult<String> {
    let media = probe_media(Path::new(&plan.input_path))?;
    if plan.source_width.is_none() {
        plan.source_width = Some(media.width);
    }
    if plan.source_height.is_none() {
        plan.source_height = Some(media.height);
    }
    plan.output_path = resolve_unique_output_path(Path::new(&plan.output_path))
        .to_string_lossy()
        .to_string();
    validate_render_plan(&plan, &media)?;
    if !plan.active_text_layers().is_empty() && ffmpeg_supports_filter("drawtext") == Some(false) {
        return Err(app_error(
            "DRAW_TEXT_UNAVAILABLE",
            "当前 FFmpeg 构建不包含 drawtext，请换用带 libfreetype 的 FFmpeg。",
        ));
    }
    let args = build_ffmpeg_args(&plan);
    let ffmpeg_bin = resolve_binary("ffmpeg");
    let mut child = new_hidden_command(&ffmpeg_bin)
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
        let status = new_hidden_command("powershell")
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
    } else if lower.ends_with(".avif") {
        "image/avif"
    } else if lower.ends_with(".mp4") {
        "video/mp4"
    } else if lower.ends_with(".mp3") {
        "audio/mpeg"
    } else if lower.ends_with(".wav") {
        "audio/wav"
    } else if lower.ends_with(".ogg") {
        "audio/ogg"
    } else if lower.ends_with(".m4a") || lower.ends_with(".aac") {
        "audio/mp4"
    } else {
        "image/gif"
    };
    Ok(format!("data:{};base64,{}", mime, encoded))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinSticker {
    pub id: String,
    pub name: String,
    pub category: String,
    pub path: String,
    pub data_url: String,
    pub default_width: u32,
}

struct StickerDef {
    id: &'static str,
    name: &'static str,
    category: &'static str,
    filename: &'static str,
    default_width: u32,
}

const BUILTIN_STICKERS: &[StickerDef] = &[
    StickerDef { id: "deal_with_it", name: "暴徒墨镜", category: "classic", filename: "deal_with_it.png", default_width: 140 },
    StickerDef { id: "red_question", name: "经典问号", category: "classic", filename: "red_question.png", default_width: 90 },
    StickerDef { id: "confused_questions", name: "连环问号", category: "classic", filename: "confused_questions.png", default_width: 120 },
    StickerDef { id: "doge_face", name: "旺柴狗头", category: "classic", filename: "doge_face.png", default_width: 120 },
    StickerDef { id: "panda_face", name: "搞笑熊猫头", category: "classic", filename: "panda_face.png", default_width: 130 },
    StickerDef { id: "clown_nose", name: "小丑红鼻", category: "classic", filename: "clown_nose.png", default_width: 90 },
    StickerDef { id: "sweat_smile", name: "流汗黄豆", category: "emotion", filename: "sweat_smile.png", default_width: 120 },
    StickerDef { id: "approved_stamp", name: "绝赞印章", category: "emotion", filename: "approved_stamp.png", default_width: 130 },
    StickerDef { id: "rejected_stamp", name: "驳回印章", category: "emotion", filename: "rejected_stamp.png", default_width: 130 },
    StickerDef { id: "anger_vein", name: "暴青筋", category: "emotion", filename: "anger_vein.png", default_width: 80 },
    StickerDef { id: "sweat_drop", name: "汗颜汗滴", category: "emotion", filename: "sweat_drop.png", default_width: 70 },
    StickerDef { id: "sparkle_stars", name: "崇拜闪光", category: "emotion", filename: "sparkle_stars.png", default_width: 100 },
    StickerDef { id: "heart_eyes", name: "爱心桃心眼", category: "emotion", filename: "heart_eyes.png", default_width: 120 },
    StickerDef { id: "fire_flame", name: "高能烈火", category: "emotion", filename: "fire_flame.png", default_width: 110 },
    StickerDef { id: "crying_tears", name: "面条宽泪", category: "emotion", filename: "crying_tears.png", default_width: 120 },
    StickerDef { id: "thumbs_up", name: "超赞大拇指", category: "emotion", filename: "thumbs_up.png", default_width: 100 },
    StickerDef { id: "shout_bubble", name: "尖角咆哮气泡", category: "bubble", filename: "shout_bubble.png", default_width: 160 },
    StickerDef { id: "dots_bubble", name: "无语省略号气泡", category: "bubble", filename: "dots_bubble.png", default_width: 150 },
];

fn resolve_stickers_dir() -> Option<PathBuf> {
    if let Ok(val) = env::var("MEME_STUDIO_STICKERS_DIR") {
        let p = PathBuf::from(val);
        if p.is_dir() {
            return Some(fs::canonicalize(&p).unwrap_or(p));
        }
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let prod_candidates = [
                exe_dir.join("stickers"),
                exe_dir.join("../Resources/stickers"),
                exe_dir.join("../Resources"),
            ];
            for cand in prod_candidates {
                if cand.is_dir() && cand.join("deal_with_it.png").exists() {
                    return Some(fs::canonicalize(&cand).unwrap_or(cand));
                }
            }
        }
    }

    let dev_candidates = [
        PathBuf::from("src-tauri").join("stickers"),
        PathBuf::from("stickers"),
        PathBuf::from("..").join("src-tauri").join("stickers"),
        PathBuf::from("..").join("stickers"),
    ];
    for cand in dev_candidates {
        if cand.is_dir() && cand.join("deal_with_it.png").exists() {
            return Some(fs::canonicalize(&cand).unwrap_or(cand));
        }
    }
    None
}

#[tauri::command]
fn get_builtin_stickers() -> CommandResult<Vec<BuiltinSticker>> {
    let dir = resolve_stickers_dir().ok_or_else(|| {
        app_error("STICKERS_DIR_NOT_FOUND", "找不到内置贴纸素材目录。")
    })?;

    let mut result = Vec::with_capacity(BUILTIN_STICKERS.len());
    for def in BUILTIN_STICKERS {
        let file_path = dir.join(def.filename);
        if !file_path.exists() {
            continue;
        }
        let path_str = path_to_string(&file_path)?;
        let data_url = read_file_as_data_url(path_str.clone())?;
        result.push(BuiltinSticker {
            id: def.id.to_string(),
            name: def.name.to_string(),
            category: def.category.to_string(),
            path: path_str,
            data_url,
            default_width: def.default_width,
        });
    }
    Ok(result)
}

struct SfxDef {
    id: &'static str,
    name: &'static str,
    category: &'static str,
    filename: &'static str,
    duration: f64,
}

const BUILTIN_SFX: &[SfxDef] = &[
    SfxDef { id: "censor_bleep", name: "消音哔哔", category: "reaction", filename: "censor_bleep.mp3", duration: 0.8 },
    SfxDef { id: "crisp_ding", name: "绝了清脆叮", category: "reaction", filename: "crisp_ding.mp3", duration: 0.8 },
    SfxDef { id: "boom_808", name: "震撼低音808", category: "drama", filename: "boom_808.mp3", duration: 1.2 },
    SfxDef { id: "alarm_danger", name: "高能警报", category: "drama", filename: "alarm_danger.mp3", duration: 1.5 },
    SfxDef { id: "boing_spring", name: "弹簧搞笑", category: "funny", filename: "boing_spring.mp3", duration: 0.8 },
    SfxDef { id: "victory_cheer", name: "欢呼和弦", category: "funny", filename: "victory_cheer.mp3", duration: 1.6 },
    SfxDef { id: "face_slap", name: "打脸啪啪", category: "reaction", filename: "face_slap.mp3", duration: 0.35 },
    SfxDef { id: "goofy_laugh", name: "滑稽笑声", category: "funny", filename: "goofy_laugh.mp3", duration: 1.5 },
];

fn resolve_sfx_dir() -> Option<PathBuf> {
    if let Ok(val) = env::var("MEME_STUDIO_SFX_DIR") {
        let p = PathBuf::from(val);
        if p.is_dir() {
            return Some(fs::canonicalize(&p).unwrap_or(p));
        }
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let prod_candidates = [
                exe_dir.join("sfx"),
                exe_dir.join("../Resources/sfx"),
                exe_dir.join("../Resources"),
            ];
            for cand in prod_candidates {
                if cand.is_dir() && cand.join("censor_bleep.mp3").exists() {
                    return Some(fs::canonicalize(&cand).unwrap_or(cand));
                }
            }
        }
    }

    let dev_candidates = [
        PathBuf::from("src-tauri").join("sfx"),
        PathBuf::from("sfx"),
        PathBuf::from("..").join("src-tauri").join("sfx"),
        PathBuf::from("..").join("sfx"),
    ];
    for cand in dev_candidates {
        if cand.is_dir() && cand.join("censor_bleep.mp3").exists() {
            return Some(fs::canonicalize(&cand).unwrap_or(cand));
        }
    }
    None
}

#[tauri::command]
fn get_builtin_sfx() -> CommandResult<Vec<BuiltinSfx>> {
    let dir = resolve_sfx_dir().ok_or_else(|| {
        app_error("SFX_DIR_NOT_FOUND", "找不到内置音效素材目录。")
    })?;

    let mut result = Vec::with_capacity(BUILTIN_SFX.len());
    for def in BUILTIN_SFX {
        let file_path = dir.join(def.filename);
        if !file_path.exists() {
            continue;
        }
        let path_str = path_to_string(&file_path)?;
        let data_url = read_file_as_data_url(path_str.clone())?;
        result.push(BuiltinSfx {
            id: def.id.to_string(),
            name: def.name.to_string(),
            category: def.category.to_string(),
            path: path_str,
            data_url,
            duration: def.duration,
        });
    }
    Ok(result)
}

#[tauri::command]
fn extract_video_frame(
    path: String,
    time_secs: f64,
    max_width: Option<u32>,
) -> CommandResult<String> {
    let p = Path::new(&path);
    validate_input_file(p)?;
    if !time_secs.is_finite() || time_secs < 0.0 {
        return Err(app_error("INVALID_TIME", "截取时间点必须大于等于 0 秒。"));
    }

    let ffmpeg_bin = resolve_binary("ffmpeg");
    let limit_w = max_width.unwrap_or(640).clamp(160, 1920);
    let scale_filter = format!("scale='min({},iw)':-2", limit_w);
    let time_str = format_secs(time_secs);

    let output = new_hidden_command(&ffmpeg_bin)
        .args([
            "-ss",
            &time_str,
            "-i",
            &path,
            "-frames:v",
            "1",
            "-vf",
            &scale_filter,
            "-f",
            "image2pipe",
            "-vcodec",
            "mjpeg",
            "-q:v",
            "3",
            "-",
        ])
        .output()
        .map_err(|e| app_error("FFMPEG_SPAWN_ERROR", format!("启动 FFmpeg 抽帧失败: {}", e)))?;

    if !output.status.success() || output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(app_error(
            "FRAME_EXTRACT_FAILED",
            format!("抽帧失败: {}", stderr.lines().last().unwrap_or("未能获取画面帧")),
        ));
    }

    let encoded = BASE64.encode(&output.stdout);
    Ok(format!("data:image/jpeg;base64,{}", encoded))
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
            read_file_as_data_url,
            get_builtin_stickers,
            get_builtin_sfx,
            get_hardware_acceleration_info,
            extract_video_frame
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
        assert_eq!(escape_drawtext("it's"), "it'\\''s");
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
            text_layers: None,
            image_overlay: None,
            format: None,
            ..Default::default()
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
                id: None,
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
                visible: Some(true),
                ..Default::default()
            }),
            text_layers: None,
            image_overlay: None,
            format: Some("gif".to_string()),
            ..Default::default()
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
                id: None,
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
                visible: Some(true),
                ..Default::default()
            }),
            text_layers: None,
            image_overlay: None,
            format: None,
            ..Default::default()
        };
        let graph = build_filter_graph(&plan);
        assert!(graph.contains("box=0"));
        assert!(!graph.contains("borderw="));
    }

    #[test]
    fn builds_multi_text_layers_filter_graph() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            crop: None,
            output_width: 320,
            fps: 10,
            quality: None,
            text_overlay: None,
            text_layers: Some(vec![
                TextOverlay {
                    id: Some("layer-top".to_string()),
                    text: "顶部吐槽".to_string(),
                    x: 10,
                    y: 10,
                    size: 24,
                    color: "#ffffff".to_string(),
                    stroke_width: Some(2),
                    stroke_color: Some("#000000".to_string()),
                    use_box: Some(true),
                    box_color: Some("#000000".to_string()),
                    box_opacity: Some(0.5),
                    visible: Some(true),
                    ..Default::default()
                },
                TextOverlay {
                    id: Some("layer-hidden".to_string()),
                    text: "隐藏不渲染".to_string(),
                    x: 50,
                    y: 50,
                    size: 20,
                    color: "#ffff00".to_string(),
                    visible: Some(false),
                    ..Default::default()
                },
                TextOverlay {
                    id: Some("layer-bottom".to_string()),
                    text: "底部反转".to_string(),
                    x: 10,
                    y: 150,
                    size: 28,
                    color: "#ff0000".to_string(),
                    stroke_width: None,
                    stroke_color: None,
                    use_box: None,
                    box_color: None,
                    box_opacity: None,
                    visible: Some(true),
                    ..Default::default()
                },
            ]),
            image_overlay: None,
            format: Some("gif".to_string()),
            ..Default::default()
        };
        let graph = build_filter_graph(&plan);
        assert!(graph.contains("顶部吐槽"));
        assert!(graph.contains("底部反转"));
        assert!(!graph.contains("隐藏不渲染"));
        assert_eq!(graph.matches("drawtext=").count(), 2);
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
            text_layers: None,
            image_overlay: None,
            format: Some("webp".to_string()),
            ..Default::default()
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
    fn builds_avif_filter_graph_and_args() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.avif".to_string(),
            start_secs: 1.0,
            end_secs: 3.5,
            crop: None,
            output_width: 320,
            fps: 12,
            quality: Some("low".to_string()),
            text_overlay: None,
            text_layers: None,
            image_overlay: None,
            format: Some("avif".to_string()),
            video_filter: None,
            include_audio: None,
            hw_accel: None,
            ..Default::default()
        };
        let graph = build_filter_graph(&plan);
        assert!(!graph.contains("palettegen"));
        assert!(!graph.contains("paletteuse"));
        assert!(graph.contains("format=yuv420p[vout]"));

        let args = build_ffmpeg_args(&plan);
        assert!(args.iter().any(|arg| arg == "libsvtav1"));
        assert!(args.iter().any(|arg| arg == "-crf"));
        assert!(args.iter().any(|arg| arg == "38"));
        assert!(args.iter().any(|arg| arg == "-preset"));
        assert!(args.iter().any(|arg| arg == "8"));
        assert!(args.iter().any(|arg| arg == "-pix_fmt"));
        assert!(args.iter().any(|arg| arg == "yuv420p"));
    }

    #[test]
    fn builds_filter_graph_with_video_filters() {
        let base_plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            crop: None,
            output_width: 300,
            fps: 10,
            quality: None,
            text_overlay: None,
            text_layers: None,
            image_overlay: None,
            format: Some("gif".to_string()),
            video_filter: None,
            include_audio: None,
            hw_accel: None,
            ..Default::default()
        };

        // 1. Grayscale
        let mut p = base_plan.clone();
        p.video_filter = Some("grayscale".to_string());
        let g = build_filter_graph(&p);
        assert!(g.contains("hue=s=0"));

        // 2. Invert / Negate
        p.video_filter = Some("invert".to_string());
        let g = build_filter_graph(&p);
        assert!(g.contains("negate"));

        // 3. Fast 2x
        p.video_filter = Some("fast2x".to_string());
        let g = build_filter_graph(&p);
        assert!(g.contains("setpts=0.5*PTS"));

        // 4. Reverse
        p.video_filter = Some("reverse".to_string());
        let g = build_filter_graph(&p);
        assert!(g.contains("reverse"));

        // 5. None or invalid
        p.video_filter = Some("none".to_string());
        let g = build_filter_graph(&p);
        assert!(!g.contains("hue=s=0"));
        assert!(!g.contains("negate"));
    }

    #[test]
    fn validates_render_plan_supports_gif_webp_and_avif() {
        let media = MediaInfo {
            path: "in.mp4".to_string(),
            file_name: "in.mp4".to_string(),
            format: "mov,mp4,m4a,3gp,3g2,mj2".to_string(),
            duration_secs: 5.0,
            width: 1280,
            height: 720,
            fps: 30.0,
            has_audio: false,
        };

        let temp_dir = env::temp_dir();
        let webp_out = temp_dir.join("test_valid.webp");
        let plan_webp = RenderPlan {
            input_path: "in.mp4".to_string(),
            output_path: webp_out.to_string_lossy().to_string(),
            start_secs: 0.0,
            end_secs: 2.0,
            crop: None,
            output_width: 320,
            fps: 15,
            quality: None,
            text_overlay: None,
            text_layers: None,
            image_overlay: None,
            format: Some("webp".to_string()),
            ..Default::default()
        };
        assert!(validate_render_plan(&plan_webp, &media).is_ok());

        let avif_out = temp_dir.join("test_valid.avif");
        let plan_avif = RenderPlan {
            output_path: avif_out.to_string_lossy().to_string(),
            format: Some("avif".to_string()),
            ..plan_webp.clone()
        };
        assert!(validate_render_plan(&plan_avif, &media).is_ok());

        let mp4_out = temp_dir.join("test_valid.mp4");
        let plan_mp4 = RenderPlan {
            output_path: mp4_out.to_string_lossy().to_string(),
            format: Some("mp4".to_string()),
            include_audio: Some(true),
            ..plan_webp.clone()
        };
        assert!(validate_render_plan(&plan_mp4, &media).is_ok());

        // 测试格式不匹配
        let mismatch_plan = RenderPlan {
            output_path: avif_out.to_string_lossy().to_string(),
            format: Some("gif".to_string()),
            ..plan_webp.clone()
        };
        assert!(validate_render_plan(&mismatch_plan, &media).is_err());
    }

    #[test]
    fn builds_mp4_filter_graph_and_args() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.mp4".to_string(),
            start_secs: 0.5,
            end_secs: 2.5,
            crop: None,
            output_width: 320,
            fps: 24,
            quality: Some("high".to_string()),
            text_overlay: None,
            text_layers: None,
            image_overlay: None,
            format: Some("mp4".to_string()),
            video_filter: None,
            include_audio: Some(true),
            hw_accel: None,
            ..Default::default()
        };
        let graph = build_filter_graph(&plan);
        assert!(!graph.contains("palettegen"));
        assert!(!graph.contains("paletteuse"));
        assert!(graph.contains("format=yuv420p[vout]"));
        assert!(graph.contains("scale=320:-2:flags=lanczos"));

        let args = build_ffmpeg_args(&plan);
        assert!(args.iter().any(|arg| arg == "libx264" || arg == "h264_videotoolbox"));
        assert!(args.iter().any(|arg| arg == "+faststart"));
        assert!(args.iter().any(|arg| arg == "-crf" || arg == "-q:v"));
        assert!(args.iter().any(|arg| arg == "0:a?"));
        assert!(args.iter().any(|arg| arg == "aac"));
        assert!(args.iter().any(|arg| arg == "128k"));
        assert!(!args.iter().any(|arg| arg == "-an"));
    }

    #[test]
    fn builds_mp4_args_muted_when_include_audio_false() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.mp4".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            crop: None,
            output_width: 320,
            fps: 15,
            quality: None,
            text_overlay: None,
            text_layers: None,
            image_overlay: None,
            format: Some("mp4".to_string()),
            video_filter: None,
            include_audio: Some(false),
            hw_accel: None,
            ..Default::default()
        };
        let args = build_ffmpeg_args(&plan);
        assert!(args.iter().any(|arg| arg == "-an"));
        assert!(!args.iter().any(|arg| arg == "0:a?"));
        assert!(!args.iter().any(|arg| arg == "aac"));
    }

    #[test]
    fn builds_mp4_args_with_audio_filters_when_speed_active() {
        let mut plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.mp4".to_string(),
            start_secs: 0.0,
            end_secs: 2.0,
            crop: None,
            output_width: 320,
            fps: 20,
            quality: None,
            text_overlay: None,
            text_layers: None,
            image_overlay: None,
            format: Some("mp4".to_string()),
            video_filter: Some("fast2x".to_string()),
            include_audio: Some(true),
            hw_accel: None,
            ..Default::default()
        };

        // 1. fast2x -> atempo=2.0
        let args1 = build_ffmpeg_args(&plan);
        let af_pos = args1.iter().position(|a| a == "-af").expect("-af should exist");
        assert_eq!(args1[af_pos + 1], "atempo=2.0");

        // 2. slow05x -> atempo=0.5
        plan.video_filter = Some("slow05x".to_string());
        let args2 = build_ffmpeg_args(&plan);
        let af_pos2 = args2.iter().position(|a| a == "-af").expect("-af should exist");
        assert_eq!(args2[af_pos2 + 1], "atempo=0.5");

        // 3. reverse -> areverse
        plan.video_filter = Some("reverse".to_string());
        let args3 = build_ffmpeg_args(&plan);
        let af_pos3 = args3.iter().position(|a| a == "-af").expect("-af should exist");
        assert_eq!(args3[af_pos3 + 1], "areverse");
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
    fn read_file_as_data_url_encodes_mp4_file() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("test_sample.mp4");
        fs::write(&test_file, b"\x00\x00\x00 ftypisom").unwrap();
        let res = read_file_as_data_url(test_file.to_string_lossy().to_string());
        assert!(res.is_ok());
        let url = res.unwrap();
        assert!(url.starts_with("data:video/mp4;base64,"));
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn copy_file_to_clipboard_fails_on_missing_file() {
        let res = copy_file_to_clipboard("/path/to/nonexistent/file.gif".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "FILE_NOT_FOUND");
    }

    #[test]
    fn resolves_stickers_dir_and_loads_builtin_stickers() {
        let stickers = get_builtin_stickers();
        assert!(stickers.is_ok(), "get_builtin_stickers should succeed");
        let list = stickers.unwrap();
        assert_eq!(list.len(), 18, "Should load all 18 builtin stickers");
        assert!(list.iter().any(|s| s.id == "deal_with_it"));
        assert!(list.iter().any(|s| s.id == "doge_face"));
        assert!(list.iter().any(|s| s.id == "approved_stamp"));
        for s in &list {
            assert!(s.data_url.starts_with("data:image/png;base64,"));
            assert!(Path::new(&s.path).exists());
            assert!(s.default_width >= 50 && s.default_width <= 200);
        }
    }

    #[test]
    fn detects_hardware_acceleration_info() {
        let ffmpeg_bin = resolve_binary("ffmpeg");
        let info = detect_hardware_acceleration(&ffmpeg_bin);
        if cfg!(target_os = "macos") && ffmpeg_bin.exists() {
            assert!(info.supported);
            assert_eq!(info.engine, "VideoToolbox");
            assert_eq!(info.encoder, "h264_videotoolbox");
        }
    }

    #[test]
    fn builds_mp4_args_with_hardware_acceleration_when_available() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.mp4".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            output_width: 320,
            fps: 20,
            quality: Some("high".to_string()),
            format: Some("mp4".to_string()),
            hw_accel: Some("auto".to_string()),
            include_audio: Some(true),
            ..Default::default()
        };

        let mock_hw = HardwareAccelerationInfo {
            supported: true,
            engine: "VideoToolbox".to_string(),
            encoder: "h264_videotoolbox".to_string(),
            description: "Mock VideoToolbox".to_string(),
        };

        let args = build_ffmpeg_args_with_hw(&plan, &mock_hw);
        assert!(args.iter().any(|a| a == "-hwaccel"));
        assert!(args.iter().any(|a| a == "h264_videotoolbox"));
        assert!(args.iter().any(|a| a == "-q:v"));
        assert!(args.iter().any(|a| a == "75"));
        assert!(!args.iter().any(|a| a == "libx264"));
    }

    #[test]
    fn builds_mp4_args_with_software_when_hw_disabled() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.mp4".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            output_width: 320,
            fps: 20,
            quality: Some("medium".to_string()),
            format: Some("mp4".to_string()),
            hw_accel: Some("disabled".to_string()),
            include_audio: Some(true),
            ..Default::default()
        };

        let mock_hw = HardwareAccelerationInfo {
            supported: true,
            engine: "VideoToolbox".to_string(),
            encoder: "h264_videotoolbox".to_string(),
            description: "Mock VideoToolbox".to_string(),
        };

        let args = build_ffmpeg_args_with_hw(&plan, &mock_hw);
        assert!(!args.iter().any(|a| a == "-hwaccel"));
        assert!(args.iter().any(|a| a == "libx264"));
        assert!(args.iter().any(|a| a == "-crf"));
        assert!(args.iter().any(|a| a == "23"));
        assert!(!args.iter().any(|a| a == "h264_videotoolbox"));
    }

    #[test]
    fn read_file_as_data_url_encodes_mp3_file() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("test_sample.mp3");
        fs::write(&test_file, b"ID3\x04\x00\x00\x00\x00\x00\x00").unwrap();
        let res = read_file_as_data_url(test_file.to_string_lossy().to_string());
        assert!(res.is_ok());
        let url = res.unwrap();
        assert!(url.starts_with("data:audio/mpeg;base64,"));
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn resolves_sfx_dir_and_loads_builtin_sfx() {
        let sfx_res = get_builtin_sfx();
        assert!(sfx_res.is_ok(), "get_builtin_sfx should succeed");
        let list = sfx_res.unwrap();
        assert_eq!(list.len(), 8, "Should load all 8 builtin sfx files");
        assert!(list.iter().any(|s| s.id == "censor_bleep"));
        assert!(list.iter().any(|s| s.id == "crisp_ding"));
        assert!(list.iter().any(|s| s.id == "boom_808"));
        assert!(list.iter().any(|s| s.id == "alarm_danger"));
        assert!(list.iter().any(|s| s.id == "boing_spring"));
        assert!(list.iter().any(|s| s.id == "victory_cheer"));
        assert!(list.iter().any(|s| s.id == "face_slap"));
        assert!(list.iter().any(|s| s.id == "goofy_laugh"));
        for s in &list {
            assert!(s.data_url.starts_with("data:audio/mpeg;base64,"));
            assert!(Path::new(&s.path).exists());
            assert!(s.duration > 0.0);
        }
    }

    #[test]
    fn builds_mp4_args_with_bgm_mixing() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.mp4".to_string(),
            start_secs: 0.0,
            end_secs: 2.0,
            output_width: 320,
            fps: 20,
            format: Some("mp4".to_string()),
            has_audio: Some(true),
            include_audio: Some(true),
            bgm_overlay: Some(BgmOverlay {
                path: "test_bgm.mp3".to_string(),
                volume: Some(0.8),
                orig_volume: Some(0.5),
            }),
            ..Default::default()
        };

        let args = build_ffmpeg_args(&plan);
        // BGM input should be passed
        let bgm_pos = args.iter().position(|a| a == "test_bgm.mp3").expect("BGM input exists");
        assert_eq!(args[bgm_pos - 1], "-i");

        // Filter graph should contain amix
        let fc_pos = args.iter().position(|a| a == "-filter_complex").expect("-filter_complex exists");
        let fc = &args[fc_pos + 1];
        assert!(fc.contains("amix=inputs=2:duration=first:dropout_transition=0"));
        assert!(fc.contains("volume=0.50[a0]"));
        assert!(fc.contains("[1:a]volume=0.80,atrim=0:2.000[a1]"));
        assert!(fc.contains("[aout]"));

        // Audio mapping should map [aout]
        let aout_pos = args.iter().position(|a| a == "[aout]").expect("[aout] mapped");
        assert_eq!(args[aout_pos - 1], "-map");
        assert!(args.iter().any(|a| a == "aac"));
    }

    #[test]
    fn builds_mp4_args_with_bgm_when_orig_muted_or_no_audio() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.mp4".to_string(),
            start_secs: 0.0,
            end_secs: 1.5,
            output_width: 320,
            fps: 20,
            format: Some("mp4".to_string()),
            has_audio: Some(false),
            include_audio: Some(true),
            bgm_overlay: Some(BgmOverlay {
                path: "test_bgm.mp3".to_string(),
                volume: Some(0.9),
                orig_volume: Some(1.0),
            }),
            ..Default::default()
        };

        let args = build_ffmpeg_args(&plan);
        let fc_pos = args.iter().position(|a| a == "-filter_complex").expect("-filter_complex exists");
        let fc = &args[fc_pos + 1];
        // Must NOT reference [0:a] or amix when original has no audio!
        assert!(!fc.contains("[0:a]"));
        assert!(!fc.contains("amix"));
        assert!(fc.contains("[1:a]volume=0.90,atrim=0:1.500[aout]"));
        assert!(args.iter().any(|a| a == "[aout]"));
    }

    #[test]
    fn builds_mp4_args_with_bgm_and_sticker_overlay() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.mp4".to_string(),
            start_secs: 0.0,
            end_secs: 2.0,
            output_width: 320,
            fps: 20,
            format: Some("mp4".to_string()),
            has_audio: Some(true),
            include_audio: Some(true),
            image_overlay: Some(ImageOverlay {
                path: "sticker.png".to_string(),
                x: 10,
                y: 10,
                width: 100,
            }),
            bgm_overlay: Some(BgmOverlay {
                path: "test_bgm.mp3".to_string(),
                volume: Some(1.0),
                orig_volume: Some(1.0),
            }),
            ..Default::default()
        };

        let args = build_ffmpeg_args(&plan);
        // Input 0: input.mp4, Input 1: sticker.png, Input 2: test_bgm.mp3
        let fc_pos = args.iter().position(|a| a == "-filter_complex").expect("-filter_complex exists");
        let fc = &args[fc_pos + 1];
        // BGM input index should be 2!
        assert!(fc.contains("[2:a]volume=1.00,atrim=0:2.000[a1]"));
        assert!(fc.contains("[0:a]volume=1.00[a0]"));
        assert!(fc.contains("amix=inputs=2:duration=first:dropout_transition=0"));
    }

    #[test]
    fn builds_drawtext_filter_graph_with_shadow() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            output_width: 320,
            fps: 15,
            text_overlay: Some(TextOverlay {
                text: "阴影测试".to_string(),
                x: 20,
                y: 30,
                size: 24,
                color: "#FFFFFF".to_string(),
                shadow_offset: Some(4),
                shadow_color: Some("#112233".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let graph = build_filter_graph(&plan);
        assert!(graph.contains("shadowx=4:shadowy=4:shadowcolor=0x112233"));
    }

    #[test]
    fn builds_drawtext_filter_graph_with_rotation() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            output_width: 320,
            fps: 15,
            text_overlay: Some(TextOverlay {
                text: "旋转45度".to_string(),
                x: 40,
                y: 50,
                size: 28,
                color: "#FFFF00".to_string(),
                rotation: Some(45),
                ..Default::default()
            }),
            ..Default::default()
        };

        let graph = build_filter_graph(&plan);
        assert!(graph.contains("color=c=black@0.0:s="));
        assert!(graph.contains("rotate=45*PI/180:c=none:ow=rotw(45*PI/180):oh=roth(45*PI/180)"));
        assert!(graph.contains("overlay="));
    }

    #[test]
    fn builds_drawtext_filter_graph_with_italic_shear() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.mp4".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            output_width: 320,
            fps: 15,
            text_overlay: Some(TextOverlay {
                text: "斜体动感".to_string(),
                x: 10,
                y: 20,
                size: 30,
                color: "#FFFFFF".to_string(),
                is_italic: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };

        let graph = build_filter_graph(&plan);
        assert!(graph.contains("shear=shx=-0.22:c=none"));
        assert!(graph.contains("format=yuv420p[vout]"));
    }

    #[test]
    fn builds_drawtext_filter_graph_with_arc_curvature() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            output_width: 320,
            fps: 15,
            text_overlay: Some(TextOverlay {
                text: "高能预警".to_string(),
                x: 60,
                y: 40,
                size: 26,
                color: "#FF0000".to_string(),
                arc_radius: Some(60),
                ..Default::default()
            }),
            ..Default::default()
        };

        let graph = build_filter_graph(&plan);
        assert!(graph.contains("text='高'"));
        assert!(graph.contains("text='能'"));
        assert!(graph.contains("text='预'"));
        assert!(graph.contains("text='警'"));
        assert!(graph.contains("tc_1_"));
    }

    #[test]
    fn builds_drawtext_filter_graph_combined_advanced_effects() {
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.webp".to_string(),
            start_secs: 0.0,
            end_secs: 1.0,
            output_width: 320,
            fps: 15,
            text_overlay: Some(TextOverlay {
                text: "全能特效".to_string(),
                x: 30,
                y: 30,
                size: 32,
                color: "#00FFCC".to_string(),
                rotation: Some(-25),
                is_italic: Some(true),
                stroke_width: Some(3),
                stroke_color: Some("#000000".to_string()),
                shadow_offset: Some(6),
                shadow_color: Some("#222222".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let graph = build_filter_graph(&plan);
        assert!(graph.contains("shear=shx=-0.22:c=none"));
        assert!(graph.contains("rotate=-25*PI/180"));
        assert!(graph.contains("borderw=3"));
        assert!(graph.contains("shadowx=6:shadowy=6:shadowcolor=0x222222"));
        assert!(graph.ends_with("[vout]"));
    }

    #[test]
    fn extract_video_frame_fails_on_missing_file() {
        let res = extract_video_frame("/path/not/exist.mp4".to_string(), 0.0, Some(320));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "INPUT_NOT_FOUND");
    }

    #[test]
    fn extract_video_frame_fails_on_negative_time() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("test_dummy.mp4");
        fs::write(&test_file, b"test").unwrap();
        let res = extract_video_frame(test_file.to_string_lossy().to_string(), -1.0, Some(320));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "INVALID_TIME");
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn parses_numbered_stem_correctly() {
        assert_eq!(parse_numbered_stem("output"), None);
        assert_eq!(parse_numbered_stem("output()"), None);
        assert_eq!(parse_numbered_stem("output (0)"), None);
        assert_eq!(parse_numbered_stem("output (abc)"), None);
        assert_eq!(parse_numbered_stem("output (1)"), Some(("output".to_string(), 2)));
        assert_eq!(parse_numbered_stem("my meme (5)"), Some(("my meme".to_string(), 6)));
        assert_eq!(parse_numbered_stem("nested (meme) (2)"), Some(("nested (meme)".to_string(), 3)));
    }

    #[test]
    fn resolves_unique_output_path_without_overwrite() {
        let temp_dir = env::temp_dir().join(format!("meme_test_unique_{}", task_id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let base_file = temp_dir.join("meme.gif");
        // 1. File doesn't exist yet -> returns base_file directly
        let res1 = resolve_unique_output_path(&base_file);
        assert_eq!(res1, base_file);

        // 2. Create base_file -> resolves to "meme (1).gif"
        fs::write(&base_file, b"data 0").unwrap();
        let res2 = resolve_unique_output_path(&base_file);
        assert_eq!(res2, temp_dir.join("meme (1).gif"));

        // 3. Create "meme (1).gif" -> resolves to "meme (2).gif"
        fs::write(&res2, b"data 1").unwrap();
        let res3 = resolve_unique_output_path(&base_file);
        assert_eq!(res3, temp_dir.join("meme (2).gif"));

        // 4. Starting from already-numbered file "meme (1).gif" when it exists -> resolves to "meme (2).gif"
        let res4 = resolve_unique_output_path(&temp_dir.join("meme (1).gif"));
        assert_eq!(res4, temp_dir.join("meme (2).gif"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn scales_text_and_sticker_from_source_resolution_to_output_width() {
        // Source 1920x1080 -> Output 320x180 (scale_factor = 1/6)
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 2.0,
            crop: None,
            output_width: 320,
            fps: 12,
            source_width: Some(1920),
            source_height: Some(1080),
            text_overlay: Some(TextOverlay {
                text: "居中中文字幕".to_string(),
                x: 600,
                y: 900,
                size: 120,
                color: "#ffffff".to_string(),
                stroke_width: Some(6),
                stroke_color: Some("#000000".to_string()),
                visible: Some(true),
                ..Default::default()
            }),
            image_overlay: Some(ImageOverlay {
                path: "sticker.png".to_string(),
                x: 900,
                y: 480,
                width: 480,
            }),
            format: Some("gif".to_string()),
            ..Default::default()
        };

        let graph = build_filter_graph(&plan);
        // Text: x = 600/6 = 100, y = 900/6 = 150, fontsize = 120/6 = 20, borderw = 6/6 = 1
        assert!(graph.contains("fontsize=20:x=100:y=150:borderw=1"));
        // Sticker: width = 480/6 = 80, x = 900/6 = 150, y = 480/6 = 80
        assert!(graph.contains("[1:v]scale=80:-1:flags=lanczos[sticker]"));
        assert!(graph.contains("[base][sticker]overlay=150:80:eof_action=pass[composed]"));
    }

    #[test]
    fn scales_text_and_sticker_with_crop_offset_and_scale() {
        // Source 1920x1080, Crop at (320, 140) size 640x360 -> Output 320x180 (scale_factor = 320/640 = 0.5)
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 2.0,
            crop: Some(CropArea {
                x: 320,
                y: 140,
                width: 640,
                height: 360,
            }),
            output_width: 320,
            fps: 12,
            source_width: Some(1920),
            source_height: Some(1080),
            text_overlay: Some(TextOverlay {
                text: "裁切内文字".to_string(),
                x: 420, // rel_x = 100 -> scaled_x = 50
                y: 340, // rel_y = 200 -> scaled_y = 100
                size: 48, // scaled_size = 24
                color: "#ffe600".to_string(),
                visible: Some(true),
                ..Default::default()
            }),
            image_overlay: Some(ImageOverlay {
                path: "sticker.png".to_string(),
                x: 520, // rel_x = 200 -> scaled_x = 100
                y: 240, // rel_y = 100 -> scaled_y = 50
                width: 160, // scaled_width = 80
            }),
            format: Some("gif".to_string()),
            ..Default::default()
        };

        let graph = build_filter_graph(&plan);
        assert!(graph.contains("crop=640:360:320:140,scale=320:-1:flags=lanczos"));
        assert!(graph.contains("fontsize=24:x=50:y=100"));
        assert!(graph.contains("[1:v]scale=80:-1:flags=lanczos[sticker]"));
        assert!(graph.contains("[base][sticker]overlay=100:50:eof_action=pass[composed]"));
    }

    #[test]
    fn detect_system_font_prefers_cjk_font_on_macos() {
        if cfg!(target_os = "macos") {
            let font = detect_system_font();
            assert!(font.is_some());
            let f = font.unwrap();
            assert!(
                f.contains("STHeiti") || f.contains("Hiragino") || f.contains("PingFang") || f.contains("Songti"),
                "Expected CJK-capable font on macOS, got: {}",
                f
            );
        }
    }

    #[test]
    fn builds_image_adjustments_filter_chain_for_all_frames() {
        // Default / zero adjustments produce no extra FFmpeg filter
        let zero = ImageAdjustments::default();
        assert_eq!(build_image_adjustments_filter(&zero), None);

        // Brightness +25, Contrast +30, Saturation +40, Sharpen 50, Blur 20, Hue -45, Vignette true
        let adj = ImageAdjustments {
            brightness: Some(25),
            contrast: Some(30),
            saturation: Some(40),
            sharpen: Some(50),
            blur: Some(20),
            hue: Some(-45),
            vignette: Some(true),
        };
        let chain = build_image_adjustments_filter(&adj).expect("filter chain");
        assert!(chain.contains("eq=brightness=0.125:contrast=1.30:saturation=1.60"));
        assert!(chain.contains("hue=h=-45"));
        assert!(chain.contains("unsharp=5:5:1.25:5:5:0.0"));
        assert!(chain.contains("gblur=sigma=1.60"));
        assert!(chain.contains("vignette=PI/4"));

        // Verify integration into build_filter_graph before drawtext/sticker
        let plan = RenderPlan {
            input_path: "input.mp4".to_string(),
            output_path: "output.gif".to_string(),
            start_secs: 0.0,
            end_secs: 2.0,
            crop: None,
            output_width: 320,
            fps: 12,
            image_adjustments: Some(adj),
            format: Some("gif".to_string()),
            ..Default::default()
        };
        let graph = build_filter_graph(&plan);
        assert!(graph.contains("scale=320:-1:flags=lanczos,fps=12,eq=brightness=0.125:contrast=1.30:saturation=1.60,hue=h=-45,unsharp=5:5:1.25:5:5:0.0,gblur=sigma=1.60,vignette=PI/4[base]"));
    }

    #[test]
    fn windows_font_path_and_unc_prefix_compatibility() {
        // Verify Windows drive colon is escaped cleanly without backslash ambiguity when normalized to '/'
        let win_font = "C:/Windows/Fonts/msyh.ttc";
        assert_eq!(escape_drawtext(win_font), r"C\:/Windows/Fonts/msyh.ttc");

        // Verify extended-length Windows UNC prefix \\?\ is stripped by path_to_string
        let unc_path = Path::new(r"\\?\C:\Users\meme\stickers\deal_with_it.png");
        assert_eq!(
            path_to_string(unc_path).unwrap(),
            r"C:\Users\meme\stickers\deal_with_it.png"
        );
    }
}
