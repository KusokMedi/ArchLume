//! Discover installed applications from XDG `.desktop` entries.
//!
//! Deliberately minimal and dependency-free: it reads `applications/*.desktop`
//! from the XDG data directories and extracts the display name of entries that
//! should appear in a launcher. Localized `Name[xx]` keys are ignored for now
//! (English-first); key parsing stops at the `[Desktop Entry]` group.

use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

/// Returns the sorted, de-duplicated display names of installed applications.
pub fn discover() -> Vec<String> {
    let mut names = BTreeSet::new();
    for dir in data_dirs() {
        let apps = dir.join("applications");
        collect_from_dir(&apps, &mut names);
    }
    names.into_iter().collect()
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

fn collect_from_dir(dir: &Path, names: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            if let Some(name) = parse_desktop(&content) {
                names.insert(name);
            }
        }
    }
}

/// Extracts the `Name` of a `[Desktop Entry]` that should be shown in a
/// launcher. Returns `None` for non-applications, hidden entries, or entries
/// without a name.
fn parse_desktop(content: &str) -> Option<String> {
    let mut in_entry = false;
    let mut name: Option<String> = None;
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
            "NoDisplay" => hidden = value.trim().eq_ignore_ascii_case("true"),
            "Type" => is_application = value.trim() == "Application",
            _ => {}
        }
    }

    if hidden || !is_application {
        return None;
    }
    name.filter(|n| !n.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_visible_application() {
        let entry = "[Desktop Entry]\nType=Application\nName=Firefox\nExec=firefox %u\n";
        assert_eq!(parse_desktop(entry), Some("Firefox".to_string()));
    }

    #[test]
    fn skips_hidden_and_non_applications() {
        let hidden = "[Desktop Entry]\nType=Application\nName=Secret\nNoDisplay=true\n";
        assert_eq!(parse_desktop(hidden), None);

        let link = "[Desktop Entry]\nType=Link\nName=Homepage\n";
        assert_eq!(parse_desktop(link), None);
    }

    #[test]
    fn ignores_other_groups_and_localized_names() {
        let entry = "\
[Desktop Entry]\n\
Type=Application\n\
Name[de]=Datei\n\
Name=Files\n\
[Desktop Action new]\n\
Name=New Window\n";
        assert_eq!(parse_desktop(entry), Some("Files".to_string()));
    }
}
