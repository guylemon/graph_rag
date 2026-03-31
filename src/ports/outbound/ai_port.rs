use crate::domain::EntityExtractionRequest;
use crate::domain::EntityExtractionResponse;
use crate::domain::RelationshipExtractionRequest;
use crate::domain::RelationshipExtractionResponse;

pub(crate) trait EntityExtractionPort: Send + Sync {
    type Error;
    fn extract_entities(
        &self,
        req: EntityExtractionRequest,
    ) -> Result<EntityExtractionResponse, Self::Error>;
}

pub(crate) trait RelationshipExtractionPort: Send + Sync {
    type Error;
    fn extract_relationships(
        &self,
        req: RelationshipExtractionRequest,
    ) -> Result<RelationshipExtractionResponse, Self::Error>;
}
