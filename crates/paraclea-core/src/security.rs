//! Pre-Flight Autonomous Security Shield (Adapted from Cynapse-Mini & Strix).
//!
//! Validates prospective command executions and filesystem operations,
//! intercepting command injection, path traversal escapes, and destructive commands.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityVerdict {
    Safe,
    RequiresApproval { reason: String },
    Blocked { reason: String },
}

const PERMANENTLY_BLOCKED_COMMANDS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    ":(){ :|:& };:",
    "mkfs",
    "> /dev/sda",
    "> /dev/nvme",
    "dd if=/dev/zero",
    "dd if=/dev/urandom",
];

const APPROVAL_ESCALATION_PATTERNS: &[&str] = &[
    "| bash",
    "| sh",
    "| zsh",
    "curl",
    "wget",
    "sudo",
    "doas",
    "chmod 777",
    "chown root",
    "shutdown",
    "reboot",
];

const SENSITIVE_PATH_PATTERNS: &[&str] = &[
    "/etc/shadow",
    "/etc/sudoers",
    "id_rsa",
    "id_ed25519",
    ".ssh/authorized_keys",
    ".gnupg",
];

pub struct SecurityShield;

impl SecurityShield {
    /// Evaluates shell command string before execution.
    pub fn evaluate_command(cmd: &str) -> SecurityVerdict {
        let trimmed = cmd.trim();
        let lower = trimmed.to_lowercase();

        for &blocked in PERMANENTLY_BLOCKED_COMMANDS {
            if lower.contains(blocked) {
                return SecurityVerdict::Blocked {
                    reason: format!("Destructive command blocked: {}", blocked),
                };
            }
        }

        for &approval in APPROVAL_ESCALATION_PATTERNS {
            if lower.contains(approval) {
                return SecurityVerdict::RequiresApproval {
                    reason: format!("Escalated privilege or network execution requires confirmation: {}", approval),
                };
            }
        }

        SecurityVerdict::Safe
    }

    /// Evaluates target file path for path traversal or sensitive file reads.
    pub fn evaluate_path(path: &str) -> SecurityVerdict {
        let lower = path.to_lowercase();
        if path.contains("..") {
            return SecurityVerdict::Blocked {
                reason: "Directory traversal '..' is prohibited".to_string(),
            };
        }

        for &sensitive in SENSITIVE_PATH_PATTERNS {
            if lower.contains(sensitive) {
                return SecurityVerdict::Blocked {
                    reason: format!("Access to sensitive system credential path is blocked: {}", sensitive),
                };
            }
        }

        SecurityVerdict::Safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_shield_evaluations() {
        assert_eq!(SecurityShield::evaluate_command("cargo test"), SecurityVerdict::Safe);
        assert!(matches!(
            SecurityShield::evaluate_command("rm -rf / --no-preserve-root"),
            SecurityVerdict::Blocked { .. }
        ));
        assert!(matches!(
            SecurityShield::evaluate_command("sudo systemctl restart"),
            SecurityVerdict::RequiresApproval { .. }
        ));
        assert!(matches!(
            SecurityShield::evaluate_path("../../etc/passwd"),
            SecurityVerdict::Blocked { .. }
        ));
        assert!(matches!(
            SecurityShield::evaluate_path("/home/user/.ssh/id_rsa"),
            SecurityVerdict::Blocked { .. }
        ));
        assert_eq!(SecurityShield::evaluate_path("data/bible.db"), SecurityVerdict::Safe);
    }
}
