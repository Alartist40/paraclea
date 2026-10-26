//! Multi-Column Prophetic Timeline Visualizer.
//!
//! Synchronizes historical epochs, Daniel's visions, Revelation's prophecies,
//! and Sanctuary typology across a unified chronological horology grid.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEpoch {
    pub year_marker: &'static str,
    pub daniel_vision: &'static str,
    pub revelation_event: &'static str,
    pub sanctuary_phase: &'static str,
    pub historical_fulfillment: &'static str,
}

pub const PROPHETIC_EPOCHS: &[TimelineEpoch] = &[
    TimelineEpoch {
        year_marker: "605 BC",
        daniel_vision: "Dan 2 Head of Gold / Dan 7 Lion",
        revelation_event: "Echoed in Rev 13:2 (lion's mouth)",
        sanctuary_phase: "Camp in Wilderness / Babylonian Exile",
        historical_fulfillment: "Rise of Neo-Babylonian Empire (Nebuchadnezzar II)",
    },
    TimelineEpoch {
        year_marker: "538 BC",
        daniel_vision: "Dan 2 Breast of Silver / Dan 7 Bear / Dan 8 Ram",
        revelation_event: "Echoed in Rev 13:2 (bear's feet)",
        sanctuary_phase: "Decree to rebuild Jerusalem altar",
        historical_fulfillment: "Fall of Babylon to Medo-Persia (Cyrus the Great)",
    },
    TimelineEpoch {
        year_marker: "457 BC",
        daniel_vision: "Dan 9:25 Decree to restore Jerusalem",
        revelation_event: "Origin anchor of the 2300 evenings/mornings",
        sanctuary_phase: "Artaxerxes Longimanus 7th year decree",
        historical_fulfillment: "Ezra returns with full civil authority to Judea",
    },
    TimelineEpoch {
        year_marker: "27 AD",
        daniel_vision: "Dan 9:25 Anointing of Messiah the Prince",
        revelation_event: "Rev 12:1-5 Birth & Ministry of the Male Child",
        sanctuary_phase: "Holy Spirit anoints the True High Priest at Jordan",
        historical_fulfillment: "Baptism of Jesus Christ (15th year of Tiberius Caesar)",
    },
    TimelineEpoch {
        year_marker: "31 AD",
        daniel_vision: "Dan 9:27 Messiah cut off in the midst of the week",
        revelation_event: "Rev 5:6 Lamb as it had been slain",
        sanctuary_phase: "Veil torn in two; animal sacrifices cease in efficacy",
        historical_fulfillment: "Crucifixion and Resurrection of Jesus Christ at Passover",
    },
    TimelineEpoch {
        year_marker: "34 AD",
        daniel_vision: "Dan 9:24 End of the 70 prophetic weeks (490 years)",
        revelation_event: "Rev 12:11 Word of their testimony unto death",
        sanctuary_phase: "Gospel commission shifts outward to all nations",
        historical_fulfillment: "Stoning of Stephen; conversion of Saul of Tarsus",
    },
    TimelineEpoch {
        year_marker: "538 AD",
        daniel_vision: "Dan 7:25 Start of Time, Times, and Half a Time",
        revelation_event: "Rev 12:6 Woman flees into wilderness (1260 days)",
        sanctuary_phase: "Outer Court trampled (Rev 11:2)",
        historical_fulfillment: "Ostrogothic siege of Rome broken; Justinian decree active",
    },
    TimelineEpoch {
        year_marker: "1798 AD",
        daniel_vision: "Dan 7:26 Judgment begins; mortal wound inflicted",
        revelation_event: "Rev 13:3 One of its heads as it were wounded to death",
        sanctuary_phase: "Time of the end begins (Dan 12:4)",
        historical_fulfillment: "General Berthier takes Pope Pius VI captive to Valence",
    },
    TimelineEpoch {
        year_marker: "1844 AD",
        daniel_vision: "Dan 8:14 Unto 2300 days; then sanctuary cleansed",
        revelation_event: "Rev 10:1-11 Little Book opened / Rev 14:7 Judgment hour",
        sanctuary_phase: "Antitypical Day of Atonement; Most Holy Place ministry",
        historical_fulfillment: "End of 2300 years (457 BC - 1844 AD); Great Advent movement",
    },
];

pub struct TimelineVisualizer;

impl TimelineVisualizer {
    /// Renders a multi-column text table aligning the prophetic tracks.
    pub fn render_table() -> String {
        let mut out = String::new();
        out.push_str("============================ PROPHETIC HOROLOGY MATRIX ============================\n");
        out.push_str(format!(
            "{:<9} | {:<24} | {:<22} | {:<25}\n",
            "YEAR", "DANIEL VISION", "REVELATION EVENT", "HISTORICAL FULFILLMENT"
        ).as_str());
        out.push_str("-----------------------------------------------------------------------------------\n");

        for epoch in PROPHETIC_EPOCHS {
            out.push_str(format!(
                "{:<9} | {:<24} | {:<22} | {:<25}\n",
                epoch.year_marker,
                truncate_str(epoch.daniel_vision, 24),
                truncate_str(epoch.revelation_event, 22),
                truncate_str(epoch.historical_fulfillment, 25),
            ).as_str());
        }
        out.push_str("===================================================================================\n");
        out
    }

    /// Finds prophetic matches for a given year query or keyword.
    pub fn query_epoch(query: &str) -> Vec<&'static TimelineEpoch> {
        let q = query.to_lowercase();
        PROPHETIC_EPOCHS
            .iter()
            .filter(|e| {
                e.year_marker.to_lowercase().contains(&q)
                    || e.daniel_vision.to_lowercase().contains(&q)
                    || e.revelation_event.to_lowercase().contains(&q)
                    || e.sanctuary_phase.to_lowercase().contains(&q)
                    || e.historical_fulfillment.to_lowercase().contains(&q)
            })
            .collect()
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_len - 3).collect();
        truncated.push_str("...");
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prophetic_timeline_query() {
        let hits = TimelineVisualizer::query_epoch("1844");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].year_marker, "1844 AD");
        assert!(hits[0].daniel_vision.contains("Dan 8:14"));

        let dan9 = TimelineVisualizer::query_epoch("457 BC");
        assert_eq!(dan9.len(), 2);
        assert!(dan9[0].historical_fulfillment.contains("Ezra"));
    }

    #[test]
    fn test_table_render() {
        let table = TimelineVisualizer::render_table();
        assert!(table.contains("PROPHETIC HOROLOGY MATRIX"));
        assert!(table.contains("1844 AD"));
    }
}
