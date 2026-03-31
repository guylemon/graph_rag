use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::hash::Hash;

use crate::domain::util::deduplicate;

#[derive(Clone, Debug, Eq, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct RelationshipMention {
    pub source: String,
    pub target: String,
    pub keywords: Vec<String>,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct GraphEdge {
    pub source: String,
    pub target: String,
    pub keyword: String,
    pub description: String,
}

/// Normalizes raw relationship mentions into validated graph edges.
///
/// Processing steps:
/// - removes duplicate `RelationshipMention` values,
/// - trims and validates source/target/keywords,
/// - keeps only relationships whose source and target are in `seen_entities`,
/// - expands each relationship into one `GraphEdge` per keyword.
pub(crate) fn normalize_relationships(
    relationships: Vec<RelationshipMention>,
    seen_entities: HashSet<String>,
) -> Vec<GraphEdge> {
    let max_entity_len = 200;
    let max_keyword_len = 100;

    relationships
        .into_iter()
        .filter(deduplicate())
        .filter_map(|r| clean_extracted_relationship(r, max_entity_len, max_keyword_len))
        .filter(|r| is_valid_relationship(r, &seen_entities))
        .flat_map(expand_single_relationship)
        .collect()
}

/// Expands a single relationship mention into one edge per keyword.
///
/// The returned iterator preserves keyword order and duplicates the same
/// source, target, and description across all produced `GraphEdge`s.
fn expand_single_relationship(
    relationship: RelationshipMention,
) -> impl Iterator<Item = GraphEdge> {
    let source = relationship.source;
    let target = relationship.target;
    let description = relationship.description;

    relationship
        .keywords
        .into_iter()
        .map(move |keyword| GraphEdge {
            keyword,
            description: description.to_owned(),
            source: source.to_owned(),
            target: target.to_owned(),
        })
}

/// Returns `true` when both relationship endpoints are known entities.
fn is_valid_relationship(relationship: &RelationshipMention, entities: &HashSet<String>) -> bool {
    entities.contains(&relationship.source) && entities.contains(&relationship.target)
}

/// Cleans a single raw LLM-extracted relationship:
/// - trims whitespace from all string fields,
/// - removes empty or overly long keywords,
/// - deduplicates keywords (preserving first-occurrence order),
/// - discards the relationship if source/target entities are empty or exceed length limits.
fn clean_extracted_relationship(
    mut rel: RelationshipMention,
    max_entity_len: usize,
    max_keyword_len: usize,
) -> Option<RelationshipMention> {
    // Trim and validate entities (early exit for clearly invalid items).
    rel.source = rel.source.trim().to_string();
    rel.target = rel.target.trim().to_string();
    rel.description = rel.description.trim().to_string();

    if rel.source.is_empty()
        || rel.target.is_empty()
        || rel.source.len() > max_entity_len
        || rel.target.len() > max_entity_len
    {
        return None;
    }

    // Clean keywords: trim, remove empty/over-length, deduplicate.
    let mut seen = std::collections::HashSet::new();
    let cleaned_keywords: Vec<String> = rel
        .keywords
        .into_iter()
        .filter_map(|k| {
            let trimmed = k.trim().to_string();
            if trimmed.is_empty() || trimmed.len() > max_keyword_len {
                None
            } else if seen.insert(trimmed.clone()) {
                Some(trimmed)
            } else {
                None
            }
        })
        .collect();

    if cleaned_keywords.is_empty() {
        return None;
    }

    rel.keywords = cleaned_keywords;
    Some(rel)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel(
        source_entity: &str,
        target_entity: &str,
        relationship_keywords: &[&str],
        relationship_description: &str,
    ) -> RelationshipMention {
        RelationshipMention {
            source: source_entity.to_owned(),
            target: target_entity.to_owned(),
            keywords: relationship_keywords
                .iter()
                .map(|keyword| (*keyword).to_owned())
                .collect(),
            description: relationship_description.to_owned(),
        }
    }

    fn vrel(
        source_entity: &str,
        target_entity: &str,
        keyword: &str,
        relationship_description: &str,
    ) -> GraphEdge {
        GraphEdge {
            source: source_entity.to_owned(),
            target: target_entity.to_owned(),
            keyword: keyword.to_owned(),
            description: relationship_description.to_owned(),
        }
    }

    fn seen_entities(entities: &[&str]) -> HashSet<String> {
        entities.iter().map(|entity| (*entity).to_owned()).collect()
    }

    #[test]
    fn standardize_relationships_returns_empty_for_empty_input() {
        let out = normalize_relationships(vec![], seen_entities(&[]));

        assert!(out.is_empty());
    }

    #[test]
    fn standardize_relationships_expands_single_relationship_with_single_keyword() {
        let input = vec![rel("A", "B", &["is_a"], "A is a B")];

        let out = normalize_relationships(input, seen_entities(&["A", "B"]));

        assert_eq!(out, vec![vrel("A", "B", "is_a", "A is a B")]);
    }

    #[test]
    fn standardize_relationships_expands_single_relationship_with_multiple_keywords_in_order() {
        let input = vec![rel(
            "A",
            "B",
            &["is_a", "authored_by", "related_to"],
            "desc",
        )];

        let out = normalize_relationships(input, seen_entities(&["A", "B"]));

        assert_eq!(
            out,
            vec![
                vrel("A", "B", "is_a", "desc"),
                vrel("A", "B", "authored_by", "desc"),
                vrel("A", "B", "related_to", "desc"),
            ]
        );
    }

    #[test]
    fn standardize_relationships_flattens_multiple_relationships_in_iteration_order() {
        let input = vec![
            rel("A", "B", &["k1", "k2"], "d1"),
            rel("C", "D", &["k3"], "d2"),
        ];

        let out = normalize_relationships(input, seen_entities(&["A", "B", "C", "D"]));

        assert_eq!(
            out,
            vec![
                vrel("A", "B", "k1", "d1"),
                vrel("A", "B", "k2", "d1"),
                vrel("C", "D", "k3", "d2"),
            ]
        );
    }

    #[test]
    fn standardize_relationships_relationship_with_no_keywords_contributes_no_rows() {
        let input = vec![rel("A", "B", &[], "desc")];

        let out = normalize_relationships(input, seen_entities(&["A", "B"]));

        assert!(out.is_empty());
    }

    #[test]
    fn standardize_relationships_returns_empty_for_empty_input_when_seen_entities_non_empty() {
        let out = normalize_relationships(vec![], seen_entities(&["A", "B"]));

        assert!(out.is_empty());
    }

    #[test]
    fn standardize_relationships_filters_everything_when_seen_entities_empty() {
        let input = vec![rel("A", "B", &["k1"], "d1"), rel("B", "C", &["k2"], "d2")];

        let out = normalize_relationships(input, seen_entities(&[]));

        assert!(out.is_empty());
    }

    #[test]
    fn standardize_relationships_keeps_only_edges_with_seen_source_and_target() {
        let input = vec![
            rel("A", "B", &["k1"], "valid"),
            rel("A", "X", &["k2"], "invalid_target"),
            rel("Y", "B", &["k3"], "invalid_source"),
        ];

        let out = normalize_relationships(input, seen_entities(&["A", "B"]));

        assert_eq!(out, vec![vrel("A", "B", "k1", "valid")]);
    }

    #[test]
    fn standardize_relationships_preserves_relative_order_of_retained_edges() {
        let input = vec![
            rel("A", "B", &["k1"], "first_valid"),
            rel("A", "X", &["k2"], "invalid"),
            rel("B", "A", &["k3"], "second_valid"),
        ];

        let out = normalize_relationships(input, seen_entities(&["A", "B"]));

        assert_eq!(
            out,
            vec![
                vrel("A", "B", "k1", "first_valid"),
                vrel("B", "A", "k3", "second_valid"),
            ]
        );
    }

    #[test]
    fn standardize_relationships_keeps_self_loop_if_entity_seen() {
        let input = vec![rel("A", "A", &["k1"], "self")];

        let out = normalize_relationships(input, seen_entities(&["A"]));

        assert_eq!(out, vec![vrel("A", "A", "k1", "self")]);
    }

    #[test]
    fn standardize_relationships_uses_case_sensitive_matching() {
        let input = vec![rel("Alice", "Bob", &["k1"], "desc")];

        let out = normalize_relationships(input, seen_entities(&["alice", "bob"]));

        assert!(out.is_empty());
    }
}
