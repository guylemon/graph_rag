use graphqlite::Graph;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};

use llm_msg::Message;
use llm_msg::Role;
use llm_provider::{ChatRequest, Format, Provider};

type AppError = Box<dyn std::error::Error>;

#[derive(Debug, Deserialize, Clone)]
pub struct ExtractionOutput {
    pub entities: Vec<ExtractedEntity>,
    pub relationships: Vec<ExtractedRelationship>,
}

#[derive(Debug, Deserialize, Clone)]
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
    pub relationship_strength: u8,
}

pub fn run() -> Result<(), AppError> {
    // Read input
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Extract entity mentions
    let ollama_config = llm_provider::Config::new(Some("http://studio:11434/api"));
    let provider = Provider::Ollama(ollama_config);
    let sys_content = fs::read_to_string("./prompts/entity_identification_sys.txt")?;
    let sys_prompt = Message::new(Role::System, &sys_content);
    let template = fs::read_to_string("./prompts/entity_identification_user.txt")?;
    let mut variables = HashMap::new();
    variables.insert("input_text".to_string(), input);
    let user_prompt = Message::new(Role::User, &llm_prompt::substitute(&template, &variables)?);
    let schema_raw = fs::read_to_string("./prompts/entity_extraction_schema.json")?;
    let format: Format = Format::Schema(serde_json::from_str(&schema_raw)?);
    let chat_request = ChatRequest::builder("granite4:1b")
        .add_message(sys_prompt)
        .add_message(user_prompt)
        .format(format)
        .stream(false)
        .temperature(0.0)
        .build()?;

    let response = llm_generate::generate(&chat_request, &provider)?;
    // parse to domain type
    let extraction: ExtractionOutput = serde_json::from_str(&response.content)?;

    // ----- Graph operations
    let g = Graph::open(":memory:")?;

    // Add nodes
    for entity in extraction.entities {
        g.upsert_node(
            &entity.entity_name,
            [("description", entity.entity_description)],
            &entity.entity_type,
        )?;
    }

    // Add edges
    for edge in extraction.relationships {
        g.upsert_edge(
            &edge.source_entity,
            &edge.target_entity,
            [("description", &edge.relationship_description)],
            &edge.relationship_description,
        )?;
    }

    // Query
    println!("{:?}", g.stats()?); // GraphStats { nodes: 2, edges: 1 }
    println!("{:?}", g.get_neighbors("APPLE")?);
    //
    // Graph algorithms
    let ranks = g.pagerank(0.85, 20)?;
    let communities = g.community_detection(10)?;

    dbg!(ranks);
    dbg!(communities);

    Ok(())
}
