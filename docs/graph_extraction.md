# Graph extraction

## Entity extraction phase

### Validation rules

- `AT_LEAST_TWO_NODES`:
    - Rule: An extraction pass must yield at least two node
    - Correction: Prompt the LLM to extract more entities from the provided text.

---

## Relationship extraction phase

### Validation rules

- `INVALID_EDGE_PAIRING`:
    - Rule: An edge of type T must originate from an allowed source entity type and target an allowed destination entity type. See the rules below.
    - Correction: Prompt the LLM to correct the entity relation or provide an specific response if no valid option exists.

```json
PAIRING_RULES = {
    "WORKED_AT": {
        "description": "Person/Author worked at Organization",
        "source_types": ["Person", "Author"],
        "target_types": ["Organization"],
        "symmetric": False
    },
    "RELATED_TO": {
        "description": "General semantic relationship between entities",
        "source_types": ["*"],  # Any entity type
        "target_types": ["*"],
        "symmetric": True,
        "properties": ["context", "relationship_type", "confidence"]
    },
    "LOCATED_IN": {
        "description": "Entity is physically located in a Location",
        "source_types": ["Person", "Organization", "Event", "Product", "Lifeform"],
        "target_types": ["Location"],
        "symmetric": False,
        "properties": ["start_date", "end_date", "context"]
    },
    "COLLABORATED_WITH": {
        "description": "Collaboration between authors or persons",
        "source_types": ["Author", "Person"],
        "target_types": ["Author", "Person"],
        "symmetric": True,
        "properties": ["project", "document_id", "confidence"]
    },
    "PART_OF": {
        "description": "Hierarchical membership or composition",
        "source_types": ["Person", "Organization", "Location", "Concept", "Lifeform"],
        "target_types": ["Organization", "Location", "Concept", "Lifeform"],
        "symmetric": False,
        "properties": ["role", "context"]
    },
    "CREATED": {
        "description": "Entity created another entity (product, technology)",
        "source_types": ["Person", "Organization"],
        "target_types": ["Product", "Technology", "Concept"],
        "symmetric": False,
        "properties": ["date", "context"]
    },
    "USES": {
        "description": "Entity uses technology or product",
        "source_types": ["Person", "Organization", "Lifeform"],
        "target_types": ["Technology", "Product"],
        "symmetric": False
    },
    "IMPLEMENTS": {
        "description": "Technology implements concept",
        "source_types": ["Technology", "Product"],
        "target_types": ["Concept"],
        "symmetric": False
    },
    "PARTICIPATED_IN": {
        "description": "Entity participated in an event",
        "source_types": ["Person", "Organization", "Author"],
        "target_types": ["Event"],
        "symmetric": False,
        "properties": ["role", "context"]
    },
    "OCCURRED_IN": {
        "description": "Event occurred in location",
        "source_types": ["Event"],
        "target_types": ["Location"],
        "symmetric": False,
        "properties": ["date", "context"]
    },
    "AFFILIATED_WITH": {
        "description": "Professional affiliation",
        "source_types": ["Person", "Author"],
        "target_types": ["Organization"],
        "symmetric": False,
        "properties": ["role", "start_date", "end_date"]
    },
    "MENTIONED_WITH": {
        "description": "Co-occurrence in same context (chunk)",
        "source_types": ["*"],
        "target_types": ["*"],
        "symmetric": True,
        "properties": ["chunk_id", "frequency", "confidence"]
    },
    "FOUNDED": {
        "description": "Person founded organization",
        "source_types": ["Person", "Author"],
        "target_types": ["Organization"],
        "symmetric": False,
        "properties": ["date", "context"]
    }
}
```

---

## Graph finalization phase

### Validation rules

- `NO_NODE_WITHOUT_EDGE`
    - Rule: All extracted nodes must have at least one edge connecting to another node.
    - Correction: Provide context chunks mentioning the entity to the LLM and prompt to fill a missing edge. If no edge exists, return a certain response to signify.
