pub const upsert_service_node_cypher: &str =
    r#"
    MERGE (s:ServiceNode {name: $name})
    ON CREATE SET s.name = $name
    RETURN s
"#;

pub const upsert_operation_cypher: &str =
    r#"
    MERGE (o:Operation {id: $id})
    ON CREATE o.label = $label, o.id = $id
    RETURN o
"#;

pub const upsert_service_node_to_operation_cypher: &str =
    r#"
    MATCH (s:ServiceNode {name: $name}), (o:Operation {id: $id})
    MERGE (s)-[r:EXPOSES]-(o)
    RETURN r
"#;

pub const upsert_service_to_service_operation_relation: &str =
    r#"
    MATCH (s:ServiceNode {name: $name}), (o:Operation {id: $id})
    MERGE (s)-[r:INVOKES]->(o)
    ON CREATE SET r.method = $method, r.path = $path
    RETURN r
"#;
