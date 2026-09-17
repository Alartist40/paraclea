//! Paraclea Galaxy 3D interface module.

pub mod data;
pub mod physics;
pub mod renderer;

#[cfg(test)]
mod tests {
    use super::data::build_galaxy;
    use super::physics::{EntityType, GalaxyNode, GalaxySystem};
    use super::renderer::{Camera3D, GalaxyRenderer};
    use paraclea_core::bible::BibleReader;
    use paraclea_core::library::LibraryEngine;

    #[test]
    fn test_galaxy_hierarchical_orbit() {
        let mut system = GalaxySystem::new();
        let sun = GalaxyNode::new_sun("sun", "The Word");
        let planet = GalaxyNode::new_planet("earth", "Earth", "ENG", 10.0, 1.0, 0.0, 0.0, 0.85);
        let moon = GalaxyNode::new_moon("moon", "Luna", "LUN", "earth", 2.0, 2.0, 0.0, 0.5, true);

        system.add_node(sun);
        system.add_node(planet);
        system.add_node(moon);

        // Initial positions at time 0
        system.update(0.0);
        assert_eq!(system.nodes[0].pos, [0.0, 0.0, 0.0]);
        assert!((system.nodes[1].pos[0] - 10.0).abs() < 1e-4);
        assert!((system.nodes[2].pos[0] - 12.0).abs() < 1e-4);

        // Step simulation forward by PI/2
        system.update(std::f32::consts::PI / 2.0);
        assert_eq!(system.nodes[0].pos, [0.0, 0.0, 0.0]);
        // Planet at angle PI/2 with speed 1.0 => x ~ 0, z ~ 10
        assert!(system.nodes[1].pos[0].abs() < 0.1);
        assert!((system.nodes[1].pos[2] - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_galaxy_projection_depth() {
        let cam = Camera3D::new();
        let center = GalaxyRenderer::project_point([0.0, 0.0, 0.0], &cam, 100, 50);
        assert!(center.visible);
        assert_eq!(center.screen_x, 50);
        assert_eq!(center.screen_y, 25);
        assert!(center.depth > 0.0);

        // Far away point behind camera
        let behind = GalaxyRenderer::project_point([0.0, 0.0, -100.0], &cam, 100, 50);
        assert!(!behind.visible || behind.depth < 2.0 || behind.screen_x < 0 || behind.screen_x >= 100);
    }

    #[test]
    fn test_galaxy_data_population() {
        let reader = BibleReader::load_auto().unwrap_or_else(|_| BibleReader::from_json_str("[]").unwrap());
        let library = LibraryEngine::load_auto();
        let system = build_galaxy(&reader, &library);

        assert!(!system.nodes.is_empty());
        assert_eq!(system.nodes[0].entity_type, EntityType::Sun);

        let planet_count = system.nodes.iter().filter(|n| n.entity_type == EntityType::Planet).count();
        let moon_count = system.nodes.iter().filter(|n| n.entity_type == EntityType::Moon).count();

        assert!(planet_count > 0, "Planets should be populated from languages");
        assert!(moon_count > 0, "Moons should be populated from translations");
    }

    #[test]
    fn test_galaxy_enter_inspection() {
        use crate::views::galaxy::{GalaxyAction, GalaxyState};
        use crossterm::event::{KeyCode, KeyModifiers};

        let reader = BibleReader::load_auto().unwrap_or_else(|_| BibleReader::from_json_str("[]").unwrap());
        let library = LibraryEngine::load_auto();
        let mut state = GalaxyState::new(&reader, &library);

        assert!(!state.system.nodes.is_empty());
        let action = state.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        match action {
            GalaxyAction::Inspect(node) => {
                assert_eq!(node.id, state.system.nodes[0].id);
            }
            _ => panic!("Expected GalaxyAction::Inspect on Enter"),
        }
    }
}
