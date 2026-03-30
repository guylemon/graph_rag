mod domain;

use graphqlite::Graph;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Read};

use crate::domain::*;
use llm_msg::Message;
use llm_msg::Role;
use llm_provider::{ChatRequest, Format, Provider};

pub fn run() -> Result<(), AppError> {
    // Read input
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Extract entity mentions
    let ollama_config = llm_provider::Config::new(Some("http://studio:11434/api"));
    let provider = Provider::Ollama(ollama_config);
    let entities = extract_entity_mentions(&provider, &input)?;
    let relationships = extract_relationship_mentions(&provider, &input, &entities)?;

    // ----- Graph operations
    let g = Graph::open(":memory:")?;

    // Add nodes
    let mut entity_set: HashSet<String> = HashSet::new();
    for entity in entities[0..].iter() {
        entity_set.insert(entity.entity_name.to_owned());
        g.upsert_node(
            &entity.entity_name,
            [("description", &entity.entity_description)],
            &entity.entity_type,
        )?;
    }

    let mut validated_relationships: Vec<&ExtractedRelationship> = Vec::new();
    for edge in relationships[0..].iter() {
        // filter invalid edges from LLM
        if entity_set.contains(&edge.source_entity) && entity_set.contains(&edge.target_entity) {
            g.upsert_edge(
                &edge.source_entity,
                &edge.target_entity,
                [("description", &edge.relationship_description)],
                &edge.relationship_description,
            )?;
            validated_relationships.push(edge);
        } else {
            println!(
                "Invalid edge from {} -> {}",
                &edge.source_entity, &edge.target_entity
            );
            continue;
        }
    }

    // transform and export for terminal display
    let nodes: Vec<NodeExport> = entities
        .iter()
        .map(|n| NodeExport {
            id: n.entity_name.to_owned(),
            value: n.entity_description.to_owned(),
        })
        .collect();

    let edges: Vec<EdgeExport> = validated_relationships
        .iter()
        .map(|r| EdgeExport {
            id: r.source_entity.to_owned(),
            source: r.source_entity.to_owned(),
            target: r.target_entity.to_owned(),
        })
        .collect();

    // Serialize and write to file
    let export = GraphExport {
        version: 1,
        nodes,
        edges,
    };
    let json = serde_json::to_string_pretty(&export)?;

    fs::write("./output/graph.json", json)?;
    println!("Graph exported to graph.json");

    // Transform and export for Cytoscape.js web viewer
    let mut cytoscape_elements: Vec<CytoscapeElementExport> = entities
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
                label: Some(r.relationship_description.to_owned()),
                entity_type: None,
                description: None,
                source: Some(r.source_entity.to_owned()),
                target: Some(r.target_entity.to_owned()),
            },
        }
    }));

    let cytoscape_export = CytoscapeGraphExport {
        elements: cytoscape_elements,
    };
    let cytoscape_json = serde_json::to_string_pretty(&cytoscape_export)?;
    fs::write("./cytoscape/data.json", cytoscape_json)?;
    println!("Graph exported to cytoscape/data.json");

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

fn extract_entity_mentions(provider: &Provider, input: &str) -> Result<Vec<ExtractedEntity>, AppError> {
    let sys_content = fs::read_to_string("./prompts/entity_identification_sys.txt")?;
    let sys_prompt = Message::new(Role::System, &sys_content);
    let template = fs::read_to_string("./prompts/entity_identification_user.txt")?;
    let mut variables = HashMap::new();
    variables.insert("input_text".to_string(), input.to_owned());
    let user_prompt = Message::new(Role::User, &llm_prompt::substitute(&template, &variables)?);
    let schema_raw = fs::read_to_string("./prompts/entity_extraction_schema.json")?;
    let format: Format = Format::Schema(serde_json::from_str(&schema_raw)?);
    let chat_request = ChatRequest::builder("qwen3:8b")
        .add_message(sys_prompt)
        .add_message(user_prompt)
        .format(format)
        .stream(false)
        .temperature(0.0)
        .build()?;

    let response = llm_generate::generate(&chat_request, provider)?;
    // parse to domain type
    let extraction: EntityExtractionOutput = serde_json::from_str(&response.content)?;

    Ok(extraction.entities)
}

fn extract_relationship_mentions(provider: &Provider, input: &str, entities: &Vec<ExtractedEntity>) -> Result<Vec<ExtractedRelationship>, AppError> {
    let sys_content = fs::read_to_string("./prompts/relationship_identification_sys.txt")?;
    let sys_prompt = Message::new(Role::System, &sys_content);
    let template = fs::read_to_string("./prompts/relationship_identification_user.txt")?;
    let mut variables = HashMap::new();
    let entities = serde_json::to_string(entities)?;
    variables.insert("input_text".to_string(), input.to_owned());
    variables.insert("entities".to_string(), entities.to_owned());
    let user_prompt = Message::new(Role::User, &llm_prompt::substitute(&template, &variables)?);
    let schema_raw = fs::read_to_string("./prompts/entity_relationship_schema.json")?;
    let format: Format = Format::Schema(serde_json::from_str(&schema_raw)?);
    let chat_request = ChatRequest::builder("granite4:1b")
        .add_message(sys_prompt)
        .add_message(user_prompt)
        .format(format)
        .stream(false)
        .temperature(0.0)
        .build()?;

    let response = llm_generate::generate(&chat_request, provider)?;
    // parse to domain type
    let extraction: RelationshipExtractionOutput = serde_json::from_str(&response.content)?;

    Ok(extraction.relationships)
}
