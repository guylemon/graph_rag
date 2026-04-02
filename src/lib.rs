mod adapters;
mod domain;
mod ports;
mod service;

use graphqlite::Graph;
use llm_provider::Provider;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Read};

use crate::adapters::OllamaEntityExtractor;
use crate::adapters::OllamaRelationshipExtractor;
use crate::domain::*;
use crate::service::KnowledgeGraphExtractor;

pub fn run() -> Result<(), AppError> {
    let app_config = AppConfig::new();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let ollama_config = llm_provider::Config::new(Some(&app_config.ollama_base_url));
    let ollama_provider = Provider::Ollama(ollama_config);
    let entity_extractor = OllamaEntityExtractor::new(&app_config, &ollama_provider)?;
    let relationship_extractor = OllamaRelationshipExtractor::new(&app_config, &ollama_provider)?;
    let knowledge_graph =
        KnowledgeGraphExtractor::new(entity_extractor, relationship_extractor).execute(&input)?;

    // ----- Graph operations
    let g = Graph::open(&app_config.graphqlite_db)?;

    // Add nodes
    let mut entity_set: HashSet<String> = HashSet::new();
    for entity in knowledge_graph.entities[0..].iter() {
        entity_set.insert(entity.name.to_owned());
        g.upsert_node(
            &entity.name,
            [("description", &entity.description)],
            &entity.entity_type,
        )?;
    }

    for edge in knowledge_graph.relationships[0..].iter() {
        g.upsert_edge(
            &edge.source,
            &edge.target,
            [("description", &edge.description)],
            &edge.keyword,
        )?;
    }

    // Transform and export for Cytoscape.js web viewer
    let mut cytoscape_elements: Vec<CytoscapeElementExport> = knowledge_graph
        .entities
        .iter()
        .map(|n| CytoscapeElementExport {
            data: CytoscapeDataExport {
                id: n.name.to_owned(),
                label: Some(n.name.to_owned()),
                entity_type: Some(n.entity_type.to_owned()),
                description: Some(n.description.to_owned()),
                source: None,
                target: None,
            },
        })
        .collect();

    cytoscape_elements.extend(
        knowledge_graph
            .relationships
            .iter()
            .enumerate()
            .map(|(i, r)| CytoscapeElementExport {
                data: CytoscapeDataExport {
                    id: format!("edge-{i}"),
                    label: Some(r.keyword.to_owned()),
                    entity_type: None,
                    description: Some(r.description.to_owned()),
                    source: Some(r.source.to_owned()),
                    target: Some(r.target.to_owned()),
                },
            }),
    );

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
