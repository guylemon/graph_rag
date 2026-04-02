use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use crate::domain::{GraphEdge, GraphNode};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum EntityType {
    Author,
    Concept,
    Event,
    Lifeform,
    Location,
    Organization,
    Person,
    Product,
    Technology,
}

impl EntityType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Author => "AUTHOR",
            Self::Concept => "CONCEPT",
            Self::Event => "EVENT",
            Self::Lifeform => "LIFEFORM",
            Self::Location => "LOCATION",
            Self::Organization => "ORGANIZATION",
            Self::Person => "PERSON",
            Self::Product => "PRODUCT",
            Self::Technology => "TECHNOLOGY",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "AUTHOR" => Some(Self::Author),
            "CONCEPT" => Some(Self::Concept),
            "EVENT" => Some(Self::Event),
            "LIFEFORM" => Some(Self::Lifeform),
            "LOCATION" => Some(Self::Location),
            "ORGANIZATION" => Some(Self::Organization),
            "PERSON" => Some(Self::Person),
            "PRODUCT" => Some(Self::Product),
            "TECHNOLOGY" => Some(Self::Technology),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum RelationshipKeyword {
    WorkedAt,
    RelatedTo,
    LocatedIn,
    CollaboratedWith,
    PartOf,
    Created,
    Uses,
    Implements,
    ParticipatedIn,
    OccurredIn,
    AffiliatedWith,
    MentionedWith,
    Founded,
}

impl RelationshipKeyword {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::WorkedAt => "WORKED_AT",
            Self::RelatedTo => "RELATED_TO",
            Self::LocatedIn => "LOCATED_IN",
            Self::CollaboratedWith => "COLLABORATED_WITH",
            Self::PartOf => "PART_OF",
            Self::Created => "CREATED",
            Self::Uses => "USES",
            Self::Implements => "IMPLEMENTS",
            Self::ParticipatedIn => "PARTICIPATED_IN",
            Self::OccurredIn => "OCCURRED_IN",
            Self::AffiliatedWith => "AFFILIATED_WITH",
            Self::MentionedWith => "MENTIONED_WITH",
            Self::Founded => "FOUNDED",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "WORKED_AT" => Some(Self::WorkedAt),
            "RELATED_TO" => Some(Self::RelatedTo),
            "LOCATED_IN" => Some(Self::LocatedIn),
            "COLLABORATED_WITH" => Some(Self::CollaboratedWith),
            "PART_OF" => Some(Self::PartOf),
            "CREATED" => Some(Self::Created),
            "USES" => Some(Self::Uses),
            "IMPLEMENTS" => Some(Self::Implements),
            "PARTICIPATED_IN" => Some(Self::ParticipatedIn),
            "OCCURRED_IN" => Some(Self::OccurredIn),
            "AFFILIATED_WITH" => Some(Self::AffiliatedWith),
            "MENTIONED_WITH" => Some(Self::MentionedWith),
            "FOUNDED" => Some(Self::Founded),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InvariantCode {
    AtLeastTwoNodes,
    InvalidEdgePairing,
    NoNodeWithoutEdge,
}

impl InvariantCode {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::AtLeastTwoNodes => "AT_LEAST_TWO_NODES",
            Self::InvalidEdgePairing => "INVALID_EDGE_PAIRING",
            Self::NoNodeWithoutEdge => "NO_NODE_WITHOUT_EDGE",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InvariantPhase {
    EntityExtraction,
    RelationshipExtraction,
    GraphFinalization,
}

impl InvariantPhase {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::EntityExtraction => "entity_extraction",
            Self::RelationshipExtraction => "relationship_extraction",
            Self::GraphFinalization => "graph_finalization",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InvariantViolation {
    pub(crate) code: InvariantCode,
    pub(crate) phase: InvariantPhase,
    pub(crate) message: String,
    pub(crate) entity_names: Vec<String>,
    pub(crate) edges: Vec<(String, String, String)>,
    pub(crate) attempt: usize,
}

#[derive(Debug)]
pub(crate) struct InvariantError {
    pub(crate) violations: Vec<InvariantViolation>,
}

impl fmt::Display for InvariantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, violation) in self.violations.iter().enumerate() {
            if idx > 0 {
                write!(f, "; ")?;
            }
            write!(
                f,
                "[{}:{} attempt={}] {}",
                violation.phase.as_str(),
                violation.code.as_str(),
                violation.attempt,
                violation.message
            )?;
        }
        Ok(())
    }
}

impl Error for InvariantError {}

#[derive(Clone, Copy)]
struct RelationshipRule {
    keyword: RelationshipKeyword,
    source_types: &'static [EntityType],
    target_types: &'static [EntityType],
    wildcard_source: bool,
    wildcard_target: bool,
}

impl RelationshipRule {
    fn allows(&self, source: EntityType, target: EntityType) -> bool {
        (self.wildcard_source || self.source_types.contains(&source))
            && (self.wildcard_target || self.target_types.contains(&target))
    }
}

const EMPTY_TYPES: &[EntityType] = &[];

const RELATIONSHIP_RULES: &[RelationshipRule] = &[
    RelationshipRule {
        keyword: RelationshipKeyword::WorkedAt,
        source_types: &[EntityType::Person, EntityType::Author],
        target_types: &[EntityType::Organization],
        wildcard_source: false,
        wildcard_target: false,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::RelatedTo,
        source_types: EMPTY_TYPES,
        target_types: EMPTY_TYPES,
        wildcard_source: true,
        wildcard_target: true,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::LocatedIn,
        source_types: &[
            EntityType::Person,
            EntityType::Organization,
            EntityType::Event,
            EntityType::Product,
            EntityType::Lifeform,
            EntityType::Author,
        ],
        target_types: &[EntityType::Location],
        wildcard_source: false,
        wildcard_target: false,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::CollaboratedWith,
        source_types: &[EntityType::Author, EntityType::Person],
        target_types: &[EntityType::Author, EntityType::Person],
        wildcard_source: false,
        wildcard_target: false,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::PartOf,
        source_types: &[
            EntityType::Person,
            EntityType::Organization,
            EntityType::Location,
            EntityType::Concept,
            EntityType::Lifeform,
            EntityType::Author,
        ],
        target_types: &[
            EntityType::Organization,
            EntityType::Location,
            EntityType::Concept,
            EntityType::Lifeform,
        ],
        wildcard_source: false,
        wildcard_target: false,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::Created,
        source_types: &[EntityType::Person, EntityType::Organization, EntityType::Author],
        target_types: &[
            EntityType::Product,
            EntityType::Technology,
            EntityType::Concept,
        ],
        wildcard_source: false,
        wildcard_target: false,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::Uses,
        source_types: &[
            EntityType::Person,
            EntityType::Organization,
            EntityType::Lifeform,
            EntityType::Author,
        ],
        target_types: &[EntityType::Technology, EntityType::Product],
        wildcard_source: false,
        wildcard_target: false,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::Implements,
        source_types: &[EntityType::Technology, EntityType::Product],
        target_types: &[EntityType::Concept],
        wildcard_source: false,
        wildcard_target: false,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::ParticipatedIn,
        source_types: &[EntityType::Person, EntityType::Organization, EntityType::Author],
        target_types: &[EntityType::Event],
        wildcard_source: false,
        wildcard_target: false,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::OccurredIn,
        source_types: &[EntityType::Event],
        target_types: &[EntityType::Location],
        wildcard_source: false,
        wildcard_target: false,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::AffiliatedWith,
        source_types: &[EntityType::Person, EntityType::Author],
        target_types: &[EntityType::Organization],
        wildcard_source: false,
        wildcard_target: false,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::MentionedWith,
        source_types: EMPTY_TYPES,
        target_types: EMPTY_TYPES,
        wildcard_source: true,
        wildcard_target: true,
    },
    RelationshipRule {
        keyword: RelationshipKeyword::Founded,
        source_types: &[EntityType::Person, EntityType::Author],
        target_types: &[EntityType::Organization],
        wildcard_source: false,
        wildcard_target: false,
    },
];

pub(crate) fn canonicalize_entity_type(value: &str) -> Option<String> {
    EntityType::parse(value).map(|entity_type| entity_type.as_str().to_string())
}

pub(crate) fn validate_minimum_nodes(
    entities: &[GraphNode],
    attempt: usize,
) -> Result<(), Vec<InvariantViolation>> {
    if entities.len() >= 2 {
        return Ok(());
    }

    Err(vec![InvariantViolation {
        code: InvariantCode::AtLeastTwoNodes,
        phase: InvariantPhase::EntityExtraction,
        message: format!(
            "expected at least 2 nodes after normalization, found {}",
            entities.len()
        ),
        entity_names: entities.iter().map(|entity| entity.name.clone()).collect(),
        edges: vec![],
        attempt,
    }])
}

#[cfg(test)]
pub(crate) fn validate_relationship_pairings(
    relationships: &[GraphEdge],
    entities: &[GraphNode],
    attempt: usize,
) -> Result<(), Vec<InvariantViolation>> {
    let (_, violations) = partition_relationship_pairings(relationships, entities, attempt);

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

pub(crate) fn partition_relationship_pairings(
    relationships: &[GraphEdge],
    entities: &[GraphNode],
    attempt: usize,
) -> (Vec<GraphEdge>, Vec<InvariantViolation>) {
    let entity_types: HashMap<&str, EntityType> = entities
        .iter()
        .filter_map(|entity| {
            EntityType::parse(&entity.entity_type).map(|entity_type| (entity.name.as_str(), entity_type))
        })
        .collect();

    let mut valid_relationships = Vec::new();
    let mut violations = Vec::new();

    for relationship in relationships {
        let violation = (|| {
            let keyword = RelationshipKeyword::parse(&relationship.keyword)?;
            let rule = RELATIONSHIP_RULES
                .iter()
                .find(|rule| rule.keyword == keyword)?;
            let source_type = entity_types.get(relationship.source.as_str())?;
            let target_type = entity_types.get(relationship.target.as_str())?;

            if rule.allows(*source_type, *target_type) {
                return None;
            }

            Some(InvariantViolation {
                code: InvariantCode::InvalidEdgePairing,
                phase: InvariantPhase::RelationshipExtraction,
                message: format!(
                    "{} cannot connect {} ({}) -> {} ({})",
                    relationship.keyword,
                    relationship.source,
                    source_type.as_str(),
                    relationship.target,
                    target_type.as_str()
                ),
                entity_names: vec![relationship.source.clone(), relationship.target.clone()],
                edges: vec![(
                    relationship.source.clone(),
                    relationship.target.clone(),
                    relationship.keyword.clone(),
                )],
                attempt,
            })
        })();

        match violation {
            Some(violation) => violations.push(violation),
            None => valid_relationships.push(relationship.clone()),
        }
    }

    (valid_relationships, violations)
}

pub(crate) fn validate_no_orphan_nodes(
    entities: &[GraphNode],
    relationships: &[GraphEdge],
    attempt: usize,
) -> Result<(), Vec<InvariantViolation>> {
    let connected_nodes: HashSet<&str> = relationships
        .iter()
        .flat_map(|relationship| [relationship.source.as_str(), relationship.target.as_str()])
        .collect();

    let orphan_nodes: Vec<String> = entities
        .iter()
        .filter(|entity| !connected_nodes.contains(entity.name.as_str()))
        .map(|entity| entity.name.clone())
        .collect();

    if orphan_nodes.is_empty() {
        return Ok(());
    }

    Err(vec![InvariantViolation {
        code: InvariantCode::NoNodeWithoutEdge,
        phase: InvariantPhase::GraphFinalization,
        message: format!("nodes without edges: {}", orphan_nodes.join(", ")),
        entity_names: orphan_nodes,
        edges: vec![],
        attempt,
    }])
}

pub(crate) fn relationship_rules_for_prompt() -> &'static str {
    r#"Allowed relationship pairings:
- WORKED_AT: AUTHOR|PERSON -> ORGANIZATION
- RELATED_TO: any -> any
- LOCATED_IN: AUTHOR|PERSON|ORGANIZATION|EVENT|PRODUCT|LIFEFORM -> LOCATION
- COLLABORATED_WITH: AUTHOR|PERSON -> AUTHOR|PERSON
- PART_OF: AUTHOR|PERSON|ORGANIZATION|LOCATION|CONCEPT|LIFEFORM -> ORGANIZATION|LOCATION|CONCEPT|LIFEFORM
- CREATED: AUTHOR|PERSON|ORGANIZATION -> PRODUCT|TECHNOLOGY|CONCEPT
- USES: AUTHOR|PERSON|ORGANIZATION|LIFEFORM -> TECHNOLOGY|PRODUCT
- IMPLEMENTS: TECHNOLOGY|PRODUCT -> CONCEPT
- PARTICIPATED_IN: AUTHOR|PERSON|ORGANIZATION -> EVENT
- OCCURRED_IN: EVENT -> LOCATION
- AFFILIATED_WITH: AUTHOR|PERSON -> ORGANIZATION
- MENTIONED_WITH: any -> any
- FOUNDED: AUTHOR|PERSON -> ORGANIZATION"#
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(name: &str, entity_type: &str) -> GraphNode {
        GraphNode {
            name: name.to_string(),
            entity_type: entity_type.to_string(),
            description: "desc".to_string(),
        }
    }

    fn edge(source: &str, target: &str, keyword: &str) -> GraphEdge {
        GraphEdge {
            source: source.to_string(),
            target: target.to_string(),
            keyword: keyword.to_string(),
            description: "desc".to_string(),
        }
    }

    #[test]
    fn canonicalize_entity_type_accepts_author() {
        assert_eq!(canonicalize_entity_type("author"), Some("AUTHOR".to_string()));
    }

    #[test]
    fn validate_relationship_pairings_rejects_invalid_author_pairing() {
        let entities = vec![node("Ada", "AUTHOR"), node("Paris", "LOCATION")];
        let relationships = vec![edge("Ada", "Paris", "WORKED_AT")];

        let result = validate_relationship_pairings(&relationships, &entities, 0);

        assert!(result.is_err());
    }

    #[test]
    fn validate_no_orphan_nodes_detects_unconnected_entities() {
        let entities = vec![node("Ada", "AUTHOR"), node("OpenAI", "ORGANIZATION")];
        let relationships = vec![];

        let result = validate_no_orphan_nodes(&entities, &relationships, 0);

        assert!(result.is_err());
    }
}
