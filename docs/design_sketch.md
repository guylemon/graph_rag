# Graph RAG indexing POC

Inspired by the [Microsoft Graph RAG workflow](https://microsoft.github.io/graphrag/index/default_dataflow/#indexing-dataflow), fit for my purpose to use with SQLite + GraphQLite extension.

## Glossary

- **Canonical**: The normalized, deduplicated representation of something. For example, "apple" is the canonical form of the **surface forms** "Apples", "apple"; `GraphRelationship(apple, IS_A, fruit)` is canonical, while a `relationship_mentions` are concrete evidence of the relationship (aka **edge**).

## Domain Concepts

### RawInput

Represents an unprocessed ingestion payload before indexing starts.

- **text** (`String`): Raw source text to ingest.

### Document

Top-level source **node** for a single ingested input in the graph layer.

- **id** (`String`): Unique document identifier.
- **title** (`String?`): Optional human-readable title.
- **text** (`String`): Full document text.
- **content_hash** (`String`): Deterministic hash of the full text for dedup/change detection.
- **metadata_json** (`String`): JSON-encoded source and ingest metadata.

### TextUnit

Chunked unit of a document used as the primary extraction/provenance boundary. Chunks may overlap.

- **id** (`String`): Unique text unit identifier.
- **document_id** (`String`): Owning `Document.id`.
- **raw_text** (`String`): Text content for this unit.
- **char_start** (`Int`): Inclusive start offset in the parent document.
- **char_end** (`Int`): Exclusive end offset in the parent document.
- **token_count** (`Int`): Token count for extraction/model budgeting.

### EntityType

Enum used by `Entity.entity_type`.

- **values** (`Enum`): Domain categories (for example: `Food`, `Color`, `FoodCategory`, `Plant`).

### Entity

Canonical concept **node** produced by normalizing and deduplicating mentions.

- **id** (`String`): Unique entity identifier.
- **canonical_name** (`String`): Normalized name used for deduplication.
- **display_name** (`String`): Human-readable surface form.
- **entity_type** (`EntityType`): Semantic category.
- **description** (`String?`): Optional free-text description.

### HAS_TEXT_UNIT

Directed **edge** from `Document` to `TextUnit` indicating containment.

> `Document -[:HAS_TEXT_UNIT]-> TextUnit`

- **document_id** (`String`): Source `Document.id`.
- **text_unit_id** (`String`): Target `TextUnit.id`.

### MENTIONS

Directed **edge** from `TextUnit` to `Entity` carrying mention-level extraction attributes.

> `TextUnit -[:MENTIONS {span_start:Int, span_end:Int, mention_text:String, confidence:Float}]-> Entity`

- **text_unit_id** (`String`): Source `TextUnit.id`.
- **entity_id** (`String`): Target `Entity.id`.
- **span_start** (`Int`): Inclusive mention start offset in `TextUnit.raw_text`.
- **span_end** (`Int`): Exclusive mention end offset in `TextUnit.raw_text`.
- **mention_text** (`String`): Exact extracted surface text.
- **confidence** (`Float`): Extraction confidence score.

### RelationshipPredicate

Enum used by relationship predicates.

- **values** (`Enum`): Normalized predicates (for example: `IS_A`, `HAS_COLOR`, `GROWS_ON`).

### GraphRelationship (`RELATES_TO`)

Canonical directed relationship **edge** between two entities.

> `Entity -[:RELATES_TO {predicate:Enum, weight:Float, evidence_count:Int, description:String?}]-> Entity`

- **source_entity_id** (`String`): Source `Entity.id`.
- **target_entity_id** (`String`): Target `Entity.id`.
- **predicate** (`RelationshipPredicate`): Normalized relationship type.
- **weight** (`Float`): Aggregated relationship strength.
- **evidence_count** (`Int`): Number of supporting mentions.
- **description** (`String?`): Optional human-readable relationship description.

### EntityMention (`entity_mentions`)

Provenance sidecar **record** for a single entity mention extraction.

> table name `entity_mentions`

- **text_unit_id** (`String`): `TextUnit.id` where mention was found.
- **entity_id** (`String`): Canonical `Entity.id` linked to the mention.
- **mention_text** (`String`): Extracted surface text.
- **span_start** (`Int`): Inclusive mention start offset in the text unit.
- **span_end** (`Int`): Exclusive mention end offset in the text unit.
- **confidence** (`Float`): Mention extraction confidence.
- **extractor_version** (`String`): Version identifier of the extraction pipeline.

### RelationshipMention (`relationship_mentions`)

Provenance sidecar **record** for a single relationship mention extraction.

> table name `relationship_mentions`

- **text_unit_id** (`String`): `TextUnit.id` where relationship evidence was found.
- **source_entity_id** (`String`): Source canonical entity.
- **target_entity_id** (`String`): Target canonical entity.
- **predicate** (`RelationshipPredicate`): Relationship predicate extracted from text.
- **span_start** (`Int`): Inclusive evidence span start offset in the text unit.
- **span_end** (`Int`): Exclusive evidence span end offset in the text unit.
- **confidence** (`Float`): Relationship extraction confidence.
- **extractor_version** (`String`): Version identifier of the extraction pipeline.

---

## Data flow

**Input**
- Raw text: `"Apples are a red fruit that grow on trees."`

**Indexing Workflow (transformation focused `A` -> `B`)**

1. **Raw input -> Document object**
- Input type: `RawInput {text:String}`
- Output type: `Document`
- Example output values:
  - `id`: `doc_001`
  - `text`: original sentence
  - `content_hash`: deterministic hash string of full text
  - `metadata_json`: source info (e.g., ingest timestamp, source type)

2. **Document -> TextUnits**
- Input type: `Document`
- Output type: `List<TextUnit>`
- Example (single chunk because sentence is short):
  - `TextUnit.id`: `tu_doc_001_0`
  - `document_id`: `doc_001`
  - `raw_text`: same sentence
  - `char_start`: `0`
  - `char_end`: `42`
  - `token_count`: `9`

3. **TextUnit -> Entity mentions (surface forms)**
- Input type: `TextUnit.raw_text:String`
- Output type: `List<EntityMention>`
- Example extracted mentions:
  - `"Apples"` span `[0,6)` type candidate `Food`
  - `"red"` span `[13,16)` type candidate `Color`
  - `"fruit"` span `[17,22)` type candidate `FoodCategory`
  - `"trees"` span `[36,41)` type candidate `Plant`

4. **Entity mentions -> Canonical entities**
- Input type: `List<EntityMention>`
- Output type: `List<Entity>`
- Transformations happening:
  - plural to singular: `"Apples"` -> `"apple"`, `"trees"` -> `"tree"`
  - normalized key generation for dedup
- Example canonical entities:
  - `e_apple`: `canonical_name=apple`, `display_name=Apples`, `entity_type=Food`
  - `e_red`: `canonical_name=red`, `entity_type=Color`
  - `e_fruit`: `canonical_name=fruit`, `entity_type=FoodCategory`
  - `e_tree`: `canonical_name=tree`, `entity_type=Plant`

5. **TextUnit -> Relationship mentions (statement candidates)**
- Input type: `TextUnit.raw_text + canonical entities`
- Output type: `List<RelationshipMention>`
- Example relationship mentions:
  - `(apple) IS_A (fruit)` confidence `0.96`
  - `(apple) HAS_COLOR (red)` confidence `0.93`
  - `(apple) GROWS_ON (tree)` confidence `0.91`

6. **Relationship mentions -> Canonical graph relationships**
- Input type: `List<RelationshipMention>`
- Output type: `List<GraphRelationship>`
- Transformations happening:
  - predicate normalization to enum (`IS_A`, `HAS_COLOR`, `GROWS_ON`)
  - duplicate collapse (none in this one-sentence example)
  - aggregate metrics:
    - `weight` from confidence aggregation
    - `evidence_count` from number of supporting mentions
- Example outputs:
  - `e_apple -[:RELATES_TO {predicate:IS_A, weight:0.96, evidence_count:1}]-> e_fruit`
  - `e_apple -[:RELATES_TO {predicate:HAS_COLOR, weight:0.93, evidence_count:1}]-> e_red`
  - `e_apple -[:RELATES_TO {predicate:GROWS_ON, weight:0.91, evidence_count:1}]-> e_tree`

7. **Write provenance links**
- Input type: entity/relationship mentions
- Output type: provenance records linking extracted facts to source text
- Example entity provenance (`entity_mentions`):
  - `(tu_doc_001_0, e_apple, "Apples", 0, 6, 0.99, "extractor_v1")`
  - `(tu_doc_001_0, e_red, "red", 13, 16, 0.98, "extractor_v1")`
- Example relationship provenance (`relationship_mentions`):
  - `(tu_doc_001_0, e_apple, e_fruit, IS_A, 0, 22, 0.96, "extractor_v1")`
  - `(tu_doc_001_0, e_apple, e_tree, GROWS_ON, 28, 41, 0.91, "extractor_v1")`

8. **Final persisted state (same DB file)**
- Graph layer now contains:
  - `1 Document`, `1 TextUnit`, `4 Entity` nodes
  - `1 HAS_TEXT_UNIT`, `4 MENTIONS`, `3 RELATES_TO` edges
- Provenance layer now contains:
  - `4 entity mention rows`
  - `3 relationship mention rows`
