use chrono::{DateTime, Utc};

#[cfg(test)]
use chrono::Duration;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::error::Result;
use crate::frontmatter;
use crate::model::Frontmatter;
use crate::storage;

struct BlueprintInfo {
    handle: String,
    description: Option<String>,
    saved_at: Option<DateTime<Utc>>,
}

pub async fn run(global: bool, sort_by_time: bool, filter: Option<&str>) -> Result<()> {
    let dir = storage::storage_dir(global)?;
    let handles = storage::list_handles(&dir).await?;

    if handles.is_empty() {
        let scope = if global { "global" } else { "project" };
        println!("No blueprints found ({scope} scope).");
        println!("Storage path: {}", dir.display());
        return Ok(());
    }

    // Load all blueprint info
    let mut blueprints: Vec<BlueprintInfo> = Vec::new();
    for handle in &handles {
        let info = storage::read_blueprint(&dir, handle)
            .await
            .ok()
            .and_then(|content| {
                let (yaml_opt, _) = frontmatter::split(&content);
                yaml_opt.and_then(|yaml| {
                    frontmatter::parse(yaml)
                        .ok()
                        .map(|fm: Frontmatter| BlueprintInfo {
                            handle: handle.clone(),
                            description: fm.description,
                            saved_at: fm.saved_at,
                        })
                })
            })
            .unwrap_or_else(|| BlueprintInfo {
                handle: handle.clone(),
                description: None,
                saved_at: None,
            });
        blueprints.push(info);
    }

    // Apply fuzzy filter if provided
    if let Some(pattern) = filter {
        let matcher = SkimMatcherV2::default();
        blueprints.retain(|bp| {
            let handle_match = matcher.fuzzy_match(&bp.handle, pattern).is_some();
            let desc_match = bp
                .description
                .as_ref()
                .map(|d| matcher.fuzzy_match(d, pattern).is_some())
                .unwrap_or(false);
            handle_match || desc_match
        });

        if blueprints.is_empty() {
            println!("No blueprints matching '{}'.", pattern);
            return Ok(());
        }
    }

    // Sort if requested
    if sort_by_time {
        blueprints.sort_by(|a, b| {
            // Sort by saved_at descending (newest first), with None values last
            match (b.saved_at, a.saved_at) {
                (Some(b_time), Some(a_time)) => b_time.cmp(&a_time),
                (Some(_), None) => std::cmp::Ordering::Greater, // b has time, a doesn't: b comes first
                (None, Some(_)) => std::cmp::Ordering::Less, // b has no time, a does: b comes last
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
    }

    // Calculate column widths
    let max_handle_width = blueprints.iter().map(|b| b.handle.len()).max().unwrap_or(0);
    let now = Utc::now();

    // Calculate max time width for alignment
    let max_time_width = blueprints
        .iter()
        .map(|b| b.saved_at.map(|t| format_ago(&now, &t).len()).unwrap_or(0))
        .max()
        .unwrap_or(0);

    // Print header
    let time_header = if max_time_width > 0 {
        let header = "WHEN";
        format!(
            "{:<width$}",
            header,
            width = max_time_width.max(header.len())
        )
    } else {
        String::new()
    };

    if !time_header.is_empty() {
        println!(
            "  {:<handle_width$}  {}  DESCRIPTION",
            "HANDLE",
            time_header,
            handle_width = max_handle_width
        );
        println!("  {}", "─".repeat(max_handle_width + max_time_width + 30));
    }

    // Print blueprints
    for bp in &blueprints {
        let time_str = bp
            .saved_at
            .map(|t| format_ago(&now, &t))
            .unwrap_or_default();

        let desc_str = bp.description.as_deref().unwrap_or("(no description)");

        if max_time_width > 0 {
            println!(
                "  {:<handle_width$}  {:<time_width$}  {}",
                bp.handle,
                time_str,
                desc_str,
                handle_width = max_handle_width,
                time_width = max_time_width.max(4)
            );
        } else {
            println!(
                "  {:<handle_width$}  {}",
                bp.handle,
                desc_str,
                handle_width = max_handle_width
            );
        }
    }

    Ok(())
}

/// Format a duration as "how long ago"
/// - < 60 mins: "5m"
/// - < 24 hours: "2h 30m"
/// - >= 24 hours: "3d 2h"
fn format_ago(now: &DateTime<Utc>, then: &DateTime<Utc>) -> String {
    let duration = now.signed_duration_since(*then);
    let total_minutes = duration.num_minutes();

    if total_minutes < 60 {
        // Less than 1 hour: show minutes
        format!("{}m", total_minutes.max(0))
    } else if total_minutes < 24 * 60 {
        // Less than 24 hours: show hours and minutes
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;
        if minutes > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}h", hours)
        }
    } else {
        // 24 hours or more: show days and hours
        let days = total_minutes / (24 * 60);
        let hours = (total_minutes % (24 * 60)) / 60;
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ago_minutes() {
        let now = Utc::now();
        let then = now - Duration::minutes(5);
        assert_eq!(format_ago(&now, &then), "5m");
    }

    #[test]
    fn format_ago_hours_and_minutes() {
        let now = Utc::now();
        let then = now - Duration::minutes(150); // 2h 30m
        assert_eq!(format_ago(&now, &then), "2h 30m");
    }

    #[test]
    fn format_ago_exact_hour() {
        let now = Utc::now();
        let then = now - Duration::minutes(120); // 2h exactly
        assert_eq!(format_ago(&now, &then), "2h");
    }

    #[test]
    fn format_ago_days_and_hours() {
        let now = Utc::now();
        let then = now - Duration::minutes(2 * 24 * 60 + 5 * 60); // 2d 5h
        assert_eq!(format_ago(&now, &then), "2d 5h");
    }

    #[test]
    fn format_ago_exact_day() {
        let now = Utc::now();
        let then = now - Duration::minutes(3 * 24 * 60); // 3d exactly
        assert_eq!(format_ago(&now, &then), "3d");
    }

    #[test]
    fn format_ago_zero() {
        let now = Utc::now();
        assert_eq!(format_ago(&now, &now), "0m");
    }

    #[test]
    fn fuzzy_filter_matches_handle() {
        let matcher = SkimMatcherV2::default();
        let blueprint = BlueprintInfo {
            handle: "auth-refactor".to_string(),
            description: Some("Refactor auth".to_string()),
            saved_at: None,
        };

        let handle_match = matcher.fuzzy_match(&blueprint.handle, "auth").is_some();
        let desc_match = blueprint
            .description
            .as_ref()
            .map(|d| matcher.fuzzy_match(d, "auth").is_some())
            .unwrap_or(false);

        assert!(handle_match || desc_match);
    }

    #[test]
    fn fuzzy_filter_matches_description() {
        let matcher = SkimMatcherV2::default();
        let blueprint = BlueprintInfo {
            handle: "plan1".to_string(),
            description: Some("Fix login bug".to_string()),
            saved_at: None,
        };

        let handle_match = matcher.fuzzy_match(&blueprint.handle, "login").is_some();
        let desc_match = blueprint
            .description
            .as_ref()
            .map(|d| matcher.fuzzy_match(d, "login").is_some())
            .unwrap_or(false);

        assert!(!handle_match); // handle doesn't match
        assert!(desc_match); // description matches
    }

    #[test]
    fn fuzzy_filter_no_match() {
        let matcher = SkimMatcherV2::default();
        let blueprint = BlueprintInfo {
            handle: "auth-refactor".to_string(),
            description: Some("Refactor auth".to_string()),
            saved_at: None,
        };

        let handle_match = matcher.fuzzy_match(&blueprint.handle, "xyz").is_some();
        let desc_match = blueprint
            .description
            .as_ref()
            .map(|d| matcher.fuzzy_match(d, "xyz").is_some())
            .unwrap_or(false);

        assert!(!handle_match);
        assert!(!desc_match);
    }

    #[test]
    fn fuzzy_filter_fuzzy_matching() {
        let matcher = SkimMatcherV2::default();
        let blueprint = BlueprintInfo {
            handle: "auth-refactor".to_string(),
            description: None,
            saved_at: None,
        };

        // Should match with fuzzy logic (ar matches auth-refactor)
        let handle_match = matcher.fuzzy_match(&blueprint.handle, "ar").is_some();
        assert!(handle_match);
    }

    #[test]
    fn sort_by_time_newest_first() {
        let now = Utc::now();
        let mut blueprints = vec![
            BlueprintInfo {
                handle: "oldest".to_string(),
                description: None,
                saved_at: Some(now - Duration::hours(5)),
            },
            BlueprintInfo {
                handle: "newest".to_string(),
                description: None,
                saved_at: Some(now - Duration::minutes(5)),
            },
            BlueprintInfo {
                handle: "middle".to_string(),
                description: None,
                saved_at: Some(now - Duration::hours(2)),
            },
        ];

        // Sort by saved_at descending (newest first)
        blueprints.sort_by(|a, b| match (b.saved_at, a.saved_at) {
            (Some(b_time), Some(a_time)) => b_time.cmp(&a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        assert_eq!(blueprints[0].handle, "newest");
        assert_eq!(blueprints[1].handle, "middle");
        assert_eq!(blueprints[2].handle, "oldest");
    }

    #[test]
    fn sort_by_time_with_none_last() {
        let now = Utc::now();
        let mut blueprints = vec![
            BlueprintInfo {
                handle: "with_time".to_string(),
                description: None,
                saved_at: Some(now - Duration::hours(5)),
            },
            BlueprintInfo {
                handle: "no_time".to_string(),
                description: None,
                saved_at: None,
            },
        ];

        // Sort by saved_at descending (newest first), None values last
        blueprints.sort_by(|a, b| {
            match (b.saved_at, a.saved_at) {
                (Some(b_time), Some(a_time)) => b_time.cmp(&a_time),
                (Some(_), None) => std::cmp::Ordering::Greater, // b has time, a doesn't: b comes first
                (None, Some(_)) => std::cmp::Ordering::Less, // b has no time, a does: b comes last
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        // with_time should come first (has time), no_time second (no time)
        assert_eq!(blueprints[0].handle, "with_time");
        assert_eq!(blueprints[1].handle, "no_time");
    }
}
