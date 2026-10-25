//! A document's date for the search time filter (P2-B/B3 (b); K-Q.1 (3b);
//! docs/CLIENT_CONTRACT.md §2.2). The date is derived from one of two
//! sources of the vault's `[index]` recipe, never from a file time:
//!
//! ```toml
//! [index]
//! date_from_path = "{yyyy}/{mm}/{dd}"   # a pattern of the document's path
//! # or
//! date_field = "date"                    # a front matter key
//! ```
//!
//! Both at once, a pattern without exactly one `{yyyy}`, `{mm}` and `{dd}`,
//! or an empty key is an error at load naming the field. The date never
//! enters an id, an event or the markdown file: it is computed at search
//! time from the checkpoint's documents (path and front matter).

use anyhow::{bail, Result};
use std::collections::BTreeSet;

/// A calendar date `YYYY-MM-DD` (proleptic Gregorian), comparable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub y: u16,
    pub m: u8,
    pub d: u8,
}

fn days_in(y: u16, m: u8) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400) => 29,
        2 => 28,
        _ => 0,
    }
}

impl Date {
    pub fn new(y: u16, m: u8, d: u8) -> Option<Date> {
        (y >= 1 && (1..=12).contains(&m) && d >= 1 && d <= days_in(y, m)).then_some(Date {
            y,
            m,
            d,
        })
    }

    /// `YYYY-MM-DD` exactly (an argument of the search): anything else is
    /// `None`.
    pub fn parse(s: &str) -> Option<Date> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let num = |r: std::ops::Range<usize>| -> Option<u16> {
            let t = &s[r];
            t.bytes()
                .all(|c| c.is_ascii_digit())
                .then(|| t.parse().ok())?
        };
        Date::new(num(0..4)?, num(5..7)? as u8, num(8..10)? as u8)
    }

    pub fn to_iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.y, self.m, self.d)
    }
}

/// Where a document's date comes from (`[index] date_from_path` or
/// `[index] date_field`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateSource {
    /// the pattern as written, and its parts: literal text and the three
    /// fields in their order
    Path(String, Vec<Part>),
    Field(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    Lit(String),
    Year,
    Month,
    Day,
}

impl DateSource {
    /// `date_from_path`: literal text with exactly one `{yyyy}`, `{mm}` and
    /// `{dd}`; another `{…}`, a `\`, a control character or an absolute
    /// pattern is an error naming the field.
    pub fn path(pattern: &str) -> Result<DateSource> {
        let field = "[index] date_from_path";
        if pattern.is_empty() || pattern.starts_with('/') || pattern.contains('\\') {
            bail!("{field} {pattern:?}: a vault-relative path pattern with {{yyyy}}, {{mm}} and {{dd}}");
        }
        if pattern.chars().any(char::is_control) {
            bail!("{field} {pattern:?}: a control character");
        }
        let mut parts = Vec::new();
        let mut lit = String::new();
        let mut rest = pattern;
        let (mut y, mut m, mut d) = (0, 0, 0);
        while let Some(i) = rest.find('{') {
            if rest[..i].contains('}') {
                bail!("{field} {pattern:?}: a `}}` without `{{`");
            }
            lit.push_str(&rest[..i]);
            let Some(j) = rest[i..].find('}') else {
                bail!("{field} {pattern:?}: an unclosed `{{`");
            };
            let part = match &rest[i..i + j + 1] {
                "{yyyy}" => {
                    y += 1;
                    Part::Year
                }
                "{mm}" => {
                    m += 1;
                    Part::Month
                }
                "{dd}" => {
                    d += 1;
                    Part::Day
                }
                other => {
                    bail!("{field} {pattern:?}: unknown field {other} ({{yyyy}}, {{mm}}, {{dd}})")
                }
            };
            if !lit.is_empty() {
                parts.push(Part::Lit(std::mem::take(&mut lit)));
            }
            parts.push(part);
            rest = &rest[i + j + 1..];
        }
        if rest.contains('}') {
            bail!("{field} {pattern:?}: a `}}` without `{{`");
        }
        lit.push_str(rest);
        if !lit.is_empty() {
            parts.push(Part::Lit(lit));
        }
        if (y, m, d) != (1, 1, 1) {
            bail!("{field} {pattern:?}: needs {{yyyy}}, {{mm}} and {{dd}} exactly once each");
        }
        Ok(DateSource::Path(pattern.to_string(), parts))
    }

    /// `date_field`: a non-empty front matter key without `:` or a line
    /// break.
    pub fn field(key: &str) -> Result<DateSource> {
        if key.trim().is_empty()
            || key != key.trim()
            || key.contains(':')
            || key.chars().any(char::is_control)
        {
            bail!("[index] date_field {key:?}: a front matter key (not empty, no `:`, no surrounding space)");
        }
        Ok(DateSource::Field(key.to_string()))
    }

    /// The source's description in `status` and errors.
    pub fn describe(&self) -> String {
        match self {
            DateSource::Path(p, _) => format!("date_from_path = {p:?}"),
            DateSource::Field(k) => format!("date_field = {k:?}"),
        }
    }

    /// The date of a document, `None` when the source gives none (no match,
    /// no key, not a valid date).
    pub fn date_of(&self, doc_path: &str, frontmatter: Option<&str>) -> Option<Date> {
        match self {
            DateSource::Path(_, parts) => path_date(parts, doc_path),
            DateSource::Field(k) => frontmatter.and_then(|fm| field_date(k, fm)),
        }
    }
}

/// The pattern matched against the path: the match starts at the start of
/// a path component (the path's start or after a `/`), `{yyyy}` is four
/// digits, `{mm}` and `{dd}` two; the match is followed by the end, `/`,
/// `.` or a character that is not a digit. The leftmost match is the date.
fn path_date(parts: &[Part], doc_path: &str) -> Option<Date> {
    let starts = std::iter::once(0).chain(doc_path.match_indices('/').map(|(i, _)| i + 1));
    for s in starts {
        if let Some(d) = match_at(parts, &doc_path[s..]) {
            return Some(d);
        }
    }
    None
}

fn match_at(parts: &[Part], text: &str) -> Option<Date> {
    let (mut y, mut m, mut d) = (None, None, None);
    let mut pos = 0;
    for p in parts {
        let rest = &text[pos..];
        match p {
            Part::Lit(l) => {
                if !rest.starts_with(l.as_str()) {
                    return None;
                }
                pos += l.len();
            }
            Part::Year | Part::Month | Part::Day => {
                let n = if matches!(p, Part::Year) { 4 } else { 2 };
                let digits = rest.get(..n)?;
                if !digits.bytes().all(|c| c.is_ascii_digit()) {
                    return None;
                }
                let v: u16 = digits.parse().ok()?;
                match p {
                    Part::Year => y = Some(v),
                    Part::Month => m = Some(v as u8),
                    _ => d = Some(v as u8),
                }
                pos += n;
            }
        }
    }
    if text[pos..]
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
    {
        return None;
    }
    Date::new(y?, m?, d?)
}

/// The top-level front matter line `<key>: <value>` (YAML's plain form):
/// the value without quotes; its first ten characters must be a date
/// `YYYY-MM-DD` (a time may follow: `2026-10-25T09:00`, `2026-10-25 09:00`).
fn field_date(key: &str, fm: &str) -> Option<Date> {
    for line in fm.lines() {
        let line = line.trim_end_matches('\r');
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        let Some(v) = rest.strip_prefix(':') else {
            continue;
        };
        let v = v.trim();
        let v = v
            .strip_prefix('"')
            .and_then(|x| x.strip_suffix('"'))
            .or_else(|| v.strip_prefix('\'').and_then(|x| x.strip_suffix('\'')))
            .unwrap_or(v);
        let head = v.get(..10)?;
        let tail_ok = v[10..].is_empty() || v[10..].starts_with(['T', ' ']);
        return if tail_ok { Date::parse(head) } else { None };
    }
    None
}

/// The time filter of one search: the inclusive range and the documents of
/// the checkpoint whose date lies in it (computed by the caller from the
/// checkpoint's documents); `undated` counts the documents without a date
/// (left out while the filter is on).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateFilter {
    pub from: Option<Date>,
    pub to: Option<Date>,
    pub docs: BTreeSet<String>,
    pub undated: usize,
}

impl DateFilter {
    /// The filter over `docs` (`(doc_path, front matter)` of the checkpoint).
    pub fn build<'a, I>(
        source: &DateSource,
        from: Option<Date>,
        to: Option<Date>,
        docs: I,
    ) -> DateFilter
    where
        I: IntoIterator<Item = (&'a str, Option<&'a str>)>,
    {
        let mut keep = BTreeSet::new();
        let mut undated = 0;
        for (p, fm) in docs {
            match source.date_of(p, fm) {
                None => undated += 1,
                Some(d) => {
                    if from.is_none_or(|f| d >= f) && to.is_none_or(|t| d <= t) {
                        keep.insert(p.to_string());
                    }
                }
            }
        }
        DateFilter {
            from,
            to,
            docs: keep,
            undated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_patterns() {
        let s = DateSource::path("{yyyy}/{mm}/{dd}").unwrap();
        assert_eq!(s.date_of("2026/10/25.md", None), Date::new(2026, 10, 25));
        assert_eq!(
            s.date_of("journal/2026/10/25-notes.md", None),
            Date::new(2026, 10, 25)
        );
        assert_eq!(s.date_of("x2026/10/25.md", None), None);
        assert_eq!(s.date_of("2026/02/30.md", None), None);
        assert_eq!(s.date_of("2026/10/250.md", None), None);
        let s = DateSource::path("daily/{yyyy}-{mm}-{dd}").unwrap();
        assert_eq!(
            s.date_of("daily/2024-02-29.md", None),
            Date::new(2024, 2, 29)
        );
        assert_eq!(s.date_of("other/2024-02-29.md", None), None);
        for bad in [
            "",
            "/{yyyy}{mm}{dd}",
            "{yyyy}/{mm}",
            "{yyyy}{mm}{dd}{dd}",
            "{yy}/{mm}/{dd}",
            "{yyyy/{mm}/{dd}",
            "a}{yyyy}{mm}{dd}",
        ] {
            let e = DateSource::path(bad).unwrap_err();
            assert!(format!("{e:#}").contains("date_from_path"), "{bad}: {e:#}");
        }
    }

    #[test]
    fn front_matter_field() {
        let s = DateSource::field("date").unwrap();
        assert_eq!(
            s.date_of("a.md", Some("title: x\ndate: 2026-10-25\n")),
            Date::new(2026, 10, 25)
        );
        assert_eq!(
            s.date_of("a.md", Some("date: \"2026-10-25T09:00\"\n")),
            Date::new(2026, 10, 25)
        );
        assert_eq!(s.date_of("a.md", Some("dates: 2026-10-25\n")), None);
        assert_eq!(s.date_of("a.md", Some("  date: 2026-10-25\n")), None);
        assert_eq!(s.date_of("a.md", Some("date: 25.10.2026\n")), None);
        assert_eq!(s.date_of("a.md", None), None);
        assert!(DateSource::field("").is_err());
        assert!(DateSource::field("a:b").is_err());
    }

    #[test]
    fn dates() {
        assert_eq!(Date::parse("2026-10-25"), Date::new(2026, 10, 25));
        for bad in [
            "2026-1-25",
            "2026-13-01",
            "2026-10-32",
            "26-10-25",
            "2026/10/25",
            "2026-10-25 ",
        ] {
            assert_eq!(Date::parse(bad), None, "{bad}");
        }
    }
}
