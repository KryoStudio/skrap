//! `skrap` on the command line. Read-only for now: it shows what a scan finds and what the
//! Reserve holds, it never deletes anything. Cleaning from here comes with the portable version.

use std::path::PathBuf;
use std::process::ExitCode;

use skrap_core::{Progress, Reserve, Risk, Rule, Skip, scan};

const HELP: &str = "\
skrap: the free, open-source PC cleaner (read-only command line)

Usage:
  skrap scan [--all] [--lang <code>]   what a scan finds; nothing is deleted
  skrap rules [--lang <code>]          every built-in rule and its risk
  skrap reserve                        what the Reserve holds
  skrap --version

--all    also scans the rules that are unticked by default
--lang   en or fr (default: en)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let lang = option(&args, "--lang").unwrap_or_else(|| "en".into());
    let result = match args.first().map(String::as_str) {
        Some("scan") => run_scan(args.iter().any(|a| a == "--all"), &lang),
        Some("rules") => run_rules(&lang),
        Some("reserve") => run_reserve(),
        Some("--version" | "-V") => {
            println!("skrap {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("--help" | "-h" | "help") | None => {
            println!("{HELP}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command: {other}\n\n{HELP}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn option(args: &[String], name: &str) -> Option<String> {
    let at = args.iter().position(|a| a == name)?;
    args.get(at + 1).cloned()
}

fn run_scan(all: bool, lang: &str) -> Result<(), String> {
    let rules = skrap_rules::builtin().map_err(|e| e.to_string())?;
    let ctx = skrap_win::context(&[reserve_home()]);
    let progress = Progress::new();
    let wanted = |rule: &Rule| all || rule.ticked_by_default();
    let report = scan(&rules, &wanted, &ctx, &progress);

    let mut total = 0;
    for finding in &report.findings {
        let Some(rule) = rules.get(&finding.rule) else {
            continue;
        };
        let note = match &finding.skip {
            Some(Skip::Running { exe }) => format!("  (skipped: {exe} is running)"),
            None => {
                total += finding.bytes;
                String::new()
            }
        };
        println!(
            "{:>10}  {:<7} {} · {}{note}",
            size(finding.bytes),
            risk(rule.risk),
            rule.group.get(lang),
            rule.name.get(lang),
        );
    }
    println!(
        "\n{} to clean, {} files looked at in {} ms. Nothing was deleted.",
        size(total),
        progress.snapshot().files,
        report.duration_ms
    );
    if !all {
        println!("Unticked rules were left out: add --all to see them.");
    }
    Ok(())
}

fn run_rules(lang: &str) -> Result<(), String> {
    let rules = skrap_rules::builtin().map_err(|e| e.to_string())?;
    for rule in rules.rules() {
        println!(
            "{:<7} {:<38} {} · {}",
            risk(rule.risk),
            rule.id,
            rule.group.get(lang),
            rule.name.get(lang)
        );
    }
    println!("\n{} rules.", rules.len());
    Ok(())
}

fn run_reserve() -> Result<(), String> {
    let reserve = Reserve::new(reserve_home(), skrap_win::hide);
    let batches = reserve.list();
    if batches.is_empty() {
        println!("The Reserve is empty.");
    }
    for batch in &batches {
        println!(
            "{}  {:>10}  {} files  {}",
            batch.id,
            size(batch.bytes),
            batch.files,
            batch.rules.join(", ")
        );
    }
    Ok(())
}

/// Where the app keeps the Reserve on the system disk (its local app data folder).
fn reserve_home() -> PathBuf {
    let local = std::env::var_os("LOCALAPPDATA").map_or_else(std::env::temp_dir, PathBuf::from);
    local.join("com.kryostudio.skrap").join("reserve")
}

fn risk(risk: Risk) -> &'static str {
    match risk {
        Risk::Safe => "safe",
        Risk::Review => "review",
        Risk::Expert => "expert",
    }
}

/// Sizes as File Explorer counts them (1 KB = 1,024 bytes).
fn size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
