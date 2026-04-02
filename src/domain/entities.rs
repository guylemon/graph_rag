use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

use crate::domain::canonicalize_entity_type;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct EntityMention {
    pub entity_name: String,
    pub entity_type: String,
    pub entity_description: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct GraphNode {
    pub name: String,
    pub entity_type: String,
    pub description: String,
}

#[derive(Debug)]
struct EntityConstraints {
    max_name_length: usize,
    max_type_length: usize,
    max_description_length: usize,
}

impl Default for EntityConstraints {
    fn default() -> Self {
        Self {
            max_name_length: 200,
            max_type_length: 100,
            max_description_length: usize::MAX,
        }
    }
}

/// Normalizes raw entity mentions into validated graph nodes.
///
/// Processing steps:
/// - removes duplicate entities by name and type
/// - trims and validates fields
///
/// Notes on current behavior:
/// - deduplication happens before trimming/validation
/// - if the first duplicate is invalid, later duplicates are still dropped
/// - whitespace-only key differences are treated as distinct during deduplication
pub(crate) fn normalize_entities(entities: Vec<EntityMention>) -> Vec<GraphNode> {
    let constraints = EntityConstraints::default();

    entities
        .into_iter()
        .filter_map(|e| clean_extracted_entity(e, &constraints))
        .filter({
            let mut seen = HashSet::new();
            move |e| seen.insert((e.name.clone(), e.entity_type.clone()))
        })
        .collect()
}

fn clean_extracted_entity(
    mut mention: EntityMention,
    constraints: &EntityConstraints,
) -> Option<GraphNode> {
    mention.entity_name = mention.entity_name.trim().to_string();
    mention.entity_type = canonicalize_entity_type(&mention.entity_type)?;
    mention.entity_description = mention.entity_description.trim().to_string();

    if mention.entity_name.is_empty()
        || mention.entity_type.is_empty()
        || mention.entity_description.is_empty()
    {
        return None;
    }

    if mention.entity_name.len() > constraints.max_name_length
        || mention.entity_type.len() > constraints.max_type_length
        || mention.entity_description.len() > constraints.max_description_length
    {
        return None;
    }

    let node = GraphNode {
        name: mention.entity_name,
        entity_type: mention.entity_type,
        description: mention.entity_description,
    };

    Some(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mention(name: &str, entity_type: &str, description: &str) -> EntityMention {
        EntityMention {
            entity_name: name.to_string(),
            entity_type: entity_type.to_string(),
            entity_description: description.to_string(),
        }
    }

    #[test]
    fn normalize_entities_empty_input_returns_empty() {
        let result = normalize_entities(vec![]);

        assert!(result.is_empty());
    }

    #[test]
    fn normalize_entities_valid_single_entity_maps_fields() {
        let result = normalize_entities(vec![mention("Alice", "Person", "Engineer")]);

        assert_eq!(
            result,
            vec![GraphNode {
                name: "Alice".to_string(),
                entity_type: "PERSON".to_string(),
                description: "Engineer".to_string(),
            }]
        );
    }

    #[test]
    fn normalize_entities_trims_all_fields() {
        let result = normalize_entities(vec![mention("  Alice\n", "\tPerson\t", "\nEngineer\t ")]);

        assert_eq!(
            result,
            vec![GraphNode {
                name: "Alice".to_string(),
                entity_type: "PERSON".to_string(),
                description: "Engineer".to_string(),
            }]
        );
    }

    #[test]
    fn normalize_entities_drops_empty_name_after_trim() {
        let result = normalize_entities(vec![mention("   ", "Person", "Engineer")]);

        assert!(result.is_empty());
    }

    #[test]
    fn normalize_entities_drops_empty_type_after_trim() {
        let result = normalize_entities(vec![mention("Alice", "\n\t ", "Engineer")]);

        assert!(result.is_empty());
    }

    #[test]
    fn normalize_entities_drops_empty_description_after_trim() {
        let result = normalize_entities(vec![mention("Alice", "Person", "  ")]);

        assert!(result.is_empty());
    }

    #[test]
    fn normalize_entities_accepts_boundary_name_and_type_lengths() {
        let name = "n".repeat(200);
        let entity_type = "AUTHOR".to_string();
        let description = "ok".to_string();

        let result = normalize_entities(vec![mention(&name, &entity_type, &description)]);

        assert_eq!(
            result,
            vec![GraphNode {
                name,
                entity_type,
                description,
            }]
        );
    }

    #[test]
    fn normalize_entities_rejects_overlong_name() {
        let name = "n".repeat(201);

        let result = normalize_entities(vec![mention(&name, "Person", "Engineer")]);

        assert!(result.is_empty());
    }

    #[test]
    fn normalize_entities_rejects_overlong_type() {
        let entity_type = "unknown";

        let result = normalize_entities(vec![mention("Alice", &entity_type, "Engineer")]);

        assert!(result.is_empty());
    }

    #[test]
    fn normalize_entities_deduplicates_exact_name_type_keeps_first() {
        let result = normalize_entities(vec![
            mention("Alice", "Person", "First"),
            mention("Alice", "Person", "Second"),
        ]);

        assert_eq!(
            result,
            vec![GraphNode {
                name: "Alice".to_string(),
                entity_type: "PERSON".to_string(),
                description: "First".to_string(),
            }]
        );
    }

    #[test]
    fn normalize_entities_keeps_same_name_different_type() {
        let result = normalize_entities(vec![
            mention("Alice", "Person", "Engineer"),
            mention("Alice", "Organization", "Acme"),
        ]);

        assert_eq!(
            result,
            vec![
                GraphNode {
                    name: "Alice".to_string(),
                    entity_type: "PERSON".to_string(),
                    description: "Engineer".to_string(),
                },
                GraphNode {
                    name: "Alice".to_string(),
                    entity_type: "ORGANIZATION".to_string(),
                    description: "Acme".to_string(),
                },
            ]
        );
    }

    #[test]
    fn normalize_entities_preserves_order_of_surviving_entities() {
        let result = normalize_entities(vec![
            mention("A", "Concept", "first"),
            mention("Ignored", "   ", "invalid"),
            mention("B", "Concept", "second"),
            mention("A", "Concept", "duplicate"),
            mention("C", "Concept", "third"),
        ]);

        assert_eq!(
            result,
            vec![
                GraphNode {
                    name: "A".to_string(),
                    entity_type: "CONCEPT".to_string(),
                    description: "first".to_string(),
                },
                GraphNode {
                    name: "B".to_string(),
                    entity_type: "CONCEPT".to_string(),
                    description: "second".to_string(),
                },
                GraphNode {
                    name: "C".to_string(),
                    entity_type: "CONCEPT".to_string(),
                    description: "third".to_string(),
                },
            ]
        );
    }

    #[test]
    fn clean_extracted_entity_rejects_overlong_description_with_custom_constraints() {
        let constraints = EntityConstraints {
            max_name_length: 200,
            max_type_length: 100,
            max_description_length: 3,
        };

        let result = clean_extracted_entity(mention("Alice", "Person", "long"), &constraints);

        assert!(result.is_none());
    }

    #[test]
    fn clean_extracted_entity_accepts_description_at_custom_boundary() {
        let constraints = EntityConstraints {
            max_name_length: 200,
            max_type_length: 100,
            max_description_length: 4,
        };

        let result = clean_extracted_entity(mention("Alice", "Person", "long"), &constraints);

        assert_eq!(
            result,
            Some(GraphNode {
                name: "Alice".to_string(),
                entity_type: "PERSON".to_string(),
                description: "long".to_string(),
            })
        );
    }
}
