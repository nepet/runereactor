use wasm_bindgen::prelude::*;

use rune_forge::compiler;
use rune_forge::decoder;
use rune_forge::generator;
use rune_forge::parser;
use rune_forge::types::Op;

/// Compile a .rf policy source to the specified output format.
/// `format` must be one of: "json", "cln", "raw"
#[wasm_bindgen]
pub fn compile_policy(source: &str, format: &str) -> Result<String, String> {
    let policy = parser::parse_policy(source).map_err(|e| e.to_string())?;
    let rune_policy = compiler::compile(&policy).map_err(|e| e.to_string())?;

    let restrictions: Vec<Vec<String>> = rune_policy
        .restrictions
        .iter()
        .map(|r| {
            r.alternatives
                .iter()
                .map(|c| {
                    if c.op == Op::Missing {
                        format!("{}{}", c.field, c.op.as_char())
                    } else {
                        format!("{}{}{}", c.field, c.op.as_char(), c.value)
                    }
                })
                .collect()
        })
        .collect();

    match format {
        "json" => serde_json::to_string(&restrictions).map_err(|e| e.to_string()),
        "cln" => {
            let json = serde_json::to_string(&restrictions).map_err(|e| e.to_string())?;
            Ok(format!("lightning-cli createrune -k \"restrictions\"='{}'", json))
        }
        "raw" => {
            let raw: Vec<String> = restrictions.iter().map(|alts| alts.join("|")).collect();
            Ok(raw.join("&"))
        }
        _ => Err(format!("unknown format '{}', expected json|cln|raw", format)),
    }
}

/// Parse a .rf policy source into a JSON-serialized compiled policy.
#[wasm_bindgen]
pub fn parse_policy(source: &str) -> Result<String, String> {
    let policy = parser::parse_policy(source).map_err(|e| e.to_string())?;
    let rune_policy = compiler::compile(&policy).map_err(|e| e.to_string())?;
    serde_json::to_string(&rune_policy).map_err(|e| e.to_string())
}

/// Decode a raw rune restriction string into a JSON breakdown with operator names.
#[wasm_bindgen]
pub fn decode_rune(raw: &str) -> Result<String, String> {
    let rune_policy = decoder::decode_rune(raw).map_err(|e| e.to_string())?;

    let restrictions: Vec<serde_json::Value> = rune_policy
        .restrictions
        .iter()
        .map(|r| {
            let alternatives: Vec<serde_json::Value> = r
                .alternatives
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "field": c.field,
                        "op": c.op.as_char().to_string(),
                        "op_name": c.op.name(),
                        "value": c.value,
                    })
                })
                .collect();
            serde_json::json!({ "alternatives": alternatives })
        })
        .collect();

    serde_json::to_string(&serde_json::json!({ "restrictions": restrictions }))
        .map_err(|e| e.to_string())
}

/// Generate .rf policy source text from a JSON spec.
#[wasm_bindgen]
pub fn generate_policy_from_spec(spec: &str) -> Result<String, String> {
    let policy_spec: generator::PolicySpec =
        serde_json::from_str(spec).map_err(|e| e.to_string())?;
    generator::generate_policy(&policy_spec).map_err(|e| e.to_string())
}
