use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

use crate::domain::GraphNode;
use crate::domain::RelationshipKeyword;
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

pub(crate) fn normalize_relationship_mentions(
    relationships: Vec<RelationshipMention>,
) -> Vec<RelationshipMention> {
    let max_entity_len = 200;
    let max_keyword_len = 100;

    relationships
        .into_iter()
        .filter(deduplicate())
        .filter_map(|relationship| {
            clean_extracted_relationship(relationship, max_entity_len, max_keyword_len)
        })
        .collect()
}

pub(crate) fn retain_known_entity_relationships(
    relationships: Vec<RelationshipMention>,
    entities: &[GraphNode],
) -> Vec<RelationshipMention> {
    let seen_entities: HashSet<String> =
        entities.iter().map(|entity| entity.name.clone()).collect();

    relationships
        .into_iter()
        .filter(|relationship| {
            seen_entities.contains(&relationship.source) && seen_entities.contains(&relationship.target)
        })
        .collect()
}

pub(crate) fn expand_relationships(relationships: Vec<RelationshipMention>) -> Vec<GraphEdge> {
    relationships
        .into_iter()
        .flat_map(expand_single_relationship)
        .filter(deduplicate())
        .collect()
}

#[cfg(test)]
pub(crate) fn normalize_relationships(
    relationships: Vec<RelationshipMention>,
    entities: &[GraphNode],
) -> Vec<GraphEdge> {
    expand_relationships(retain_known_entity_relationships(
        normalize_relationship_mentions(relationships),
        entities,
    ))
}

fn expand_single_relationship(relationship: RelationshipMention) -> impl Iterator<Item = GraphEdge> {
    let source = relationship.source;
    let target = relationship.target;
    let description = relationship.description;

    relationship.keywords.into_iter().filter_map(move |keyword| {
        let keyword = RelationshipKeyword::parse(&keyword)?;
        Some(GraphEdge {
            keyword: keyword.as_str().to_string(),
            description: description.to_owned(),
            source: source.to_owned(),
            target: target.to_owned(),
        })
    })
}

fn clean_extracted_relationship(
    mut rel: RelationshipMention,
    max_entity_len: usize,
    max_keyword_len: usize,
) -> Option<RelationshipMention> {
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

    let mut seen = HashSet::new();
    let cleaned_keywords: Vec<String> = rel
        .keywords
        .into_iter()
        .filter_map(|keyword| {
            let trimmed = keyword.trim().to_string();
            if trimmed.is_empty() || trimmed.len() > max_keyword_len {
                return None;
            }

            let canonical_keyword = RelationshipKeyword::parse(&trimmed)?;
            let canonical_keyword = canonical_keyword.as_str().to_string();

            if seen.insert(canonical_keyword.clone()) {
                Some(canonical_keyword)
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

    fn seen_entities(entities: &[(&str, &str)]) -> Vec<GraphNode> {
        entities
            .iter()
            .map(|(name, entity_type)| GraphNode {
                name: (*name).to_owned(),
                entity_type: (*entity_type).to_owned(),
                description: "desc".to_owned(),
            })
            .collect()
    }

    #[test]
    fn normalize_relationship_mentions_returns_empty_for_empty_input() {
        let out = normalize_relationship_mentions(vec![]);

        assert!(out.is_empty());
    }

    #[test]
    fn normalize_relationships_expands_single_relationship_with_single_keyword() {
        let input = vec![rel("A", "B", &["worked_at"], "A worked at B")];

        let out = normalize_relationships(input, &seen_entities(&[("A", "AUTHOR"), ("B", "ORGANIZATION")]));

        assert_eq!(out, vec![vrel("A", "B", "WORKED_AT", "A worked at B")]);
    }

    #[test]
    fn normalize_relationships_expands_multiple_keywords_in_order() {
        let input = vec![rel(
            "A",
            "B",
            &["worked_at", "related_to", "worked_at"],
            "desc",
        )];

        let out = normalize_relationships(input, &seen_entities(&[("A", "AUTHOR"), ("B", "ORGANIZATION")]));

        assert_eq!(
            out,
            vec![
                vrel("A", "B", "WORKED_AT", "desc"),
                vrel("A", "B", "RELATED_TO", "desc"),
            ]
        );
    }

    #[test]
    fn normalize_relationships_keeps_only_edges_with_seen_source_and_target() {
        let input = vec![
            rel("A", "B", &["WORKED_AT"], "valid"),
            rel("A", "X", &["RELATED_TO"], "invalid_target"),
            rel("Y", "B", &["RELATED_TO"], "invalid_source"),
        ];

        let out = normalize_relationships(input, &seen_entities(&[("A", "AUTHOR"), ("B", "ORGANIZATION")]));

        assert_eq!(out, vec![vrel("A", "B", "WORKED_AT", "valid")]);
    }

    #[test]
    fn normalize_relationships_drops_unknown_keywords() {
        let input = vec![rel("A", "B", &["made_of"], "invalid")];

        let out = normalize_relationships(input, &seen_entities(&[("A", "AUTHOR"), ("B", "ORGANIZATION")]));

        assert!(out.is_empty());
    }
}
