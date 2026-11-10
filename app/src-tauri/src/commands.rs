//! What the page can ask. Errors are short codes the page translates.

use std::path::PathBuf;

use serde::Serialize;
use skrap_core::{BatchSummary, Category, RestoreReport, Risk, unix_ms};
use skrap_win::Disk;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::jobs;
use crate::persist;
use crate::settings::{Settings, SettingsPatch, Stats};
use crate::state::{AppState, lock};

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings()
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    patch: SettingsPatch,
) -> CommandResult<Settings> {
    let mut settings = lock(&state.settings);
    let next = settings.merge(patch)?;
    persist::save(&state.settings_path, &next).map_err(|_| "save_failed")?;
    *settings = next.clone();
    Ok(next)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    version: String,
    elevated: bool,
    disks: Vec<Disk>,
    stats: Stats,
    reserve: ReserveTotals,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReserveTotals {
    bytes: u64,
    files: u64,
    batches: usize,
    /// When the oldest batch is freed for good (Unix ms).
    next_release: Option<u64>,
}

#[tauri::command]
pub async fn get_overview(app: AppHandle) -> Overview {
    let state = app.state::<AppState>();
    let batches = state.reserve.list();
    let delay = state.settings().reserve_ms();
    Overview {
        version: app.package_info().version.to_string(),
        elevated: skrap_win::is_elevated(),
        disks: skrap_win::disks(),
        stats: lock(&state.stats).clone(),
        reserve: ReserveTotals {
            bytes: batches.iter().map(|b| b.bytes).sum(),
            files: batches.iter().map(|b| b.files).sum(),
            batches: batches.len(),
            next_release: batches.iter().map(|b| b.created + delay).min(),
        },
    }
}

/// A rule as the page shows it, its texts in one language.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleView {
    id: String,
    category: Category,
    risk: Risk,
    group: String,
    name: String,
    explain: String,
    ticked: bool,
    admin: bool,
    /// Emptied through Windows (the Recycle Bin): never goes to the Reserve.
    special: bool,
}

#[tauri::command]
pub fn get_rules(state: State<'_, AppState>, lang: String) -> Vec<RuleView> {
    state
        .rules
        .rules()
        .iter()
        .filter(|rule| rule.risk != Risk::Expert)
        .map(|rule| RuleView {
            id: rule.id.clone(),
            category: rule.category,
            risk: rule.risk,
            group: rule.group.get(&lang).to_owned(),
            name: rule.name.get(&lang).to_owned(),
            explain: rule.explain.get(&lang).to_owned(),
            ticked: rule.ticked_by_default(),
            admin: rule.admin,
            special: rule.special.is_some(),
        })
        .collect()
}

#[tauri::command]
pub fn start_scan(app: AppHandle) -> CommandResult<()> {
    jobs::start_scan(&app).map_err(Into::into)
}

#[tauri::command]
pub fn start_clean(app: AppHandle, rules: Vec<String>) -> CommandResult<()> {
    jobs::start_clean(&app, rules).map_err(Into::into)
}

#[tauri::command]
pub fn cancel_job(app: AppHandle) {
    jobs::cancel(&app);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileView {
    path: PathBuf,
    size: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleFiles {
    /// The largest first.
    files: Vec<FileView>,
    total: u64,
}

/// The largest files a rule found in the last scan.
#[tauri::command]
pub fn get_rule_files(state: State<'_, AppState>, rule: String, limit: usize) -> RuleFiles {
    let scan = lock(&state.scan);
    let Some(finding) = scan.as_ref().and_then(|s| s.finding(&rule)) else {
        return RuleFiles {
            files: Vec::new(),
            total: 0,
        };
    };
    let mut items: Vec<&skrap_core::Item> = finding.items.iter().collect();
    items.sort_by_key(|item| std::cmp::Reverse(item.size));
    RuleFiles {
        files: items
            .into_iter()
            .take(limit.min(500))
            .map(|item| FileView {
                path: item.path.clone(),
                size: item.size,
            })
            .collect(),
        total: finding.files,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchView {
    #[serde(flatten)]
    summary: BatchSummary,
    /// When it is freed for good (Unix ms).
    expires: u64,
}

#[tauri::command]
pub async fn get_reserve(app: AppHandle) -> Vec<BatchView> {
    let state = app.state::<AppState>();
    let delay = state.settings().reserve_ms();
    state
        .reserve
        .list()
        .into_iter()
        .map(|summary| BatchView {
            expires: summary.created + delay,
            summary,
        })
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservedFile {
    path: PathBuf,
    size: u64,
    rule: String,
}

#[tauri::command]
pub async fn get_reserve_files(app: AppHandle, batch: String, limit: usize) -> Vec<ReservedFile> {
    let mut entries = app.state::<AppState>().reserve.entries(&batch);
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.size));
    entries
        .into_iter()
        .take(limit.min(500))
        .map(|e| ReservedFile {
            path: e.original,
            size: e.size,
            rule: e.rule,
        })
        .collect()
}

#[tauri::command]
pub async fn restore_batch(app: AppHandle, batch: String) -> CommandResult<RestoreReport> {
    // Never while a clean may still be filling a batch, nor while another change runs.
    let report = jobs::exclusive(&app, || {
        app.state::<AppState>().reserve.restore(&batch, None)
    })?;
    let _ = app.emit(jobs::RESERVE_CHANGED, 0);
    Ok(report)
}

/// Frees one batch for good, or the whole Reserve (`None`). Returns the bytes freed.
#[tauri::command]
pub async fn release_batch(app: AppHandle, batch: Option<String>) -> CommandResult<u64> {
    let freed = jobs::exclusive(&app, || {
        let reserve = &app.state::<AppState>().reserve;
        match batch {
            Some(id) => reserve.purge(&id),
            None => reserve.purge_all(),
        }
    })?;
    let _ = app.emit(jobs::RESERVE_CHANGED, freed);
    Ok(freed)
}

/// Shows a file (or the folder it was in) in File Explorer.
#[tauri::command]
pub fn reveal(path: PathBuf) -> CommandResult<()> {
    let target = if path.exists() {
        path
    } else {
        path.ancestors()
            .find(|p| p.is_dir())
            .map(PathBuf::from)
            .ok_or("not_found")?
    };
    tauri_plugin_opener::reveal_item_in_dir(target).map_err(|_| "reveal_failed".into())
}

/// The current time on Rust's clock, so the page and the Reserve agree on dates.
#[tauri::command]
pub fn now() -> u64 {
    unix_ms(std::time::SystemTime::now())
}
