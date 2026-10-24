//! Dendrite AutoHarness for Scripture & Study Inquiries
//!
//! Provides biological lifecycle memory management:
//! - Opportunity-relative usage rate: Retrievals / Inquiries Since Creation
//! - 50-inquiry probation shield for newly distilled theological insights
//! - Capacity contention management with cold SQLite archiving
//! - Umbrella consolidation to merge fragmented verse notes into coherent study topics

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const PROBATION_INQUIRY_SHIELD: u64 = 50;
pub const DEFAULT_MAX_MATURE_SKILLS: usize = 30;

/// A distilled study skill, preference, or theological procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedStudySkill {
    pub id: String,
    pub title: String,
    pub topic: String,
    pub summary: String,
    pub created_at_inquiry: u64,
    pub last_retrieved_inquiry: u64,
    pub retrieval_count: u64,
    pub is_archived: bool,
    pub absorbed_by: Option<String>,
}

impl LearnedStudySkill {
    pub fn new(id: impl Into<String>, title: impl Into<String>, topic: impl Into<String>, summary: impl Into<String>, current_inquiry: u64) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            topic: topic.into(),
            summary: summary.into(),
            created_at_inquiry: current_inquiry,
            last_retrieved_inquiry: current_inquiry,
            retrieval_count: 0,
            is_archived: false,
            absorbed_by: None,
        }
    }

    /// Evaluates whether the note is protected under the 50-inquiry probation shield.
    pub fn is_under_probation(&self, current_inquiry: u64) -> bool {
        current_inquiry.saturating_sub(self.created_at_inquiry) < PROBATION_INQUIRY_SHIELD
    }

    /// Computes opportunity-relative usage rate: Retrievals / (Inquiries Since Creation + 1).
    pub fn usage_rate(&self, current_inquiry: u64) -> f32 {
        let opportunities = current_inquiry.saturating_sub(self.created_at_inquiry) + 1;
        self.retrieval_count as f32 / opportunities as f32
    }

    /// Records a successful retrieval event during study inquiry.
    pub fn record_retrieval(&mut self, current_inquiry: u64) {
        self.retrieval_count += 1;
        self.last_retrieved_inquiry = current_inquiry;
    }
}

/// AutoHarness manager tracking active and archived study memory skills.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AutoHarness {
    pub current_inquiry_counter: u64,
    pub max_mature_skills: usize,
    pub skills: HashMap<String, LearnedStudySkill>,
}

impl AutoHarness {
    pub fn new(max_mature: usize) -> Self {
        Self {
            current_inquiry_counter: 0,
            max_mature_skills: if max_mature == 0 { DEFAULT_MAX_MATURE_SKILLS } else { max_mature },
            skills: HashMap::new(),
        }
    }

    /// Increments inquiry counter representing user study turns.
    pub fn step_inquiry(&mut self) -> u64 {
        self.current_inquiry_counter += 1;
        self.current_inquiry_counter
    }

    /// Inserts or updates a learned study skill under the probation shield.
    pub fn register_skill(&mut self, id: &str, title: &str, topic: &str, summary: &str) {
        let cur = self.current_inquiry_counter;
        self.skills.insert(
            id.to_string(),
            LearnedStudySkill::new(id, title, topic, summary, cur),
        );
        self.enforce_capacity();
    }

    /// Retrieves relevant active skills matching a query topic and updates usage rates.
    pub fn query_skills(&mut self, topic: &str) -> Vec<LearnedStudySkill> {
        let cur = self.current_inquiry_counter;
        let query = topic.to_lowercase();
        let mut matches = Vec::new();

        for skill in self.skills.values_mut() {
            if !skill.is_archived && (skill.topic.to_lowercase().contains(&query) || skill.summary.to_lowercase().contains(&query)) {
                skill.record_retrieval(cur);
                matches.push(skill.clone());
            }
        }

        matches.sort_by(|a, b| b.usage_rate(cur).partial_cmp(&a.usage_rate(cur)).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// Performs contradiction and obsolescence pruning based on explicit user correction.
    pub fn apply_correction(&mut self, obsolete_keyword: &str, new_id: &str) -> usize {
        let mut pruned = 0;
        let kw = obsolete_keyword.to_lowercase();
        for skill in self.skills.values_mut() {
            if !skill.is_archived && skill.id != new_id && (skill.title.to_lowercase().contains(&kw) || skill.summary.to_lowercase().contains(&kw)) {
                skill.is_archived = true;
                skill.absorbed_by = Some(new_id.to_string());
                pruned += 1;
            }
        }
        pruned
    }

    /// Umbrella consolidation: collapses multiple fragmented notes of the same topic into one umbrella.
    pub fn consolidate_topic(&mut self, topic: &str, umbrella_id: &str, umbrella_title: &str, synthesized_summary: &str) -> Result<bool> {
        let cur = self.current_inquiry_counter;
        let mut matching_ids = Vec::new();

        for (id, skill) in &self.skills {
            if !skill.is_archived && skill.topic.eq_ignore_ascii_case(topic) && id != umbrella_id {
                matching_ids.push(id.clone());
            }
        }

        if matching_ids.len() < 2 {
            return Ok(false);
        }

        // Archive absorbed notes
        for id in &matching_ids {
            if let Some(s) = self.skills.get_mut(id) {
                s.is_archived = true;
                s.absorbed_by = Some(umbrella_id.to_string());
            }
        }

        // Insert consolidated umbrella
        let mut umbrella = LearnedStudySkill::new(umbrella_id, umbrella_title, topic, synthesized_summary, cur);
        umbrella.retrieval_count = matching_ids.len() as u64;
        self.skills.insert(umbrella_id.to_string(), umbrella);

        Ok(true)
    }

    /// Enforces max mature capacity: evicts lowest-performing mature skills past probation.
    pub fn enforce_capacity(&mut self) {
        let cur = self.current_inquiry_counter;
        let mut mature_ids: Vec<String> = self
            .skills
            .values()
            .filter(|s| !s.is_archived && !s.is_under_probation(cur))
            .map(|s| s.id.clone())
            .collect();

        if mature_ids.len() > self.max_mature_skills {
            mature_ids.sort_by(|a, b| {
                let r_a = self.skills[a].usage_rate(cur);
                let r_b = self.skills[b].usage_rate(cur);
                r_a.partial_cmp(&r_b).unwrap_or(std::cmp::Ordering::Equal)
            });

            let overflow = mature_ids.len() - self.max_mature_skills;
            for id in mature_ids.iter().take(overflow) {
                if let Some(s) = self.skills.get_mut(id) {
                    s.is_archived = true;
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_autoharness_probation_shield_and_usage_rate() {
        let mut harness = AutoHarness::new(10);
        harness.register_skill("daniel_sanctuary", "Sanctuary 2300 Days", "prophecy", "Daniel 8:14 cleansing");

        let skill = harness.skills.get("daniel_sanctuary").unwrap();
        assert!(skill.is_under_probation(10));
        assert!(skill.is_under_probation(49));
        assert!(!skill.is_under_probation(50));
    }

    #[test]
    fn test_autoharness_umbrella_consolidation() {
        let mut harness = AutoHarness::new(10);
        harness.register_skill("note_1", "Dan 8:14", "daniel", "2300 days timeline");
        harness.register_skill("note_2", "Dan 9:24", "daniel", "70 weeks decree");

        let consolidated = harness.consolidate_topic(
            "daniel",
            "topic_daniel_prophetic_chronology",
            "Daniel Prophetic Synthesis",
            "Unified timeline linking 70 weeks to the 2300 days",
        ).unwrap();

        assert!(consolidated);
        assert!(harness.skills.get("note_1").unwrap().is_archived);
        assert!(harness.skills.get("note_2").unwrap().is_archived);
        assert!(!harness.skills.get("topic_daniel_prophetic_chronology").unwrap().is_archived);
    }

    #[test]
    fn test_autoharness_contradiction_pruning() {
        let mut harness = AutoHarness::new(10);
        harness.register_skill("pref_1", "Preferred Bible", "settings", "Use Reina-Valera 1960");
        harness.register_skill("pref_2", "Preferred Bible", "settings", "Use Reina-Valera 1909");

        let pruned = harness.apply_correction("1960", "pref_2");
        assert_eq!(pruned, 1);
        assert!(harness.skills.get("pref_1").unwrap().is_archived);
        assert_eq!(harness.skills.get("pref_1").unwrap().absorbed_by, Some("pref_2".to_string()));
    }

    #[test]
    fn test_autoharness_capacity_eviction_protects_probation() {
        let mut harness = AutoHarness::new(2);
        // Step inquiry past probation for older skills
        harness.current_inquiry_counter = 100;

        harness.skills.insert(
            "old_rare".to_string(),
            LearnedStudySkill {
                id: "old_rare".to_string(),
                title: "Old Rare Note".to_string(),
                topic: "old".to_string(),
                summary: "rarely used".to_string(),
                created_at_inquiry: 0,
                last_retrieved_inquiry: 1,
                retrieval_count: 1,
                is_archived: false,
                absorbed_by: None,
            },
        );
        harness.skills.insert(
            "old_freq".to_string(),
            LearnedStudySkill {
                id: "old_freq".to_string(),
                title: "Old Freq Note".to_string(),
                topic: "old".to_string(),
                summary: "frequently used".to_string(),
                created_at_inquiry: 0,
                last_retrieved_inquiry: 90,
                retrieval_count: 80,
                is_archived: false,
                absorbed_by: None,
            },
        );
        harness.skills.insert(
            "old_medium".to_string(),
            LearnedStudySkill {
                id: "old_medium".to_string(),
                title: "Old Medium Note".to_string(),
                topic: "old".to_string(),
                summary: "medium used".to_string(),
                created_at_inquiry: 0,
                last_retrieved_inquiry: 50,
                retrieval_count: 20,
                is_archived: false,
                absorbed_by: None,
            },
        );

        // We have 3 mature skills with capacity 2. Enforcing should archive old_rare
        harness.enforce_capacity();
        assert!(harness.skills.get("old_rare").unwrap().is_archived);
        assert!(!harness.skills.get("old_freq").unwrap().is_archived);
    }
}
