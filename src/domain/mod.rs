mod config;
mod value_objects;

use serde::Deserialize;
use serde::Serialize;

pub(crate) use config::AppConfig;
pub(crate) use value_objects::*;

pub type AppError = Box<dyn std::error::Error>;

// TODO this domain object has invariants
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct KnowledgeGraph {
    pub(crate) entities: Vec<ExtractedEntity>,
    pub(crate) relationships: Vec<ExtractedRelationship>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EntityExtractionOutput {
    pub entities: Vec<ExtractedEntity>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct RelationshipExtractionOutput {
    pub relationships: Vec<ExtractedRelationship>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtractedEntity {
    pub entity_name: String,
    pub entity_type: String,
    pub entity_description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExtractedRelationship {
    pub source_entity: String,
    pub target_entity: String,
    pub relationship_keywords: Vec<String>,
    pub relationship_description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidatedExtractedRelationship {
    pub source_entity: String,
    pub target_entity: String,
    pub keyword: String,
    pub relationship_description: String,
}

#[derive(Serialize)]
pub struct CytoscapeDataExport {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Serialize)]
pub struct CytoscapeElementExport {
    pub data: CytoscapeDataExport,
}

#[derive(Serialize)]
pub struct CytoscapeGraphExport {
    pub elements: Vec<CytoscapeElementExport>,
}
