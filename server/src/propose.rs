//! The text side of `propose` (GOAL P1-D): which document an agent may write
//! and what the new file is. Pure functions; `tools::Kernel::propose` writes
//! and ingests. Nothing here adds anything to the agent's text: no identity,
//! no metadata, no markers (NORTH_STAR principle 1).

use crate::bail_code;
use crate::error::Code;
use anyhow::Result;

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
            _ => bail_code!(
                Code::InvalidArguments,
                "propose: unknown mode {s:?} (create, append, replace_section)"
            ),
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
/// are refused by `paths::doc_file`. A path that leaves the vault or its
/// documents is `path_outside_vault`; one that is not a `.md` name,
/// `invalid_arguments` (K-D1.5).
pub fn check_doc_path(doc: &str) -> Result<()> {
    if doc.is_empty() || doc.starts_with('/') || doc.contains('\\') || doc.contains('\0') {
        bail_code!(
            Code::PathOutsideVault,
            "propose: doc_path {doc:?} must be a vault-relative path with `/` separators"
        );
    }
    for c in doc.split('/') {
        if c.is_empty() || c == "." || c == ".." {
            bail_code!(
                Code::PathOutsideVault,
                "propose: doc_path {doc:?} must be a vault-relative path: empty, `.` or `..` component (outside the vault or not normalized)"
            );
        }
        if c.starts_with('.') {
            bail_code!(
                Code::PathOutsideVault,
                "propose: doc_path {doc:?}: hidden component {c:?} is not a vault document"
            );
        }
    }
    if !doc.ends_with(".md") || doc.len() <= 3 || doc.ends_with("/.md") {
        bail_code!(
            Code::InvalidArguments,
            "propose: doc_path {doc:?} must name a .md file"
        );
    }
    Ok(())
}

/// The line ending of a document: CRLF if it uses CRLF, else LF. The
/// agent's text and the separators take the document's line ending, so a
/// CRLF file never gets mixed endings.
fn eol(old: &str) -> &'static str {
    if old.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// `text` with line ending `eol` throughout, ending with exactly one
/// (nothing when empty).
fn lines_with(text: &str, eol: &str) -> String {
    let t = text.replace("\r\n", "\n");
    let t = t.trim_end_matches('\n');
    if t.is_empty() {
        return String::new();
    }
    let mut out = t.replace('\n', eol);
    out.push_str(eol);
    out
}

/// The new document of an `append`: the old text unchanged, a blank line,
/// then the agent's text. Only the added lines differ.
pub fn append(old: &str, content: &str) -> String {
    let e = eol(old);
    let add = lines_with(content, e);
    if old.is_empty() {
        return add;
    }
    let sep = if old.ends_with(&format!("{e}{e}")) || old.ends_with("\n\n") {
        String::new()
    } else if old.ends_with('\n') {
        e.to_string()
    } else {
        format!("{e}{e}")
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
    let (at, end) = section_lines(old, heading)?;
    let e = eol(old);
    let lines: Vec<&str> = old.split_inclusive('\n').collect();
    let mut out = String::new();
    for l in &lines[..=at] {
        out.push_str(l);
    }
    if !out.ends_with('\n') {
        out.push_str(e);
    }
    let body = lines_with(content, e);
    if !body.is_empty() {
        out.push_str(e);
        out.push_str(&body);
    }
    if end < lines.len() {
        out.push_str(e);
        for l in &lines[end..] {
            out.push_str(l);
        }
    }
    Ok(out)
}

/// The section `replace_section` addresses: `(at, end)`, the 0-based line
/// of its one ATX heading whose text is `heading` and the line after its
/// body (the next heading of the same or a higher level outside fenced
/// code, or the end). No such heading, or more than one, is
/// `heading_not_found`.
pub fn section_lines(old: &str, heading: &str) -> Result<(usize, usize)> {
    let want = heading.trim();
    if want.is_empty() {
        bail_code!(
            Code::InvalidArguments,
            "propose: replace_section needs a non-empty heading"
        );
    }
    let lines: Vec<&str> = old.split_inclusive('\n').collect();
    // YAML front matter (`---` on the first line up to the next `---` or
    // `...`) holds no headings
    let body_start = if lines.first().map(|l| l.trim_end()) == Some("---") {
        lines
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, l)| matches!(l.trim_end(), "---" | "..."))
            .map(|(i, _)| i + 1)
            .unwrap_or(0)
    } else {
        0
    };
    let mut fence: Option<String> = None;
    let mut heads: Vec<(usize, usize, String)> = Vec::new(); // (line, level, text)
    for (i, l) in lines.iter().enumerate().skip(body_start) {
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
        [] => bail_code!(
            Code::HeadingNotFound,
            "propose: replace_section: no heading {want:?} in the document"
        ),
        [one] => (one.0, one.1),
        many => bail_code!(
            Code::HeadingNotFound,
            "propose: replace_section: heading {want:?} occurs {} times (ambiguous)",
            many.len()
        ),
    };
    let end = heads
        .iter()
        .find(|h| h.0 > at && h.1 <= level)
        .map(|h| h.0)
        .unwrap_or(lines.len());
    Ok((at, end))
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

    #[test]
    fn front_matter_holds_no_heading() {
        let fm = "---\ntitle: x\n# A\n---\n## A\n\nold\n";
        assert_eq!(
            replace_section(fm, "A", "new").unwrap(),
            "---\ntitle: x\n# A\n---\n## A\n\nnew\n"
        );
    }

    #[test]
    fn crlf_documents_stay_crlf() {
        assert_eq!(
            append("# A\r\n\r\nx\r\n", "y\nz"),
            "# A\r\n\r\nx\r\n\r\ny\r\nz\r\n"
        );
        assert_eq!(
            replace_section("## A\r\n\r\nold\r\n\r\n## B\r\n", "A", "n1\nn2").unwrap(),
            "## A\r\n\r\nn1\r\nn2\r\n\r\n## B\r\n"
        );
    }
}
