#[derive(Clone, Debug)]
pub(crate) struct AppConfig {
    pub(crate) ollama_base_url: String,
    pub(crate) graphqlite_db: String,
    pub(crate) cytoscape_json_file: String,
    pub(crate) entity_id_sys_prompt: String,
    pub(crate) entity_id_user_prompt: String,
    pub(crate) entity_id_llm_schema: String,
    pub(crate) entity_id_llm_model: String,
    pub(crate) rel_id_sys_prompt: String,
    pub(crate) rel_id_user_prompt: String,
    pub(crate) rel_id_llm_schema: String,
    pub(crate) rel_id_llm_model: String,
}

impl AppConfig {
    pub(crate) fn new() -> Self {
        Self {
            ollama_base_url: "http://studio:11434/api".to_owned(),
            graphqlite_db: ":memory:".to_owned(),
            cytoscape_json_file: "./cytoscape/data.json".to_owned(),
            entity_id_sys_prompt: "./prompts/entity_identification_sys.txt".to_owned(),
            entity_id_user_prompt: "./prompts/entity_identification_user.txt".to_owned(),
            entity_id_llm_schema: "./prompts/entity_extraction_schema.json".to_owned(),
            entity_id_llm_model: "granite4:latest".to_owned(),
            rel_id_sys_prompt: "./prompts/relationship_identification_sys.txt".to_owned(),
            rel_id_user_prompt: "./prompts/relationship_identification_user.txt".to_owned(),
            rel_id_llm_schema: "./prompts/entity_relationship_schema.json".to_owned(),
            rel_id_llm_model: "granite4:latest".to_owned(),
        }
    }
}
