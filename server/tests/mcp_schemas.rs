//! D-1 (GOAL P1-D): the tool schemas a client sees. `MDK_TOOLS_LIST` is the
//! `tools/list` result printed by the official MCP Inspector
//! (`scripts/mcp_inspector.sh`); without it the server's own `tools::specs`
//! are checked (unit run). Per tool: name and description present; the
//! `inputSchema` is valid against the JSON Schema 2020-12 meta-schema (the
//! MCP default dialect) and compiles; it is an object schema that rejects
//! unknown properties, and every `required` name is a property. The Inspector
//! list must name exactly the server's tools, with the server's schemas.
//! `MDK_SCHEMA_OUT` receives the per-tool result as JSON.

use mdkserver::tools;
use serde_json::{json, Value};

fn check(tool: &Value) -> Result<(), String> {
    let name = tool["name"].as_str().ok_or("no name")?;
    let desc = tool["description"].as_str().unwrap_or("");
    if desc.trim().len() < 40 {
        return Err(format!("{name}: description missing or too short"));
    }
    let schema = &tool["inputSchema"];
    jsonschema::draft202012::meta::validate(schema)
        .map_err(|e| format!("{name}: not a valid JSON Schema 2020-12: {e}"))?;
    jsonschema::draft202012::new(schema).map_err(|e| format!("{name}: does not compile: {e}"))?;
    if schema["type"] != "object" {
        return Err(format!("{name}: inputSchema type is not object"));
    }
    if schema["additionalProperties"] != false {
        return Err(format!("{name}: unknown arguments are not rejected"));
    }
    let props = schema["properties"]
        .as_object()
        .ok_or(format!("{name}: no properties object"))?;
    for r in schema["required"].as_array().cloned().unwrap_or_default() {
        let r = r
            .as_str()
            .ok_or(format!("{name}: required entry not a string"))?;
        if !props.contains_key(r) {
            return Err(format!("{name}: required {r:?} is not a property"));
        }
    }
    for (p, s) in props {
        if s["description"].as_str().unwrap_or("").is_empty() {
            return Err(format!("{name}: property {p} has no description"));
        }
    }
    // the schema rejects an unknown argument
    let v = jsonschema::draft202012::new(schema).unwrap();
    if v.is_valid(&json!({"__unknown__": 1})) {
        return Err(format!("{name}: accepts an unknown argument"));
    }
    Ok(())
}

#[test]
fn tool_schemas_are_valid() {
    let own: Vec<Value> = tools::specs()
        .into_iter()
        .map(|s| json!({"name": s.name, "description": s.description, "inputSchema": s.input_schema}))
        .collect();
    let (source, listed) = match std::env::var("MDK_TOOLS_LIST") {
        Ok(p) if !p.is_empty() => {
            let v: Value = serde_json::from_slice(&std::fs::read(&p).expect("read MDK_TOOLS_LIST"))
                .expect("MDK_TOOLS_LIST is JSON");
            (p, v["tools"].as_array().expect("tools array").clone())
        }
        _ => ("tools::specs".to_string(), own.clone()),
    };
    let mut per = serde_json::Map::new();
    let mut errors = Vec::new();
    for t in &listed {
        let name = t["name"].as_str().unwrap_or("?").to_string();
        let mut r = check(t);
        if r.is_ok() {
            match own.iter().find(|o| o["name"] == t["name"]) {
                None => r = Err(format!("{name}: not a tool of this server")),
                Some(o) if o["inputSchema"] != t["inputSchema"] => {
                    r = Err(format!("{name}: listed schema differs from the server's"))
                }
                Some(o) if o["description"] != t["description"] => {
                    r = Err(format!(
                        "{name}: listed description differs from the server's"
                    ))
                }
                _ => {}
            }
        }
        per.insert(name, json!({"valid": r.is_ok(), "error": r.as_ref().err()}));
        if let Err(e) = r {
            errors.push(e);
        }
    }
    let names: Vec<&str> = listed.iter().filter_map(|t| t["name"].as_str()).collect();
    let expected: Vec<&str> = tools::specs().iter().map(|s| s.name).collect();
    if names != expected {
        errors.push(format!(
            "listed tools {names:?} != server tools {expected:?}"
        ));
    }
    let valid = per.values().filter(|v| v["valid"] == true).count();
    let report = json!({
        "source": source,
        "tools": listed.len(),
        "expected": expected.len(),
        "valid": valid,
        "per_tool": per,
        "pass": errors.is_empty(),
    });
    if let Ok(p) = std::env::var("MDK_SCHEMA_OUT") {
        if !p.is_empty() {
            std::fs::write(&p, serde_json::to_string_pretty(&report).unwrap() + "\n")
                .expect("write MDK_SCHEMA_OUT");
        }
    }
    assert!(errors.is_empty(), "schema errors: {errors:#?}");
}
