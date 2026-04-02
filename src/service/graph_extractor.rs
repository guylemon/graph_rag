use crate::domain::*;
use crate::ports::EntityExtractionPort;
use crate::ports::RelationshipExtractionPort;

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
        let entities = self.extract_nodes(input)?;
        let relationships = self.extract_relationships(input, &entities)?;

        Ok(KnowledgeGraph {
            entities,
            relationships,
        })
    }

    fn extract_nodes(&self, input: &str) -> Result<Vec<GraphNode>, AppError> {
        let entity_request = EntityExtractionRequest { input };
        let raw_entities: Vec<EntityMention> =
            self.entity_extractor.extract_entities(entity_request)?;

        let nodes = normalize_entities(raw_entities);

        Ok(nodes)
    }

    fn extract_relationships(
        &self,
        input: &str,
        entities: &Vec<GraphNode>,
    ) -> Result<Vec<GraphEdge>, AppError> {
        let relationship_request = RelationshipExtractionRequest {
            input,
            entities: &entities,
        };
        let raw_relationships = self
            .relationship_extractor
            .extract_relationships(relationship_request)?;

        let relationships = normalize_relationships(raw_relationships, &nodes);

        Ok(relationships)
    }
}
