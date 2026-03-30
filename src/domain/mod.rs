use serde::Deserialize;
use serde::Serialize;

pub type AppError = Box<dyn std::error::Error>;

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

#[derive(Debug, Deserialize, Clone)]
pub struct ExtractedRelationship {
    pub source_entity: String,
    pub target_entity: String,
    pub relationship_description: String,
}

#[derive(Serialize)]
pub struct NodeExport {
    pub id: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct EdgeExport {
    pub id: String,
    pub source: String,
    pub target: String,
}

// Using cosmo-flow (npm) for now
// {
//   "version": 1,
//   "nodes": [
//     { "id": "b1", "value": "Root" },
//     { "id": "b2", "value": "L1 - L" },
//     { "id": "b3", "value": "L1 - R" }
//   ],
//   "edges": [
//     { "id": "be1", "source": "b1", "target": "b2" },
//     { "id": "be2", "source": "b1", "target": "b3" }
//   ]
// }
#[derive(Serialize)]
pub struct GraphExport {
    pub version: u8,
    pub nodes: Vec<NodeExport>,
    pub edges: Vec<EdgeExport>,
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
