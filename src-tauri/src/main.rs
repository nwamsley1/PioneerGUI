#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Stdio};
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, PathResolver, Window};
use tempfile::tempdir;
use thiserror::Error;
use which::which;

static FALLBACK_BUILD: &str = include_str!("../fallback/default_build.json");
static FALLBACK_BUILD_SIMPLIFIED: &str = include_str!("../fallback/default_build_simplified.json");
static FALLBACK_SEARCH: &str = include_str!("../fallback/default_search.json");
static FALLBACK_SEARCH_SIMPLIFIED: &str =
    include_str!("../fallback/default_search_simplified.json");

#[derive(Debug, Error)]
enum ConfigLoadError {
    #[error(
        "Pioneer binary not found. Set `PIONEER_BINARY`/`PIONEER_PATH` or add the executable to PATH (tried `pioneer`, `Pioneer`, `pioneer.exe`, `Pioneer.exe`)."
    )]
    MissingBinary,
    #[error("Failed to execute Pioneer: {0}")]
    Execution(#[from] std::io::Error),
    #[error("Pioneer exited with status {0:?}")]
    NonZeroExit(Option<i32>),
    #[error("Failed to parse JSON output: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum RunMode {
    BuildSpecLib,
    SearchDia,
}

impl RunMode {
    fn subcommand(&self) -> &'static str {
        match self {
            RunMode::BuildSpecLib => "predict",
            RunMode::SearchDia => "search",
        }
    }

    fn config_filename(&self) -> &'static str {
        match self {
            RunMode::BuildSpecLib => "buildspeclib_params.json",
            RunMode::SearchDia => "search_params.json",
        }
    }

    fn stage_sequence(&self) -> &'static [StageInfo] {
        match self {
            RunMode::BuildSpecLib => &BUILD_STAGES,
            RunMode::SearchDia => &SEARCH_STAGES,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            RunMode::BuildSpecLib => "buildSpecLib",
            RunMode::SearchDia => "searchDia",
        }
    }
}

#[derive(Clone, Copy)]
struct StageInfo {
    key: &'static str,
    label: &'static str,
    keywords: &'static [&'static str],
}

const BUILD_STAGES: [StageInfo; 5] = [
    StageInfo {
        key: "starting",
        label: "Starting Pioneer",
        keywords: &[],
    },
    StageInfo {
        key: "prepare",
        label: "Preparing inputs",
        keywords: &["reading", "loading", "prepare", "initializing"],
    },
    StageInfo {
        key: "predict",
        label: "Predicting spectral library",
        keywords: &[
            "predict",
            "altimeter",
            "model",
            "generating",
            "writing predicted",
        ],
    },
    StageInfo {
        key: "write",
        label: "Writing spectral library",
        keywords: &["writing", "saving", "export"],
    },
    StageInfo {
        key: "complete",
        label: "Completed",
        keywords: &["complete", "finished", "success"],
    },
];

const SEARCH_STAGES: [StageInfo; 7] = [
    StageInfo {
        key: "starting",
        label: "Starting Pioneer",
        keywords: &[],
    },
    StageInfo {
        key: "prepare",
        label: "Preparing inputs",
        keywords: &["reading", "loading", "preparing", "initializing"],
    },
    StageInfo {
        key: "presearch",
        label: "Tuning search parameters",
        keywords: &["presearch", "tuning", "estimating"],
    },
    StageInfo {
        key: "first",
        label: "Running first pass search",
        keywords: &["first search", "index search", "first pass"],
    },
    StageInfo {
        key: "quant",
        label: "Running quantification search",
        keywords: &["quant", "quantification", "scoring"],
    },
    StageInfo {
        key: "finishing",
        label: "Finalizing results",
        keywords: &["writing results", "post-processing", "saving"],
    },
    StageInfo {
        key: "complete",
        label: "Completed",
        keywords: &["complete", "finished", "success"],
    },
];

#[derive(Serialize)]
struct ConfigSet {
    default_config: Value,
    simplified_config: Value,
    persisted_config: Option<Value>,
    persisted_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum ConfigSource {
    Binary,
    Partial,
    Fallback,
}

#[derive(Serialize)]
struct LoadConfigsResponse {
    build: ConfigSet,
    search: ConfigSet,
    source: ConfigSource,
    binary_error: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunRequest {
    mode: RunMode,
    config: Value,
}

#[derive(Serialize)]
struct RunStartedPayload {
    mode: RunMode,
    log_path: String,
    config_path: String,
    persisted_path: Option<String>,
}

#[derive(Serialize)]
struct ProgressPayload {
    mode: RunMode,
    stage_key: String,
    stage_label: String,
    progress: f32,
}

#[derive(Serialize)]
struct LogPayload {
    mode: RunMode,
    stream: &'static str,
    line: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunCompletePayload {
    mode: RunMode,
    success: bool,
    exit_code: Option<i32>,
    message: Option<String>,
}

#[tauri::command]
async fn load_configs(app_handle: AppHandle) -> Result<LoadConfigsResponse, String> {
    let fallback_build: Value = serde_json::from_str(FALLBACK_BUILD).map_err(|e| e.to_string())?;
    let fallback_build_simplified: Value =
        serde_json::from_str(FALLBACK_BUILD_SIMPLIFIED).map_err(|e| e.to_string())?;
    let fallback_search: Value =
        serde_json::from_str(FALLBACK_SEARCH).map_err(|e| e.to_string())?;
    let fallback_search_simplified: Value =
        serde_json::from_str(FALLBACK_SEARCH_SIMPLIFIED).map_err(|e| e.to_string())?;

    let mut errors = Vec::new();
    let mut build_defaults = fallback_build.clone();
    let mut search_defaults = fallback_search.clone();
    let mut source = ConfigSource::Fallback;

    match try_fetch_build_defaults() {
        Ok(value) => {
            build_defaults = value;
            source = ConfigSource::Partial;
        }
        Err(err) => errors.push(format!("BuildSpecLib defaults: {err}")),
    }

    match try_fetch_search_defaults() {
        Ok(value) => {
            search_defaults = value;
            source = match source {
                ConfigSource::Partial | ConfigSource::Binary => ConfigSource::Binary,
                ConfigSource::Fallback => ConfigSource::Partial,
            };
        }
        Err(err) => errors.push(format!("SearchDIA defaults: {err}")),
    }

    if matches!(source, ConfigSource::Partial) && errors.len() == 2 {
        source = ConfigSource::Fallback;
    }

    let resolver = app_handle.path_resolver();
    let build_path = config_storage_path(RunMode::BuildSpecLib, &resolver);
    let search_path = config_storage_path(RunMode::SearchDia, &resolver);

    let build_persisted = load_persisted_config(build_path.as_deref(), &build_defaults);
    let search_persisted = load_persisted_config(search_path.as_deref(), &search_defaults);

    let response = LoadConfigsResponse {
        build: ConfigSet {
            default_config: build_defaults,
            simplified_config: fallback_build_simplified,
            persisted_config: build_persisted,
            persisted_path: build_path.map(|p| p.to_string_lossy().to_string()),
        },
        search: ConfigSet {
            default_config: search_defaults,
            simplified_config: fallback_search_simplified,
            persisted_config: search_persisted,
            persisted_path: search_path.map(|p| p.to_string_lossy().to_string()),
        },
        source,
        binary_error: if errors.is_empty() {
            None
        } else {
            Some(errors.join("\n"))
        },
    };

    Ok(response)
}

fn try_fetch_build_defaults() -> Result<Value, ConfigLoadError> {
    let pioneer = locate_pioneer_binary()?;
    let temp_dir = tempdir().map_err(|e| ConfigLoadError::Other(e.to_string()))?;
    let lib_out = temp_dir.path().join("library_preview");
    fs::create_dir_all(&lib_out).map_err(|e| ConfigLoadError::Other(e.to_string()))?;
    let fasta_path = temp_dir.path().join("preview.fasta");
    fs::write(&fasta_path, b">Example\nM\n").map_err(|e| ConfigLoadError::Other(e.to_string()))?;
    let params_path = temp_dir.path().join("build_params.json");

    let status = StdCommand::new(pioneer)
        .arg("params-predict")
        .arg(lib_out.as_os_str())
        .arg("PreviewLibrary")
        .arg(fasta_path.as_os_str())
        .arg("--params-path")
        .arg(&params_path)
        .status()?;

    if !status.success() {
        return Err(ConfigLoadError::NonZeroExit(status.code()));
    }

    let file =
        fs::read_to_string(&params_path).map_err(|e| ConfigLoadError::Other(e.to_string()))?;
    let json: Value = serde_json::from_str(&file)?;
    Ok(json)
}

fn try_fetch_search_defaults() -> Result<Value, ConfigLoadError> {
    let pioneer = locate_pioneer_binary()?;
    let temp_dir = tempdir().map_err(|e| ConfigLoadError::Other(e.to_string()))?;
    let library_path = temp_dir.path().join("example_library.poin");
    fs::write(&library_path, b"").map_err(|e| ConfigLoadError::Other(e.to_string()))?;
    let ms_data_dir = temp_dir.path().join("ms_data");
    fs::create_dir_all(&ms_data_dir).map_err(|e| ConfigLoadError::Other(e.to_string()))?;
    let results_dir = temp_dir.path().join("results");
    fs::create_dir_all(&results_dir).map_err(|e| ConfigLoadError::Other(e.to_string()))?;
    let params_path = temp_dir.path().join("search_params.json");

    let status = StdCommand::new(pioneer)
        .arg("params-search")
        .arg(library_path.as_os_str())
        .arg(ms_data_dir.as_os_str())
        .arg(results_dir.as_os_str())
        .arg("--params-path")
        .arg(&params_path)
        .status()?;

    if !status.success() {
        return Err(ConfigLoadError::NonZeroExit(status.code()));
    }

    let file =
        fs::read_to_string(&params_path).map_err(|e| ConfigLoadError::Other(e.to_string()))?;
    let json: Value = serde_json::from_str(&file)?;
    Ok(json)
}

#[tauri::command]
async fn read_config(path: String) -> Result<Value, String> {
    let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&contents).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_config(path: String, config: Value) -> Result<(), String> {
    let pretty = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, pretty).map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_pioneer(
    window: Window,
    app_handle: AppHandle,
    request: RunRequest,
) -> Result<RunStartedPayload, String> {
    let pioneer_path = locate_pioneer_binary().map_err(|e| e.to_string())?;
    let temp_dir = tempdir().map_err(|e| e.to_string())?;
    let config_path = temp_dir.path().join(request.mode.config_filename());
    let config_str = serde_json::to_string_pretty(&request.config).map_err(|e| e.to_string())?;
    fs::write(&config_path, config_str).map_err(|e| e.to_string())?;

    let persisted_path = persist_config(&app_handle, request.mode, &request.config)?;

    let persisted_path_string = persisted_path.map(|p| p.to_string_lossy().to_string());

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    let log_path = temp_dir.path().join(format!("pioneer_run_{timestamp}.log"));
    FileCreator::create_empty(&log_path).map_err(|e| e.to_string())?;

    let payload = RunStartedPayload {
        mode: request.mode,
        log_path: log_path.to_string_lossy().to_string(),
        config_path: config_path.to_string_lossy().to_string(),
        persisted_path: persisted_path_string.clone(),
    };

    window
        .emit("pioneer-run-started", &payload)
        .map_err(|e| e.to_string())?;

    let thread_window = window.clone();
    std::thread::spawn(move || {
        let _temp_dir = temp_dir;
        if let Err(err) = run_process(
            thread_window,
            pioneer_path,
            request.mode,
            config_path,
            log_path,
        ) {
            eprintln!("Failed to run Pioneer: {err}");
        }
    });

    Ok(payload)
}

fn persist_config(
    app_handle: &AppHandle,
    mode: RunMode,
    config: &Value,
) -> Result<Option<PathBuf>, String> {
    let resolver = app_handle.path_resolver();
    let Some(path) = config_storage_path(mode, &resolver) else {
        return Ok(None);
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let pretty = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, pretty).map_err(|e| e.to_string())?;
    Ok(Some(path))
}

fn run_process(
    window: Window,
    pioneer: PathBuf,
    mode: RunMode,
    config_path: PathBuf,
    log_path: PathBuf,
) -> Result<(), String> {
    if let Err(err) = open_terminal_tail(&log_path) {
        let _ = window.emit(
            "pioneer-terminal-warning",
            &format!("Could not launch external terminal: {err}"),
        );
    }

    let mut command = StdCommand::new(&pioneer);
    command
        .arg(mode.subcommand())
        .arg(&config_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Missing stdout pipe".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Missing stderr pipe".to_string())?;

    let (tx, rx) = mpsc::channel::<(&'static str, String)>();

    spawn_reader(stdout, tx.clone(), "stdout");
    spawn_reader(stderr, tx.clone(), "stderr");
    drop(tx);

    let mut stage_index = 0usize;
    let stages = mode.stage_sequence();
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| e.to_string())?;

    send_stage_update(&window, mode, stages, stage_index);

    while let Ok((stream, line)) = rx.recv() {
        writeln!(log_file, "{stream}: {line}").ok();
        let _ = window.emit(
            "pioneer-log",
            &LogPayload {
                mode,
                stream,
                line: line.clone(),
            },
        );

        if let Some(next_index) = match_stage(&line, stage_index, stages) {
            if next_index > stage_index {
                stage_index = next_index;
                send_stage_update(&window, mode, stages, stage_index);
            }
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        stage_index = stages.len() - 1;
        send_stage_update(&window, mode, stages, stage_index);
        let _ = window.emit(
            "pioneer-run-complete",
            &RunCompletePayload {
                mode,
                success: true,
                exit_code: status.code(),
                message: None,
            },
        );
    } else {
        let message = format!(
            "Pioneer exited with status {:?}",
            status.code().or(Some(-1))
        );
        let _ = window.emit(
            "pioneer-run-complete",
            &RunCompletePayload {
                mode,
                success: false,
                exit_code: status.code(),
                message: Some(message.clone()),
            },
        );
        return Err(message);
    }

    Ok(())
}

fn spawn_reader<R: std::io::Read + Send + 'static>(
    reader: R,
    tx: mpsc::Sender<(&'static str, String)>,
    label: &'static str,
) {
    std::thread::spawn(move || {
        let buf_reader = BufReader::new(reader);
        for line in buf_reader.lines().flatten() {
            if tx.send((label, line)).is_err() {
                break;
            }
        }
    });
}

fn match_stage(line: &str, current_index: usize, stages: &[StageInfo]) -> Option<usize> {
    let normalized = line.to_lowercase();
    for (idx, stage) in stages.iter().enumerate().skip(current_index + 1) {
        if stage
            .keywords
            .iter()
            .any(|keyword| keyword.is_empty() || normalized.contains(keyword))
        {
            return Some(idx);
        }
    }
    None
}

fn send_stage_update(window: &Window, mode: RunMode, stages: &[StageInfo], index: usize) {
    let stage = &stages[index];
    let progress = if stages.len() <= 1 {
        100.0
    } else {
        (index as f32 / (stages.len() - 1) as f32) * 100.0
    };
    let _ = window.emit(
        "pioneer-progress",
        &ProgressPayload {
            mode,
            stage_key: stage.key.to_string(),
            stage_label: stage.label.to_string(),
            progress,
        },
    );
}

struct FileCreator;

impl FileCreator {
    fn create_empty(path: &Path) -> Result<(), std::io::Error> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        Ok(())
    }
}

fn open_terminal_tail(log_path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let command = format!(
            "Start-Process powershell -ArgumentList '-NoExit','-Command','Get-Content -Path \"{}\" -Wait'",
            log_path.display()
        );
        Command::new("powershell")
            .args(["-NoProfile", "-Command", &command])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let escaped = log_path.display().to_string().replace('"', "\\\"");
        let script = format!(
            "tell application \"Terminal\" to do script \"tail -n +1 -f {}\"",
            escaped
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let terminals = [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "mate-terminal",
            "xterm",
        ];
        let mut selected = None;
        for term in terminals.iter() {
            if which(term).is_ok() {
                selected = Some(*term);
                break;
            }
        }

        let Some(term) = selected else {
            return Err("No compatible terminal found".into());
        };

        let tail_command = format!(
            "tail -n +1 -f '{}' ; read -p \"Press Enter to close...\" _",
            log_path.display()
        );

        let result = match term {
            "gnome-terminal" | "mate-terminal" => Command::new(term)
                .args(["--", "bash", "-lc", &tail_command])
                .spawn(),
            "konsole" => Command::new(term)
                .args(["--noclose", "-e", "bash", "-lc", &tail_command])
                .spawn(),
            "xfce4-terminal" => Command::new(term)
                .args(["--hold", "-e", "bash", "-lc", &tail_command])
                .spawn(),
            "xterm" => Command::new(term)
                .args(["-hold", "-e", "bash", "-lc", &tail_command])
                .spawn(),
            _ => Command::new(term)
                .args(["-e", "bash", "-lc", &tail_command])
                .spawn(),
        };

        result.map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Unsupported platform".into())
}

fn config_storage_path(mode: RunMode, resolver: &PathResolver) -> Option<PathBuf> {
    let mut path = resolver.app_config_dir()?;
    let filename = match mode {
        RunMode::BuildSpecLib => "buildspeclib.json",
        RunMode::SearchDia => "searchdia.json",
    };
    path.push(filename);
    Some(path)
}

fn load_persisted_config(path: Option<&Path>, defaults: &Value) -> Option<Value> {
    let path = path?;
    let contents = fs::read_to_string(path).ok()?;
    let persisted: Value = serde_json::from_str(&contents).ok()?;
    Some(deep_merge(defaults, &persisted))
}

fn deep_merge(base: &Value, override_val: &Value) -> Value {
    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            let mut merged = base_map.clone();
            for (key, value) in override_map.iter() {
                let next = if let Some(existing) = merged.get(key) {
                    deep_merge(existing, value)
                } else {
                    value.clone()
                };
                merged.insert(key.clone(), next);
            }
            Value::Object(merged)
        }
        _ => override_val.clone(),
    }
}

fn env_pioneer_candidates() -> Vec<PathBuf> {
    const ENV_VARS: &[&str] = &["PIONEER_BINARY", "PIONEER_PATH", "PIONEER_EXE", "PIONEER"];

    let mut results = Vec::new();
    for key in ENV_VARS.iter() {
        if let Some(raw) = env::var_os(key) {
            if raw.is_empty() {
                continue;
            }
            let path = PathBuf::from(&raw);
            if path.is_file() {
                results.push(path);
            } else if path.is_dir() {
                for candidate in ["pioneer", "Pioneer", "pioneer.exe", "Pioneer.exe"] {
                    results.push(path.join(candidate));
                }
            } else {
                results.push(path);
            }
        }
    }
    results
}

fn locate_pioneer_binary() -> Result<PathBuf, ConfigLoadError> {
    for candidate in env_pioneer_candidates() {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    const CANDIDATES: &[&str] = &["pioneer", "Pioneer", "pioneer.exe", "Pioneer.exe"];
    for candidate in CANDIDATES {
        if let Ok(path) = which(candidate) {
            return Ok(path);
        }
    }
    Err(ConfigLoadError::MissingBinary)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_configs,
            read_config,
            save_config,
            run_pioneer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
