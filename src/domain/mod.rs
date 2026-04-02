mod config;
mod entities;
mod invariants;
mod relationships;
mod util;
mod value_objects;

use serde::Deserialize;
use serde::Serialize;

pub(crate) use config::AppConfig;
pub(crate) use entities::*;
pub(crate) use invariants::*;
pub(crate) use relationships::*;
pub(crate) use value_objects::*;

pub type AppError = Box<dyn std::error::Error>;

// TODO this domain object has invariants
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct KnowledgeGraph {
    pub(crate) entities: Vec<GraphNode>,
    pub(crate) relationships: Vec<GraphEdge>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EntityExtractionOutput {
    pub entities: Vec<EntityMention>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct RelationshipExtractionOutput {
    pub relationships: Vec<RelationshipMention>,
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
