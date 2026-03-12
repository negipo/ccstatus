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
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn context_percentage(data: &StatusJSON) -> Option<String> {
    let cw = data.context_window.as_ref()?;

    if let Some(pct) = cw.used_percentage {
        if pct.is_finite() && pct >= 0.0 {
            return Some(format!("{:.1}%", pct.min(100.0)));
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
    Some(format!("{:.1}%", pct))
}

fn is_inside_git_work_tree(cwd: Option<&str>) -> bool {
    run_git("rev-parse --is-inside-work-tree", cwd).as_deref() == Some("true")
}

fn git_root_dir(cwd: Option<&str>) -> String {
    if !is_inside_git_work_tree(cwd) {
        return "no git".to_string();
    }
    match run_git("rev-parse --show-toplevel", cwd) {
        Some(root) => {
            let trimmed = root.trim_end_matches(['/', '\\']);
            let name = trimmed.rsplit_once(['/', '\\']).map_or(trimmed, |(_, n)| n);
            if name.is_empty() { root } else { name.to_string() }
        }
        None => "no git".to_string(),
    }
}

fn git_branch(cwd: Option<&str>) -> String {
    if !is_inside_git_work_tree(cwd) {
        return "no git".to_string();
    }
    run_git("branch --show-current", cwd).unwrap_or_else(|| "no git".to_string())
}

fn extract_last_number(text: &str) -> i64 {
    let trimmed = text.trim();
    let start = trimmed
        .rfind(|c: char| !c.is_ascii_digit())
        .map_or(0, |i| i + 1);
    trimmed[start..].parse().unwrap_or(0)
}

fn parse_diff_shortstat(stat: &str) -> (i64, i64) {
    let insertions = stat
        .find("insertion")
        .map(|pos| extract_last_number(&stat[..pos]))
        .unwrap_or(0);
    let deletions = stat
        .find("deletion")
        .map(|pos| extract_last_number(&stat[..pos]))
        .unwrap_or(0);
    (insertions, deletions)
}

fn git_changes(cwd: Option<&str>) -> String {
    if !is_inside_git_work_tree(cwd) {
        return "(no git)".to_string();
    }

    let unstaged = run_git("diff --shortstat", cwd).unwrap_or_default();
    let staged = run_git("diff --cached --shortstat", cwd).unwrap_or_default();

    let (ui, ud) = parse_diff_shortstat(&unstaged);
    let (si, sd) = parse_diff_shortstat(&staged);

    format!("(+{},-{})", ui + si, ud + sd)
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
    let root = git_root_dir(cwd);
    let branch = git_branch(cwd);
    let changes = git_changes(cwd);

    let parts: Vec<&str> = [
        ctx_pct.as_str(),
        root.as_str(),
        branch.as_str(),
        changes.as_str(),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect();

    let line = parts.join(" | ");
    let output = format!("\x1b[0m{}", line.replace(' ', "\u{00A0}"));
    println!("{}", output);
}
