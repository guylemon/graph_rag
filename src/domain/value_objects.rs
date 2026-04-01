use crate::ExtractedEntity;
use crate::RelationshipMention;

pub(crate) struct EntityExtractionRequest<'a> {
    pub(crate) input: &'a str,
}

// TODO refactor as DTO if needed later
pub(crate) type EntityExtractionResponse = Vec<ExtractedEntity>;

pub(crate) struct RelationshipExtractionRequest<'a, 'e> {
    pub(crate) input: &'a str,
    pub(crate) entities: &'e Vec<ExtractedEntity>,
}

// TODO refactor as DTO if needed later
pub(crate) type RelationshipExtractionResponse = Vec<RelationshipMention>;
