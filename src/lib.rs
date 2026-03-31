mod domain;
mod service;

use graphqlite::Graph;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Read};

use crate::domain::*;
use crate::service::KnowledgeGraphExtractor;

pub fn run() -> Result<(), AppError> {
    let app_config = AppConfig::new();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let knowledge_graph = KnowledgeGraphExtractor::new(app_config.clone()).execute(&input)?;

    // ----- Graph operations
    let g = Graph::open(&app_config.graphqlite_db)?;

    // Add nodes
    let mut entity_set: HashSet<String> = HashSet::new();
    for entity in knowledge_graph.entities[0..].iter() {
        entity_set.insert(entity.entity_name.to_owned());
        g.upsert_node(
            &entity.entity_name,
            [("description", &entity.entity_description)],
            &entity.entity_type,
        )?;
    }

    let mut validated_relationships: Vec<ValidatedExtractedRelationship> = Vec::new();
    for edge in knowledge_graph.relationships[0..].iter() {
        // filter invalid edges from LLM
        if entity_set.contains(&edge.source_entity) && entity_set.contains(&edge.target_entity) {
            for keyword in edge.relationship_keywords[0..].iter() {
                g.upsert_edge(
                    &edge.source_entity,
                    &edge.target_entity,
                    [("description", &edge.relationship_description)],
                    keyword,
                )?;

                let validated = ValidatedExtractedRelationship {
                    source_entity: edge.source_entity.to_owned(),
                    target_entity: edge.target_entity.to_owned(),
                    relationship_description: edge.relationship_description.to_owned(),
                    keyword: keyword.to_owned(),
                };

                validated_relationships.push(validated);
            }
        } else {
            println!(
                "Invalid edge from {} -> {}",
                &edge.source_entity, &edge.target_entity
            );
            continue;
        }
    }

    // Transform and export for Cytoscape.js web viewer
    let mut cytoscape_elements: Vec<CytoscapeElementExport> = knowledge_graph
        .entities
        .iter()
        .map(|n| CytoscapeElementExport {
            data: CytoscapeDataExport {
                id: n.entity_name.to_owned(),
                label: Some(n.entity_name.to_owned()),
                entity_type: Some(n.entity_type.to_owned()),
                description: Some(n.entity_description.to_owned()),
                source: None,
                target: None,
            },
        })
        .collect();

    cytoscape_elements.extend(validated_relationships.iter().enumerate().map(|(i, r)| {
        CytoscapeElementExport {
            data: CytoscapeDataExport {
                id: format!("edge-{i}"),
                label: Some(r.keyword.to_owned()),
                entity_type: None,
                description: Some(r.relationship_description.to_owned()),
                source: Some(r.source_entity.to_owned()),
                target: Some(r.target_entity.to_owned()),
            },
        }
    }));

    let cytoscape_export = CytoscapeGraphExport {
        elements: cytoscape_elements,
    };
    let cytoscape_json = serde_json::to_string_pretty(&cytoscape_export)?;
    fs::write(&app_config.cytoscape_json_file, cytoscape_json)?;
    println!("Graph exported to {}", &app_config.cytoscape_json_file);

    // // Query
    // println!("{:?}", g.stats()?); // GraphStats { nodes: 2, edges: 1 }
    // println!("{:?}", g.get_neighbors("APPLE")?);
    // //
    // // Graph algorithms
    // let ranks = g.pagerank(0.85, 20)?;
    // let communities = g.community_detection(10)?;
    //
    // dbg!(ranks);
    // dbg!(communities);

    Ok(())
}
