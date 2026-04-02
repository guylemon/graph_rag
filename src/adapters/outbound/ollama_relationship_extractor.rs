use std::collections::HashMap;
use std::fs;

use llm_msg::Message;
use llm_msg::Role;
use llm_provider::Provider;
use llm_provider::{ChatRequest, Format};

use crate::domain::AppConfig;
use crate::domain::AppError;
use crate::domain::RelationshipExtractionOutput;
use crate::domain::RelationshipExtractionRequest;
use crate::domain::RelationshipExtractionResponse;
use crate::ports::RelationshipExtractionPort;

pub(crate) struct OllamaRelationshipExtractor {
    config: AppConfig,
    provider: Provider,
}

impl OllamaRelationshipExtractor {
    pub(crate) fn new(config: &AppConfig, provider: &Provider) -> Result<Self, AppError> {
        match provider {
            Provider::Ollama(_) => Ok(Self {
                config: config.clone(),
                provider: provider.clone(),
            }),
            _ => Err("Invalid LLM provider. Provider must be Ollama".into()),
        }
    }
}

impl RelationshipExtractionPort for OllamaRelationshipExtractor {
    type Error = AppError;

    fn extract_relationships(
        &self,
        req: RelationshipExtractionRequest,
    ) -> Result<RelationshipExtractionResponse, Self::Error> {
        let app_config = &self.config;

        let sys_content = fs::read_to_string(&app_config.rel_id_sys_prompt)?;
        let sys_prompt = Message::new(Role::System, &sys_content);
        let template = fs::read_to_string(&app_config.rel_id_user_prompt)?;
        let mut variables = HashMap::new();
        let entities = serde_json::to_string(&req.entities)?;
        variables.insert("input_text".to_string(), req.input.to_owned());
        variables.insert("entities".to_string(), entities.to_owned());
        variables.insert(
            "repair_context".to_string(),
            req.repair_context.unwrap_or_default(),
        );
        variables.insert(
            "allowed_rules".to_string(),
            req.allowed_rules.unwrap_or_default(),
        );
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

        let response = llm_generate::generate(&chat_request, &self.provider)?;
        // parse to domain type
        let extraction: RelationshipExtractionOutput = serde_json::from_str(&response.content)?;

        Ok(extraction.relationships)
    }
}
