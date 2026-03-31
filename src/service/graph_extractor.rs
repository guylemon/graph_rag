use std::collections::HashMap;
use std::fs;

use crate::domain::*;
use llm_msg::Message;
use llm_msg::Role;
use llm_provider::{ChatRequest, Format, Provider};

pub(crate) struct KnowledgeGraphExtractor {
    pub(crate) app_config: AppConfig,
}

impl KnowledgeGraphExtractor {
    pub(crate) fn new(app_config: AppConfig) -> Self {
        Self { app_config }
    }
}

impl KnowledgeGraphExtractor {
    pub(crate) fn execute(&self, input: &str) -> Result<KnowledgeGraph, AppError> {
        let cfg = &self.app_config;

        // Extract entity mentions
        let ollama_config = llm_provider::Config::new(Some(&cfg.ollama_base_url));
        let provider = Provider::Ollama(ollama_config);
        let entities = extract_entity_mentions(cfg, &provider, input)?;
        let relationships = extract_relationship_mentions(cfg, &provider, input, &entities)?;

        Ok(KnowledgeGraph {
            entities,
            relationships,
        })
    }
}

fn extract_entity_mentions(
    app_config: &AppConfig,
    provider: &Provider,
    input: &str,
) -> Result<Vec<ExtractedEntity>, AppError> {
    let sys_content = fs::read_to_string(&app_config.entity_id_sys_prompt)?;
    let sys_prompt = Message::new(Role::System, &sys_content);
    let template = fs::read_to_string(&app_config.entity_id_user_prompt)?;
    let mut variables = HashMap::new();
    variables.insert("input_text".to_string(), input.to_owned());
    let user_prompt = Message::new(Role::User, &llm_prompt::substitute(&template, &variables)?);
    let schema_raw = fs::read_to_string(&app_config.entity_id_llm_schema)?;
    let format: Format = Format::Schema(serde_json::from_str(&schema_raw)?);
    let chat_request = ChatRequest::builder(&app_config.entity_id_llm_model)
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

fn extract_relationship_mentions(
    app_config: &AppConfig,
    provider: &Provider,
    input: &str,
    entities: &Vec<ExtractedEntity>,
) -> Result<Vec<ExtractedRelationship>, AppError> {
    let sys_content = fs::read_to_string(&app_config.rel_id_sys_prompt)?;
    let sys_prompt = Message::new(Role::System, &sys_content);
    let template = fs::read_to_string(&app_config.rel_id_user_prompt)?;
    let mut variables = HashMap::new();
    let entities = serde_json::to_string(entities)?;
    variables.insert("input_text".to_string(), input.to_owned());
    variables.insert("entities".to_string(), entities.to_owned());
    let user_prompt = Message::new(Role::User, &llm_prompt::substitute(&template, &variables)?);
    let schema_raw = fs::read_to_string(&app_config.rel_id_llm_schema)?;
    let format: Format = Format::Schema(serde_json::from_str(&schema_raw)?);
    let chat_request = ChatRequest::builder(&app_config.rel_id_llm_model)
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
