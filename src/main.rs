use std::env;
use std::io::Read;
use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize)]
struct StatusJSON {
    cwd: Option<String>,
    workspace: Option<Workspace>,
    context_window: Option<ContextWindow>,
}

#[derive(Deserialize)]
struct Workspace {
    current_dir: Option<String>,
    project_dir: Option<String>,
}

#[derive(Deserialize)]
struct ContextWindow {
    used_percentage: Option<f64>,
    context_window_size: Option<f64>,
    current_usage: Option<CurrentUsage>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CurrentUsage {
    Number(f64),
    Detailed(DetailedUsage),
}

#[derive(Deserialize)]
struct DetailedUsage {
    input_tokens: Option<f64>,
    output_tokens: Option<f64>,
    cache_creation_input_tokens: Option<f64>,
    cache_read_input_tokens: Option<f64>,
}

fn resolve_cwd(data: &StatusJSON) -> Option<&str> {
    if let Some(ref cwd) = data.cwd {
        let trimmed = cwd.trim();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    if let Some(ref ws) = data.workspace {
        if let Some(ref dir) = ws.current_dir {
            let trimmed = dir.trim();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
        if let Some(ref dir) = ws.project_dir {
            let trimmed = dir.trim();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

fn run_git(args: &str, cwd: Option<&str>) -> Option<String> {
    let mut cmd = Command::new("git");
    for arg in args.split_whitespace() {
        cmd.arg(arg);
    }
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    cmd.stderr(std::process::Stdio::null());
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn format_context_percentage(pct: f64) -> String {
    let text = format!("{:.0}%", pct);
    if pct > 75.0 {
        format!("\x1b[31m{}\x1b[0m", text)
    } else {
        text
    }
}

fn context_percentage(data: &StatusJSON) -> Option<String> {
    let cw = data.context_window.as_ref()?;

    if let Some(pct) = cw.used_percentage {
        if pct.is_finite() && pct >= 0.0 {
            return Some(format_context_percentage(pct.min(100.0)));
        }
    }

    let window_size = cw.context_window_size.filter(|v| v.is_finite() && *v > 0.0)?;
    let used = match cw.current_usage.as_ref()? {
        CurrentUsage::Number(n) => {
            if n.is_finite() && *n >= 0.0 { *n } else { return None }
        }
        CurrentUsage::Detailed(d) => {
            let input = d.input_tokens.unwrap_or(0.0);
            let output = d.output_tokens.unwrap_or(0.0);
            let cache_create = d.cache_creation_input_tokens.unwrap_or(0.0);
            let cache_read = d.cache_read_input_tokens.unwrap_or(0.0);
            input + output + cache_create + cache_read
        }
    };

    let pct = (used / window_size * 100.0).min(100.0);
    Some(format_context_percentage(pct))
}

fn is_inside_git_work_tree(cwd: Option<&str>) -> bool {
    run_git("rev-parse --is-inside-work-tree", cwd).as_deref() == Some("true")
}

fn color(code: &str, text: &str) -> String {
    format!("\x1b[{}m{}\x1b[0m", code, text)
}

fn tilde_path(path: &str) -> String {
    if let Ok(home) = env::var("HOME") {
        if path.starts_with(&home) {
            return format!("~{}", &path[home.len()..]);
        }
    }
    path.to_string()
}

fn git_root_dir(cwd: Option<&str>) -> String {
    if !is_inside_git_work_tree(cwd) {
        return String::new();
    }
    match run_git("rev-parse --show-toplevel", cwd) {
        Some(root) => tilde_path(&root),
        None => String::new(),
    }
}

fn git_branch(cwd: Option<&str>) -> String {
    let branch = run_git("branch --show-current", cwd).unwrap_or_default();
    color("35", &branch)
}

fn git_status(cwd: Option<&str>) -> String {
    let output = run_git("status --porcelain", cwd).unwrap_or_default();

    let mut modified = false;
    let mut staged = false;
    let mut untracked = false;
    let mut deleted = false;

    for line in output.lines() {
        let bytes = line.as_bytes();
        if bytes.len() < 2 {
            continue;
        }
        let index = bytes[0];
        let worktree = bytes[1];

        if index == b'?' {
            untracked = true;
            continue;
        }
        if matches!(index, b'A' | b'M' | b'R' | b'C') {
            staged = true;
        }
        if index == b'D' || worktree == b'D' {
            deleted = true;
        }
        if worktree == b'M' {
            modified = true;
        }
        if matches!(index, b'D') {
            staged = true;
        }
    }

    let mut flags = String::new();
    if modified { flags.push_str(&color("33", "M")); }
    if staged { flags.push_str(&color("32", "S")); }
    if untracked { flags.push_str(&color("31", "?")); }
    if deleted { flags.push_str(&color("33", "D")); }

    let ahead_behind = git_ahead_behind(cwd);
    if !ahead_behind.is_empty() {
        flags.push_str(&ahead_behind);
    }

    flags
}

fn git_ahead_behind(cwd: Option<&str>) -> String {
    let output = run_git("rev-list --left-right --count @{upstream}...HEAD", cwd)
        .unwrap_or_default();
    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() != 2 {
        return String::new();
    }
    let behind: i64 = parts[0].parse().unwrap_or(0);
    let ahead: i64 = parts[1].parse().unwrap_or(0);

    let mut result = String::new();
    if ahead > 0 {
        result.push('⇡');
    }
    if behind > 0 {
        result.push('⇣');
    }
    result
}

fn aws_info() -> String {
    let profile = env::var("AWS_PROFILE").unwrap_or_default();
    let region = env::var("AWS_REGION")
        .or_else(|_| env::var("AWS_DEFAULT_REGION"))
        .unwrap_or_default();

    let text = match (profile.is_empty(), region.is_empty()) {
        (true, true) => return String::new(),
        (false, true) => profile,
        (true, false) => region,
        (false, false) => format!("{}/{}", profile, region),
    };
    color("33", &text)
}

fn main() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("Failed to read stdin");
        std::process::exit(1);
    }

    let input = input.trim();
    if input.is_empty() {
        eprintln!("No input received");
        std::process::exit(1);
    }

    let data: StatusJSON = match serde_json::from_str(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            std::process::exit(1);
        }
    };

    let cwd = resolve_cwd(&data);

    let ctx_pct = context_percentage(&data).unwrap_or_default();
    let in_git = is_inside_git_work_tree(cwd);
    let aws = aws_info();

    let mut line = String::new();

    if !ctx_pct.is_empty() {
        line.push_str(&ctx_pct);
        line.push_str(" | ");
    }

    if in_git {
        let root = git_root_dir(cwd);
        let branch = git_branch(cwd);
        let changes = git_status(cwd);

        if !root.is_empty() {
            line.push_str(&root);
        }
        if !branch.is_empty() {
            line.push_str(&format!(" ({})", branch));
        }
        if !changes.is_empty() {
            line.push_str(&format!(" [{}]", changes));
        }
    } else {
        let dir_display = match cwd {
            Some(dir) => tilde_path(dir),
            None => env::current_dir()
                .map(|p| tilde_path(&p.to_string_lossy()))
                .unwrap_or_default(),
        };
        line.push_str(&dir_display);
    }

    if !aws.is_empty() {
        line.push_str(&format!(" | {}", aws));
    }
    let output = format!("\x1b[0m{}", line.replace(' ', "\u{00A0}"));
    println!("{}", output);
}
