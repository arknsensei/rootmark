//! The text side of `propose` (GOAL P1-D): which document an agent may write
//! and what the new file is. Pure functions; `tools::Kernel::propose` writes
//! and ingests. Nothing here adds anything to the agent's text: no identity,
//! no metadata, no markers (NORTH_STAR principle 1).

use anyhow::{bail, Result};

/// How the agent's text enters the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// a new document; an existing one is never overwritten
    Create,
    /// after the existing document's text
    Append,
    /// as the body of the section under one heading of an existing document
    ReplaceSection,
}

impl Mode {
    pub fn parse(s: &str) -> Result<Mode> {
        Ok(match s {
            "create" => Mode::Create,
            "append" => Mode::Append,
            "replace_section" => Mode::ReplaceSection,
            _ => bail!("propose: unknown mode {s:?} (create, append, replace_section)"),
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Create => "create",
            Mode::Append => "append",
            Mode::ReplaceSection => "replace_section",
        }
    }
}

/// A document path `propose` may write: vault-relative, `/`-separated, a
/// `.md` file, no empty, `.` or `..` component, nothing absolute, no
/// backslash, and no hidden component (`.mdkernel/`, `.obsidian/`, `.x.md`:
/// the documents `ingest-dir` and the watcher never see). Symbolic links
/// are refused by `paths::doc_file`.
pub fn check_doc_path(doc: &str) -> Result<()> {
    if doc.is_empty() || doc.starts_with('/') || doc.contains('\\') || doc.contains('\0') {
        bail!("propose: doc_path {doc:?} must be a vault-relative path with `/` separators");
    }
    for c in doc.split('/') {
        if c.is_empty() || c == "." || c == ".." {
            bail!("propose: doc_path {doc:?}: empty, `.` or `..` component (outside the vault or not normalized)");
        }
        if c.starts_with('.') {
            bail!("propose: doc_path {doc:?}: hidden component {c:?} is not a vault document");
        }
    }
    if !doc.ends_with(".md") || doc.len() <= 3 || doc.ends_with("/.md") {
        bail!("propose: doc_path {doc:?} must name a .md file");
    }
    Ok(())
}

/// `text` ending with exactly one newline (nothing added when empty).
fn one_newline(text: &str) -> String {
    let t = text.trim_end_matches(['\n', '\r']);
    if t.is_empty() {
        String::new()
    } else {
        format!("{t}\n")
    }
}

/// The new document of an `append`: the old text unchanged, a blank line,
/// then the agent's text. Only the added lines differ.
pub fn append(old: &str, content: &str) -> String {
    let add = one_newline(content);
    if old.is_empty() {
        return add;
    }
    let sep = if old.ends_with("\n\n") {
        ""
    } else if old.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    format!("{old}{sep}{add}")
}

/// An ATX heading line (`#`–`######`, then a space or the end): its level
/// and text (closing `#`s and surrounding spaces removed).
fn atx(line: &str) -> Option<(usize, String)> {
    let t = line
        .strip_prefix("   ")
        .or_else(|| line.strip_prefix("  "))
        .or_else(|| line.strip_prefix(' '))
        .unwrap_or(line);
    let level = t.chars().take_while(|&c| c == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = &t[level..];
    if !(rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t')) {
        return None;
    }
    let mut text = rest.trim();
    let stripped = text.trim_end_matches('#');
    if stripped.len() < text.len() && (stripped.is_empty() || stripped.ends_with([' ', '\t'])) {
        text = stripped.trim_end();
    }
    Some((level, text.to_string()))
}

/// The new document of a `replace_section`: the body of the section under
/// the one ATX heading whose text is `heading` (up to the next heading of
/// the same or a higher level, outside fenced code) becomes the agent's
/// text; the heading line and everything else are unchanged. No such
/// heading, or more than one, is an error.
pub fn replace_section(old: &str, heading: &str, content: &str) -> Result<String> {
    let want = heading.trim();
    if want.is_empty() {
        bail!("propose: replace_section needs a non-empty heading");
    }
    let lines: Vec<&str> = old.split_inclusive('\n').collect();
    let mut fence: Option<String> = None;
    let mut heads: Vec<(usize, usize, String)> = Vec::new(); // (line, level, text)
    for (i, l) in lines.iter().enumerate() {
        let body = l.trim_end_matches(['\n', '\r']);
        let lead = body.trim_start();
        if let Some(f) = &fence {
            if lead.starts_with(f.as_str())
                && lead
                    .trim_end()
                    .chars()
                    .all(|c| c == f.chars().next().unwrap())
            {
                fence = None;
            }
            continue;
        }
        if lead.starts_with("```") || lead.starts_with("~~~") {
            let ch = lead.chars().next().unwrap();
            let n = lead.chars().take_while(|&c| c == ch).count();
            fence = Some(ch.to_string().repeat(n));
            continue;
        }
        if let Some((lvl, text)) = atx(body) {
            heads.push((i, lvl, text));
        }
    }
    let found: Vec<&(usize, usize, String)> = heads.iter().filter(|h| h.2 == want).collect();
    let (at, level) = match found.as_slice() {
        [] => bail!("propose: replace_section: no heading {want:?} in the document"),
        [one] => (one.0, one.1),
        many => bail!(
            "propose: replace_section: heading {want:?} occurs {} times (ambiguous)",
            many.len()
        ),
    };
    let end = heads
        .iter()
        .find(|h| h.0 > at && h.1 <= level)
        .map(|h| h.0)
        .unwrap_or(lines.len());
    let mut out = String::new();
    for l in &lines[..=at] {
        out.push_str(l);
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    let body = one_newline(content);
    if !body.is_empty() {
        out.push('\n');
        out.push_str(&body);
    }
    if end < lines.len() {
        out.push('\n');
        for l in &lines[end..] {
            out.push_str(l);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths() {
        for ok in ["a.md", "notes/agent.md", "x/y/z.md"] {
            check_doc_path(ok).unwrap();
        }
        for bad in [
            "",
            "/etc/x.md",
            "../x.md",
            "a/../x.md",
            "./a.md",
            "a//b.md",
            ".mdkernel/x.md",
            "a/.hidden/x.md",
            ".x.md",
            "a.txt",
            ".md",
            "a\\b.md",
            "a/.md",
        ] {
            assert!(check_doc_path(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn append_adds_only_lines() {
        assert_eq!(append("# A\n\nx\n", "y"), "# A\n\nx\n\ny\n");
        assert_eq!(append("# A\n\nx\n\n", "y\n\n"), "# A\n\nx\n\ny\n");
        assert_eq!(append("# A", "y"), "# A\n\ny\n");
        assert_eq!(append("", "y"), "y\n");
    }

    #[test]
    fn replace_section_body() {
        let doc = "# T\n\nintro\n\n## A\n\nold a\n\n### A1\n\nold a1\n\n## B\n\nb\n";
        assert_eq!(
            replace_section(doc, "A", "new a").unwrap(),
            "# T\n\nintro\n\n## A\n\nnew a\n\n## B\n\nb\n"
        );
        assert_eq!(
            replace_section(doc, "B", "new b").unwrap(),
            "# T\n\nintro\n\n## A\n\nold a\n\n### A1\n\nold a1\n\n## B\n\nnew b\n"
        );
        assert!(replace_section(doc, "C", "x").is_err());
        let fenced = "## A\n\n```\n## A\n```\n\n## B\n";
        assert_eq!(
            replace_section(fenced, "A", "z").unwrap(),
            "## A\n\nz\n\n## B\n"
        );
        assert!(replace_section("## A\n\n## A\n", "A", "x").is_err());
        assert_eq!(
            replace_section("## A ##\n\nq\n", "A", "r").unwrap(),
            "## A ##\n\nr\n"
        );
    }
}
