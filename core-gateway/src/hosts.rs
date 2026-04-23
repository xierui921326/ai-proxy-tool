use anyhow::*;
use std::fs;
use std::path::Path;

const MARK_BEGIN: &str = "# AI-PROXY-BEGIN";
const MARK_END: &str = "# AI-PROXY-END";

pub fn apply(entries: &[(&str, &str)]) -> Result<()> {
    let path = "/etc/hosts";
    let orig = fs::read_to_string(path).context("read /etc/hosts failed")?;
    let mut lines: Vec<String> = orig.lines().map(|s| s.to_string()).collect();
    // remove previous block
    if let Some((s, e)) = find_block(&lines) { lines.drain(s..=e); }
    // append new block
    lines.push(MARK_BEGIN.to_string());
    for (ip, host) in entries {
        lines.push(format!("{ip} {host}"));
    }
    lines.push(MARK_END.to_string());
    fs::write(path, lines.join("\n") + "\n").context("write /etc/hosts failed")?;
    Ok(())
}

pub fn revert() -> Result<()> {
    let path = "/etc/hosts";
    let orig = fs::read_to_string(path).context("read /etc/hosts failed")?;
    let mut lines: Vec<String> = orig.lines().map(|s| s.to_string()).collect();
    if let Some((s, e)) = find_block(&lines) { lines.drain(s..=e); }
    fs::write(path, lines.join("\n") + "\n").context("write /etc/hosts failed")?;
    Ok(())
}

fn find_block(lines: &[String]) -> Option<(usize, usize)> {
    let mut begin = None;
    for (i, l) in lines.iter().enumerate() {
        if l.trim() == MARK_BEGIN { begin = Some(i); break; }
    }
    if let Some(s) = begin {
        for (i, l) in lines.iter().enumerate().skip(s) {
            if l.trim() == MARK_END { return Some((s, i)); }
        }
    }
    None
}

pub fn apply_default() -> Result<()> {
    let entries = [
        ("127.0.0.1", "api.openai.com"),
        ("127.0.0.1", "api.anthropic.com"),
    ];
    apply(&entries)
}
