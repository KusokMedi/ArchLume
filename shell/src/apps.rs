//! Discover installed applications from XDG `.desktop` entries.
//!
//! Deliberately minimal and dependency-free: it reads `applications/*.desktop`
//! from the XDG data directories and extracts the display name and command of
//! entries that should appear in a launcher. Localized `Name[xx]` keys are
//! ignored for now (English-first); key parsing stops at the `[Desktop Entry]`
//! group.

use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

/// One launcher entry: a display name and the command to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
}

impl AppEntry {
    /// Whether the entry matches a case-insensitive search query. An empty
    /// query matches everything.
    pub fn matches(&self, query: &str) -> bool {
        query.is_empty() || self.name.to_lowercase().contains(&query.to_lowercase())
    }
}

/// Returns the application entries, sorted by name, de-duplicated by name.
pub fn discover() -> Vec<AppEntry> {
    let mut entries = BTreeMap::new();
    for dir in data_dirs() {
        let apps = dir.join("applications");
        collect_from_dir(&apps, &mut entries);
    }
    entries.into_values().collect()
}

fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = env::var_os("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(home));
    } else if let Some(home) = env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share"));
    }
    match env::var_os("XDG_DATA_DIRS") {
        Some(dirs_var) => dirs.extend(env::split_paths(&dirs_var)),
        None => {
            dirs.push(PathBuf::from("/usr/local/share"));
            dirs.push(PathBuf::from("/usr/share"));
        }
    }
    dirs
}

fn collect_from_dir(dir: &Path, entries: &mut BTreeMap<String, AppEntry>) {
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            if let Some(app) = parse_desktop(&content) {
                entries.entry(app.name.clone()).or_insert(app);
            }
        }
    }
}

/// Parses the `Name` and `Exec` of a `[Desktop Entry]` that should be shown in
/// a launcher. Returns `None` for non-applications, hidden entries, or entries
/// missing a name or command.
fn parse_desktop(content: &str) -> Option<AppEntry> {
    let mut in_entry = false;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut hidden = false;
    let mut is_application = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "Name" => name = Some(value.trim().to_string()),
            "Exec" => exec = Some(value.trim().to_string()),
            "NoDisplay" => hidden = value.trim().eq_ignore_ascii_case("true"),
            "Type" => is_application = value.trim() == "Application",
            _ => {}
        }
    }

    if hidden || !is_application {
        return None;
    }
    let name = name.filter(|n| !n.is_empty())?;
    let exec = exec.filter(|e| !e.is_empty())?;
    Some(AppEntry { name, exec })
}

/// Strips `%`-field codes (e.g. `%U`, `%f`) from an `Exec` command line.
pub fn strip_field_codes(exec: &str) -> String {
    let mut out = String::with_capacity(exec.len());
    let mut chars = exec.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let _ = chars.next();
        } else {
            out.push(c);
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_visible_application() {
        let entry = "[Desktop Entry]\nType=Application\nName=Firefox\nExec=firefox %u\n";
        assert_eq!(
            parse_desktop(entry),
            Some(AppEntry {
                name: "Firefox".into(),
                exec: "firefox %u".into(),
            })
        );
    }

    #[test]
    fn skips_hidden_and_non_applications() {
        let hidden = "[Desktop Entry]\nType=Application\nName=Secret\nExec=x\nNoDisplay=true\n";
        assert_eq!(parse_desktop(hidden), None);
        let link = "[Desktop Entry]\nType=Link\nName=Homepage\nExec=x\n";
        assert_eq!(parse_desktop(link), None);
        let no_exec = "[Desktop Entry]\nType=Application\nName=NoCommand\n";
        assert_eq!(parse_desktop(no_exec), None);
    }

    #[test]
    fn ignores_other_groups_and_localized_names() {
        let entry = "\
[Desktop Entry]\n\
Type=Application\n\
Name[de]=Datei\n\
Name=Files\n\
Exec=files\n\
[Desktop Action new]\n\
Name=New Window\n";
        assert_eq!(
            parse_desktop(entry),
            Some(AppEntry {
                name: "Files".into(),
                exec: "files".into(),
            })
        );
    }

    #[test]
    fn strips_field_codes() {
        assert_eq!(strip_field_codes("firefox %u"), "firefox");
        assert_eq!(strip_field_codes("app --flag %F %i"), "app --flag");
    }

    #[test]
    fn matches_is_case_insensitive() {
        let app = AppEntry {
            name: "Firefox".into(),
            exec: "firefox".into(),
        };
        assert!(app.matches(""));
        assert!(app.matches("FOX"));
        assert!(!app.matches("chrome"));
    }
}
