//! Read battery state from sysfs.
//!
//! Dependency-free: it reads the standard `/sys/class/power_supply` entries.
//! The directory can be overridden with `ARCHLUME_BATTERY_DIR`, which is used by
//! automated tests to feed deterministic values. On machines without a battery
//! the panel simply omits the widget.

use std::{fs, path::PathBuf};

/// A battery reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Battery {
    /// Charge in percent (0-100).
    pub capacity: u8,
    /// Whether the battery is currently charging.
    pub charging: bool,
}

impl Battery {
    /// The panel label, e.g. `BAT 42%`. Kept a fixed width (two digits) so the
    /// widget does not reflow the panel as the value changes.
    pub fn label(&self) -> String {
        format!("BAT {:>2}%", self.capacity)
    }
}

/// Reads the first battery found, or `None` when there is no battery.
pub fn read() -> Option<Battery> {
    read_from(&supply_dir())
}

fn supply_dir() -> PathBuf {
    match std::env::var_os("ARCHLUME_BATTERY_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from("/sys/class/power_supply"),
    }
}

fn read_from(dir: &std::path::Path) -> Option<Battery> {
    let mut entries: Vec<_> = fs::read_dir(dir).ok()?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let Ok(text) = fs::read_to_string(path.join("capacity")) else {
            continue;
        };
        let Ok(capacity) = text.trim().parse::<u8>() else {
            continue;
        };
        if capacity > 100 {
            continue;
        }
        let status = fs::read_to_string(path.join("status")).unwrap_or_default();
        return Some(Battery {
            capacity,
            charging: status.trim().eq_ignore_ascii_case("charging"),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("archlume-battery-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("BAT0")).unwrap();
        dir
    }

    #[test]
    fn reads_capacity_and_status() {
        let dir = temp_dir("read");
        fs::write(dir.join("BAT0/capacity"), "73\n").unwrap();
        fs::write(dir.join("BAT0/status"), "Charging\n").unwrap();
        assert_eq!(
            read_from(&dir),
            Some(Battery {
                capacity: 73,
                charging: true,
            })
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn returns_none_without_battery() {
        let dir = temp_dir("empty");
        fs::create_dir_all(dir.join("AC")).unwrap();
        fs::write(dir.join("AC/capacity"), "not-a-number").unwrap();
        assert_eq!(read_from(&dir), None);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn label_is_fixed_width() {
        assert_eq!(
            Battery {
                capacity: 7,
                charging: false
            }
            .label(),
            "BAT  7%"
        );
        assert_eq!(
            Battery {
                capacity: 42,
                charging: false
            }
            .label(),
            "BAT 42%"
        );
    }
}
