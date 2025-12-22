//! Notification Rule Engine - complex filtering rules for Power Mode.
//!
//! Provides account-based and type-based notification filtering
//! with priority organization support.

use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

// ============================================================================
// RULE ACTIONS
// ============================================================================

/// Actions that rules can take on matching notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RuleAction {
    /// **Show**: Standard behavior. The notification appears in the list and triggers a system desktop notification.
    #[default]
    Show,
    /// **Silent**: The notification appears in the list but **does not** trigger a desktop notification or sound.
    Silent,
    /// **Hide**: The notification is completely hidden from the list.
    Hide,
    /// **Priority**: The notification is prominently highlighted and always triggers a desktop notification, overriding other rules.
    Priority,
}

impl RuleAction {
    pub const ALL: &'static [Self] = &[Self::Show, Self::Silent, Self::Hide, Self::Priority];

    /// User-facing display label for the action (used in rule cards).
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::Show => "Show",
            Self::Silent => "Silent",
            Self::Hide => "Suppress",
            Self::Priority => "Priority",
        }
    }
}

impl std::fmt::Display for RuleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Show => write!(f, "Show"),
            Self::Silent => write!(f, "Silent"),
            Self::Hide => write!(f, "Suppress"),
            Self::Priority => write!(f, "Priority"),
        }
    }
}

// ============================================================================
// PRIORITY LEVELS
// ============================================================================

pub const PRIORITY_MAX: i32 = 100;
pub const PRIORITY_HIGH: i32 = 50;
pub const PRIORITY_DEFAULT: i32 = 0;
pub const PRIORITY_LOW: i32 = -50;
pub const PRIORITY_MIN: i32 = -100;

#[allow(dead_code)]
pub const PRIORITY_LEVELS: &[(&str, i32)] = &[
    ("Max (100)", PRIORITY_MAX),
    ("High (50)", PRIORITY_HIGH),
    ("Default (0)", PRIORITY_DEFAULT),
    ("Low (-50)", PRIORITY_LOW),
    ("Min (-100)", PRIORITY_MIN),
];

fn default_priority() -> i32 {
    PRIORITY_DEFAULT
}

// ============================================================================
// OVERVIEW HELPERS
// ============================================================================

/// High-impact rule info for Overview display.
#[derive(Debug, Clone)]
pub struct HighImpactRule {
    pub name: String,
    pub rule_type: String,
    pub action: RuleAction,
    pub priority: i32,
}

// ============================================================================
// RULE TYPES
// ============================================================================

/// Behavior when outside of the active schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutsideScheduleBehavior {
    /// **Suppress**: Hide notifications completely (Action::Hide).
    #[default]
    Suppress,
    /// **Defer**: Do not notify, but keep in list (Action::Silent).
    Defer,
}

impl std::fmt::Display for OutsideScheduleBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Suppress => write!(f, "Suppress notifications"),
            Self::Defer => write!(f, "Defer until next active window"),
        }
    }
}

/// Per-account notification schedule.
/// Controls when notifications from this account are shown vs suppressed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRule {
    pub id: String,
    /// Master switch for this account's schedule enforcement.
    /// If false, account is treated as "Not Active" (suppressed/hidden).
    pub enabled: bool,
    /// GitHub username.
    pub account: String,
    /// Days when notifications from this account are ACTIVE (shown).
    /// Contains days 0-6 (Sun=0, Sat=6). Empty = show all days (if enabled).
    #[serde(default = "default_active_days")]
    pub active_days: Vec<u8>,
    /// Optional: Start time when notifications are shown (e.g., "09:00").
    #[serde(default)]
    pub start_time: Option<String>,
    /// Optional: End time when notifications are shown (e.g., "18:00").
    #[serde(default)]
    pub end_time: Option<String>,
    /// Behavior when outside active schedule.
    #[serde(default)]
    pub outside_behavior: OutsideScheduleBehavior,
}

fn default_active_days() -> Vec<u8> {
    vec![0, 1, 2, 3, 4, 5, 6] // All days active by default
}

impl AccountRule {
    #[allow(dead_code)]
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            enabled: true,
            account: account.into(),
            active_days: default_active_days(),
            start_time: None,
            end_time: None,
            outside_behavior: OutsideScheduleBehavior::Suppress,
        }
    }

    /// Check if the account is currently in active schedule.
    pub fn is_active_now(&self) -> bool {
        // Master Kill Switch: If account is disabled, it is NOT active.
        if !self.enabled {
            return false;
        }

        use chrono::{Datelike, Local, Timelike};
        let now = Local::now();
        let day = now.weekday().num_days_from_sunday() as u8;

        // Check active days
        if !self.active_days.contains(&day) {
            return false;
        }

        // Check time range if specified
        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if let (Ok(start_h), Ok(end_h)) = (
                start.split(':').next().unwrap_or("0").parse::<u32>(),
                end.split(':').next().unwrap_or("23").parse::<u32>(),
            ) {
                let hour = now.hour();
                // Simple hour check for now
                if start_h <= end_h {
                    if hour < start_h || hour >= end_h {
                        return false;
                    }
                } else {
                    // Crossing midnight (e.g. 22:00 to 07:00)
                    // Active if >= 22 OR < 7
                    if hour < start_h && hour >= end_h {
                        return false;
                    }
                }
            }
        }

        true
    }
}

/// Organization-level priority and filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgRule {
    pub id: String,
    pub enabled: bool,
    /// GitHub organization name.
    pub org: String,
    /// Priority level (higher = more important).
    pub priority: i32,
    pub action: RuleAction,
}

impl OrgRule {
    #[allow(dead_code)]
    pub fn new(org: impl Into<String>, priority: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            enabled: true,
            org: org.into(),
            priority,
            action: RuleAction::Show,
        }
    }
}

/// Notification type suppression rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRule {
    pub id: String,
    pub enabled: bool,
    /// Notification reason type (mention, review_requested, etc.).
    pub notification_type: String,
    /// Optional GitHub username to scope this rule to.
    #[serde(default)]
    pub account: Option<String>,
    /// Priority level (higher = more important).
    #[serde(default)]
    pub priority: i32,
    pub action: RuleAction,
}

impl TypeRule {
    pub fn new(
        notification_type: impl Into<String>,
        account: Option<String>,
        priority: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            enabled: true,
            notification_type: notification_type.into(),
            account,
            priority,
            action: RuleAction::Show,
        }
    }
}

// ============================================================================
// RULE SET (ROOT CONTAINER)
// ============================================================================

/// Complete notification rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationRuleSet {
    /// Rule set name for organization (e.g., "Work", "On-Call", "Weekend").
    #[serde(default = "default_rule_set_name")]
    pub name: String,
    /// Global enable/disable for all rules.
    pub enabled: bool,
    /// Per-account filtering.
    pub account_rules: Vec<AccountRule>,
    /// Organization priority rules.
    pub org_rules: Vec<OrgRule>,
    /// Notification type filtering.
    pub type_rules: Vec<TypeRule>,
}

fn default_rule_set_name() -> String {
    "Default".to_string()
}

impl NotificationRuleSet {
    /// Get the rules file path.
    fn rules_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("gittop").join("rules.json"))
    }

    /// Load rules from disk, or return defaults.
    pub fn load() -> Self {
        Self::rules_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Save rules to disk.
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::rules_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }

    /// Count total active rules.
    pub fn active_rule_count(&self) -> usize {
        if !self.enabled {
            return 0;
        }
        self.account_rules.iter().filter(|r| r.enabled).count()
            + self.org_rules.iter().filter(|r| r.enabled).count()
            + self.type_rules.iter().filter(|r| r.enabled).count()
    }

    // ========================================================================
    // Overview Stats Helpers
    // ========================================================================

    /// Count rules with Hide (suppress) action.
    pub fn count_suppress_rules(&self) -> usize {
        if !self.enabled {
            return 0;
        }
        let mut count = 0;
        // Account Rules contribute suppression when they are effectively active (enforced) and outside schedule
        count += self
            .account_rules
            .iter()
            .filter(|r| {
                r.enabled
                    && !r.is_active_now()
                    && r.outside_behavior == OutsideScheduleBehavior::Suppress
            })
            .count();
        count += self
            .org_rules
            .iter()
            .filter(|r| r.enabled && r.action == RuleAction::Hide)
            .count();
        count += self
            .type_rules
            .iter()
            .filter(|r| r.enabled && r.action == RuleAction::Hide)
            .count();
        count
    }

    /// Count rules with priority >= 50.
    pub fn count_high_priority_rules(&self) -> usize {
        if !self.enabled {
            return 0;
        }
        let mut count = 0;
        count += self
            .org_rules
            .iter()
            .filter(|r| {
                r.enabled && (r.priority >= PRIORITY_HIGH || r.action == RuleAction::Priority)
            })
            .count();
        count += self
            .type_rules
            .iter()
            .filter(|r| {
                r.enabled && (r.priority >= PRIORITY_HIGH || r.action == RuleAction::Priority)
            })
            .count();
        count
    }

    /// Collected high-impact rule info for Overview display.
    pub fn get_high_impact_rules(&self) -> Vec<HighImpactRule> {
        if !self.enabled {
            return Vec::new();
        }

        let mut rules = Vec::new();

        // Account rules that are currently suppressing (outside schedule)
        for rule in &self.account_rules {
            if rule.enabled && !rule.is_active_now() {
                let action = match rule.outside_behavior {
                    OutsideScheduleBehavior::Suppress => RuleAction::Hide,
                    OutsideScheduleBehavior::Defer => RuleAction::Silent,
                };
                rules.push(HighImpactRule {
                    name: rule.account.clone(),
                    rule_type: "Account Schedule".to_string(),
                    action,
                    priority: 0,
                });
            }
        }

        // Org rules with Hide or high priority
        for rule in &self.org_rules {
            if rule.enabled
                && (rule.action == RuleAction::Hide
                    || rule.action == RuleAction::Priority
                    || rule.priority >= PRIORITY_HIGH)
            {
                rules.push(HighImpactRule {
                    name: rule.org.clone(),
                    rule_type: "Org".to_string(),
                    action: rule.action,
                    priority: rule.priority,
                });
            }
        }

        // Type rules with Hide or high priority
        for rule in &self.type_rules {
            if rule.enabled
                && (rule.action == RuleAction::Hide
                    || rule.action == RuleAction::Priority
                    || rule.priority >= PRIORITY_HIGH)
            {
                let name = if let Some(acc) = &rule.account {
                    format!("{} ({})", rule.notification_type, acc)
                } else {
                    format!("{} (Global)", rule.notification_type)
                };
                rules.push(HighImpactRule {
                    name,
                    rule_type: "Type".to_string(),
                    action: rule.action,
                    priority: rule.priority,
                });
            }
        }

        rules
    }
}

// ============================================================================
// RULE ENGINE (EVALUATION)
// ============================================================================

/// Rule engine for evaluating notifications against the rule set.
pub struct RuleEngine {
    rules: NotificationRuleSet,
}

impl RuleEngine {
    pub fn new(rules: NotificationRuleSet) -> Self {
        Self { rules }
    }

    /// Evaluate a notification and return the action to take.
    ///
    /// Priority order:
    /// 1. Priority rules always win (highest priority value wins among them)
    /// 2. Hide rules evaluated next
    /// 3. Silent rules
    /// 4. Default to Show
    #[allow(dead_code)]
    pub fn evaluate(
        &self,
        notification_type: &str,
        repo_owner: Option<&str>,
        account: Option<&str>,
        now: &chrono::DateTime<chrono::Local>,
    ) -> RuleAction {
        self.evaluate_detailed(notification_type, repo_owner, account, now)
            .0
    }

    /// Evaluate with full trace of the decision.
    pub fn evaluate_detailed(
        &self,
        notification_type: &str,
        repo_owner: Option<&str>,
        account: Option<&str>,
        now: &chrono::DateTime<chrono::Local>,
    ) -> (RuleAction, Option<RuleDecision>) {
        if !self.rules.enabled {
            return (RuleAction::Show, None);
        }

        let mut matching_actions: Vec<(i32, RuleAction, String, RuleDecisionReason)> = Vec::new();

        // 1. Account Rules (Schedule Gating)
        // This is the primary gate.
        if let Some(acc) = account {
            for rule in &self.rules.account_rules {
                if rule.account.eq_ignore_ascii_case(acc) {
                    // if rule is NOT enabled, it is inactive (Suppress/Hide)
                    if !rule.enabled {
                        matching_actions.push((
                            PRIORITY_DEFAULT,
                            RuleAction::Hide, // Disabled account = Suppress
                            rule.id.clone(),
                            RuleDecisionReason::AccountRule(format!("{} (Disabled)", rule.account)),
                        ));
                        continue;
                    }

                    // Rule is enabled, check schedule
                    if rule.is_active_now() {
                        // Account is active, show.
                        matching_actions.push((
                            PRIORITY_DEFAULT,
                            RuleAction::Show,
                            rule.id.clone(),
                            RuleDecisionReason::AccountRule(rule.account.clone()),
                        ));
                    } else {
                        // Account is inactive (outside schedule). Apply configured behavior.
                        let action = match rule.outside_behavior {
                            OutsideScheduleBehavior::Suppress => RuleAction::Hide,
                            OutsideScheduleBehavior::Defer => RuleAction::Silent,
                        };
                        matching_actions.push((
                            PRIORITY_DEFAULT,
                            action,
                            rule.id.clone(),
                            RuleDecisionReason::AccountRule(rule.account.clone()),
                        ));
                    }
                }
            }
        }

        // 2. Org Rules
        if let Some(owner) = repo_owner {
            for rule in &self.rules.org_rules {
                if rule.enabled && rule.org.eq_ignore_ascii_case(owner) {
                    matching_actions.push((
                        rule.priority,
                        rule.action,
                        rule.id.clone(),
                        RuleDecisionReason::OrgRule(rule.org.clone()),
                    ));
                }
            }
        }

        // 3. Type Rules
        for rule in &self.rules.type_rules {
            if rule.enabled
                && rule
                    .notification_type
                    .eq_ignore_ascii_case(notification_type)
            {
                let account_match = match (&rule.account, account) {
                    (None, _) => true,
                    (Some(rule_acc), Some(notif_acc)) => rule_acc.eq_ignore_ascii_case(notif_acc),
                    (Some(_), None) => false,
                };

                if account_match {
                    matching_actions.push((
                        rule.priority,
                        rule.action,
                        rule.id.clone(),
                        RuleDecisionReason::TypeRule(rule.notification_type.clone()),
                    ));
                }
            }
        }

        if matching_actions.is_empty() {
            return (RuleAction::Show, None);
        }

        // Sort by priority descending
        matching_actions.sort_by(|a, b| b.0.cmp(&a.0));

        // Get max priority
        let max_priority = matching_actions[0].0;

        // *** PRIORITY OVERRIDE ***
        // If ANY rule has Priority action, it OVERRIDES all suppression rules.
        // This is the key feature of Priority - it ignores account schedules, time rules, etc.
        let has_priority_rule = matching_actions
            .iter()
            .any(|(_, action, _, _)| *action == RuleAction::Priority);

        if has_priority_rule {
            // Find the highest priority rule with Priority action
            for (p, action, id, reason) in matching_actions.iter() {
                if *action == RuleAction::Priority {
                    return (
                        RuleAction::Priority,
                        Some(RuleDecision {
                            applied_rule_id: id.clone(),
                            action: RuleAction::Priority,
                            priority: *p,
                            reason: reason.clone(),
                        }),
                    );
                }
            }
        }

        // Standard conflict resolution (no Priority rule found)
        // Priority > Hide > Silent > Show

        // Optimization: Find the best action within the max priority band without cloning
        let mut best_action = RuleAction::Show;
        let mut best_decision: Option<RuleDecision> = None;

        for (p, action, id, reason) in matching_actions.iter() {
            if *p != max_priority {
                break; // Because sorted descending, we can stop once priority drops
            }

            let is_better = match (best_action, action) {
                (RuleAction::Priority, _) => false, // Only another Priority could match, which is same level
                (_, RuleAction::Priority) => true,  // Priority always beats non-Priority
                (RuleAction::Hide, _) => false,     // Hide beats Silent/Show
                (_, RuleAction::Hide) => true,      // Hide beats Silent/Show
                (RuleAction::Silent, _) => false,   // Silent beats Show
                (_, RuleAction::Silent) => true,    // Silent beats Show
                (RuleAction::Show, RuleAction::Show) => false, // Same level
            };

            // First valid rule sets the baseline.
            // If best_decision is None, we take the first one.
            // If we found a strictly "better" action, we take it.
            if best_decision.is_none() || is_better {
                best_action = *action;
                best_decision = Some(RuleDecision {
                    applied_rule_id: id.clone(),
                    action: *action,
                    priority: *p,
                    reason: reason.clone(),
                });
            }
        }

        (best_action, best_decision)
    }
}

/// Trace of why a specific rule was applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleDecision {
    pub applied_rule_id: String,
    pub action: RuleAction,
    pub priority: i32,
    pub reason: RuleDecisionReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleDecisionReason {
    AccountRule(String),
    OrgRule(String),
    TypeRule(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_set_default() {
        let rules = NotificationRuleSet::default();
        assert!(!rules.enabled);
        assert_eq!(rules.active_rule_count(), 0);
    }

    #[test]
    fn test_active_schedule_logic() {
        let mut rules = NotificationRuleSet::default();
        rules.enabled = true;

        let mut acc_rule = AccountRule::new("Amar");
        acc_rule.enabled = true;
        acc_rule.outside_behavior = OutsideScheduleBehavior::Suppress;
        // Make it active only on Monday (Day 1)
        acc_rule.active_days = vec![1];
        rules.account_rules.push(acc_rule);
    }

    #[test]
    fn test_priority_action_override() {
        // High priority rule (Org) vs Schedule suppression
        let mut rules = NotificationRuleSet::default();
        rules.enabled = true;

        let mut acc_rule = AccountRule::new("WorkAcc");
        acc_rule.enabled = true;
        acc_rule.active_days = vec![]; // Never active
        acc_rule.outside_behavior = OutsideScheduleBehavior::Suppress;
        rules.account_rules.push(acc_rule);

        let org_rule = OrgRule {
            id: "org1".to_string(),
            enabled: true,
            org: "WorkOrg".to_string(),
            priority: 50,
            action: RuleAction::Priority, // Force show!
        };
        rules.org_rules.push(org_rule);

        let engine = RuleEngine::new(rules);
        let now = chrono::Local::now();

        // Account rule says Hide. Org rule says Priority.
        // Priority should win.
        let (action, _) =
            engine.evaluate_detailed("mention", Some("WorkOrg"), Some("WorkAcc"), &now);
        assert_eq!(action, RuleAction::Priority);
    }
}
