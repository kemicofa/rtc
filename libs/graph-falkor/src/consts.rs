/*

*/
pub const UPSERT_SERVICE_NODE_CYPHER: &str =
    r#"
    MERGE ( s:Service { name: $name })
    ON CREATE SET s.name = $name
    RETURN s
"#;

pub const UPSERT_OPERATION_CYPHER: &str =
    r#"
    MERGE (o:Operation {id: $id})
    ON CREATE SET o.label = $label, o.id = $id
    RETURN o
"#;

pub const UPSERT_SERVICE_NODE_TO_OPERATION_CYPHER: &str =
    r#"
    MATCH (s:Service {name: $name}), (o:Operation {id: $id})
    MERGE (s)-[r:EXPOSES]-(o)
    RETURN r
"#;

pub const UPSERT_SERVICE_TO_SERVICE_OPERATION_RELATION: &str =
    r#"
    MATCH (s:Service {name: $name}), (o:Operation {id: $id})
    MERGE (s)-[r:INVOKES]->(o)
    RETURN r
"#;
