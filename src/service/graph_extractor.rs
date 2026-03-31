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
        let entity_request = EntityExtractionRequest { input };
        let entities = self.entity_extractor.extract_entities(entity_request)?;
        let relationship_request = RelationshipExtractionRequest {
            input,
            entities: &entities,
        };
        let relationships = self
            .relationship_extractor
            .extract_relationships(relationship_request)?;

        Ok(KnowledgeGraph {
            entities,
            relationships,
        })
    }
}
