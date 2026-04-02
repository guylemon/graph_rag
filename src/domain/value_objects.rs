use crate::domain::GraphNode;
use crate::domain::EntityMention;
use crate::domain::RelationshipMention;

pub(crate) struct EntityExtractionRequest<'a> {
    pub(crate) input: &'a str,
    pub(crate) repair_context: Option<String>,
}

// TODO refactor as DTO if needed later
pub(crate) type EntityExtractionResponse = Vec<EntityMention>;

pub(crate) struct RelationshipExtractionRequest<'a, 'e> {
    pub(crate) input: &'a str,
    pub(crate) entities: &'e [GraphNode],
    pub(crate) repair_context: Option<String>,
    pub(crate) allowed_rules: Option<String>,
}

// TODO refactor as DTO if needed later
pub(crate) type RelationshipExtractionResponse = Vec<RelationshipMention>;
