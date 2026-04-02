use crate::EntityMention;
use crate::RelationshipMention;
use crate::domain::GraphNode;

pub(crate) struct EntityExtractionRequest<'a> {
    pub(crate) input: &'a str,
}

// TODO refactor as DTO if needed later
pub(crate) type EntityExtractionResponse = Vec<EntityMention>;

pub(crate) struct RelationshipExtractionRequest<'a, 'e> {
    pub(crate) input: &'a str,
    pub(crate) entities: &'e Vec<GraphNode>,
}

// TODO refactor as DTO if needed later
pub(crate) type RelationshipExtractionResponse = Vec<RelationshipMention>;
