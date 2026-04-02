use std::collections::HashSet;

use crate::domain::*;
use crate::ports::EntityExtractionPort;
use crate::ports::RelationshipExtractionPort;

const MAX_REPAIR_ATTEMPTS: usize = 2;

pub(crate) struct KnowledgeGraphExtractor<E, R> {
    entity_extractor: E,
    relationship_extractor: R,
}

impl<E, R> KnowledgeGraphExtractor<E, R>
where
    E: EntityExtractionPort<Error = AppError>,
    R: RelationshipExtractionPort<Error = AppError>,
{
    pub(crate) fn new(entity_extractor: E, relationship_extractor: R) -> Self {
        Self {
            entity_extractor,
            relationship_extractor,
        }
    }

    pub(crate) fn execute(&self, input: &str) -> Result<KnowledgeGraph, AppError> {
        let entities = self.extract_nodes_with_repair(input)?;
        let relationships = self.extract_relationships_with_repair(input, &entities)?;

        self.finalize_graph_with_repair(input, entities, relationships)
    }

    fn extract_nodes_with_repair(&self, input: &str) -> Result<Vec<GraphNode>, AppError> {
        let mut repair_context = None;

        for attempt in 0..=MAX_REPAIR_ATTEMPTS {
            let entity_request = EntityExtractionRequest {
                input,
                repair_context: repair_context.clone(),
            };
            let raw_entities = self.entity_extractor.extract_entities(entity_request)?;
            let nodes = normalize_entities(raw_entities);

            match validate_minimum_nodes(&nodes, attempt) {
                Ok(()) => return Ok(nodes),
                Err(violations) if attempt == MAX_REPAIR_ATTEMPTS => {
                    return Err(Box::new(InvariantError { violations }));
                }
                Err(violations) => {
                    repair_context = Some(build_entity_repair_context(&violations));
                }
            }
        }

        unreachable!("entity extraction loop should always return")
    }

    fn extract_relationships_with_repair(
        &self,
        input: &str,
        entities: &[GraphNode],
    ) -> Result<Vec<GraphEdge>, AppError> {
        let mut repair_context = None;

        for attempt in 0..=MAX_REPAIR_ATTEMPTS {
            let relationships = self.extract_candidate_relationships(
                input,
                entities,
                repair_context.clone(),
            )?;

            let (valid_relationships, violations) =
                partition_relationship_pairings(&relationships, entities, attempt);

            if violations.is_empty() {
                return Ok(valid_relationships);
            }

            if attempt == MAX_REPAIR_ATTEMPTS {
                return Err(Box::new(InvariantError { violations }));
            }

            repair_context = Some(build_relationship_repair_context(&violations));
        }

        unreachable!("relationship extraction loop should always return")
    }

    fn finalize_graph_with_repair(
        &self,
        input: &str,
        entities: Vec<GraphNode>,
        mut relationships: Vec<GraphEdge>,
    ) -> Result<KnowledgeGraph, AppError> {
        for attempt in 0..=MAX_REPAIR_ATTEMPTS {
            match validate_no_orphan_nodes(&entities, &relationships, attempt) {
                Ok(()) => {
                    return Ok(KnowledgeGraph {
                        entities,
                        relationships,
                    });
                }
                Err(violations) if attempt == MAX_REPAIR_ATTEMPTS => {
                    return Err(Box::new(InvariantError { violations }));
                }
                Err(violations) => {
                    let repair_context = build_orphan_repair_context(&violations, &relationships);
                    let repaired_edges =
                        self.extract_candidate_relationships(input, &entities, Some(repair_context))?;
                    let (valid_repaired_edges, pairing_violations) =
                        partition_relationship_pairings(&repaired_edges, &entities, attempt);

                    if !pairing_violations.is_empty() && attempt == MAX_REPAIR_ATTEMPTS {
                        return Err(Box::new(InvariantError {
                            violations: pairing_violations,
                        }));
                    }

                    relationships = merge_relationships(relationships, valid_repaired_edges);
                }
            }
        }

        unreachable!("graph finalization loop should always return")
    }

    fn extract_candidate_relationships(
        &self,
        input: &str,
        entities: &[GraphNode],
        repair_context: Option<String>,
    ) -> Result<Vec<GraphEdge>, AppError> {
        let relationship_request = RelationshipExtractionRequest {
            input,
            entities,
            repair_context,
            allowed_rules: Some(relationship_rules_for_prompt().to_string()),
        };
        let raw_relationships = self
            .relationship_extractor
            .extract_relationships(relationship_request)?;

        let cleaned_relationships = retain_known_entity_relationships(
            normalize_relationship_mentions(raw_relationships),
            entities,
        );

        Ok(expand_relationships(cleaned_relationships))
    }
}

fn merge_relationships(
    existing_relationships: Vec<GraphEdge>,
    new_relationships: Vec<GraphEdge>,
) -> Vec<GraphEdge> {
    let mut seen = HashSet::new();

    existing_relationships
        .into_iter()
        .chain(new_relationships)
        .filter(|relationship| seen.insert(relationship.clone()))
        .collect()
}

fn build_entity_repair_context(violations: &[InvariantViolation]) -> String {
    let current_entities = violations
        .iter()
        .flat_map(|violation| violation.entity_names.iter())
        .cloned()
        .collect::<Vec<_>>();

    format!(
        "Previous extraction violated AT_LEAST_TWO_NODES. Return at least two valid entities from the text. Current normalized entities: {}.",
        if current_entities.is_empty() {
            "none".to_string()
        } else {
            current_entities.join(", ")
        }
    )
}

fn build_relationship_repair_context(violations: &[InvariantViolation]) -> String {
    let invalid_edges = violations
        .iter()
        .map(|violation| violation.message.as_str())
        .collect::<Vec<_>>()
        .join("; ");

    format!(
        "Previous extraction produced invalid relationship pairings. Correct them using only allowed source/target type pairs. Invalid items: {}.",
        invalid_edges
    )
}

fn build_orphan_repair_context(
    violations: &[InvariantViolation],
    relationships: &[GraphEdge],
) -> String {
    let orphan_nodes = violations
        .iter()
        .flat_map(|violation| violation.entity_names.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");

    let existing_edges = if relationships.is_empty() {
        "none".to_string()
    } else {
        relationships
            .iter()
            .map(|relationship| {
                format!(
                    "{} -[{}]-> {}",
                    relationship.source, relationship.keyword, relationship.target
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    };

    format!(
        "Previous graph left orphan nodes without any edge. Add only valid edges supported by the text for these nodes: {}. Existing edges: {}. Return no relationships if the text does not support a valid edge.",
        orphan_nodes, existing_edges
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct StubEntityExtractor {
        responses: Mutex<Vec<Vec<EntityMention>>>,
    }

    impl StubEntityExtractor {
        fn new(responses: Vec<Vec<EntityMention>>) -> Self {
            Self {
                responses: Mutex::new(responses),
            }
        }
    }

    impl EntityExtractionPort for StubEntityExtractor {
        type Error = AppError;

        fn extract_entities(
            &self,
            _req: EntityExtractionRequest,
        ) -> Result<EntityExtractionResponse, Self::Error> {
            Ok(self.responses.lock().unwrap().remove(0))
        }
    }

    struct StubRelationshipExtractor {
        responses: Mutex<Vec<Vec<RelationshipMention>>>,
    }

    impl StubRelationshipExtractor {
        fn new(responses: Vec<Vec<RelationshipMention>>) -> Self {
            Self {
                responses: Mutex::new(responses),
            }
        }
    }

    impl RelationshipExtractionPort for StubRelationshipExtractor {
        type Error = AppError;

        fn extract_relationships(
            &self,
            _req: RelationshipExtractionRequest,
        ) -> Result<RelationshipExtractionResponse, Self::Error> {
            Ok(self.responses.lock().unwrap().remove(0))
        }
    }

    fn entity(name: &str, entity_type: &str) -> EntityMention {
        EntityMention {
            entity_name: name.to_string(),
            entity_type: entity_type.to_string(),
            entity_description: "desc".to_string(),
        }
    }

    fn relationship(source: &str, target: &str, keyword: &str) -> RelationshipMention {
        RelationshipMention {
            source: source.to_string(),
            target: target.to_string(),
            keywords: vec![keyword.to_string()],
            description: "desc".to_string(),
        }
    }

    #[test]
    fn execute_retries_entity_extraction_until_two_nodes_exist() {
        let entity_extractor = StubEntityExtractor::new(vec![
            vec![entity("Ada", "AUTHOR")],
            vec![entity("Ada", "AUTHOR"), entity("OpenAI", "ORGANIZATION")],
        ]);
        let relationship_extractor =
            StubRelationshipExtractor::new(vec![vec![relationship("Ada", "OpenAI", "WORKED_AT")]]);

        let graph = KnowledgeGraphExtractor::new(entity_extractor, relationship_extractor)
            .execute("Ada worked at OpenAI")
            .unwrap();

        assert_eq!(graph.entities.len(), 2);
        assert_eq!(graph.relationships.len(), 1);
    }

    #[test]
    fn execute_retries_invalid_relationship_pairings() {
        let entity_extractor = StubEntityExtractor::new(vec![vec![
            entity("Ada", "AUTHOR"),
            entity("OpenAI", "ORGANIZATION"),
        ]]);
        let relationship_extractor = StubRelationshipExtractor::new(vec![
            vec![relationship("Ada", "OpenAI", "LOCATED_IN")],
            vec![relationship("Ada", "OpenAI", "WORKED_AT")],
        ]);

        let graph = KnowledgeGraphExtractor::new(entity_extractor, relationship_extractor)
            .execute("Ada worked at OpenAI")
            .unwrap();

        assert_eq!(graph.relationships[0].keyword, "WORKED_AT");
    }

    #[test]
    fn execute_fails_when_orphan_nodes_remain_after_repairs() {
        let entity_extractor = StubEntityExtractor::new(vec![vec![
            entity("Ada", "AUTHOR"),
            entity("OpenAI", "ORGANIZATION"),
        ]]);
        let relationship_extractor = StubRelationshipExtractor::new(vec![vec![], vec![], vec![]]);

        let err = KnowledgeGraphExtractor::new(entity_extractor, relationship_extractor)
            .execute("Ada wrote about OpenAI")
            .unwrap_err();

        assert!(err.to_string().contains("NO_NODE_WITHOUT_EDGE"));
    }
}
