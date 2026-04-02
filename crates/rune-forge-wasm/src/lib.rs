use wasm_bindgen::prelude::*;

use rune_forge::compiler;
use rune_forge::decoder;
use rune_forge::generator;
use rune_forge::parser;
use rune_forge::types::Op;
use runeauth::{Rune, Restriction as RuneauthRestriction};

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

/// Create a base64-encoded rune from a hex secret and raw restriction string.
#[wasm_bindgen]
pub fn create_rune(secret_hex: &str, restrictions_raw: &str) -> Result<String, String> {
    let secret = hex::decode(secret_hex).map_err(|e| format!("invalid hex secret: {}", e))?;

    let mut rune = Rune::new_master_rune(&secret, vec![], None, None)
        .map_err(|e| format!("failed to create master rune: {}", e))?;

    if !restrictions_raw.is_empty() {
        for restriction_str in restrictions_raw.split('&') {
            let (restriction, _) =
                RuneauthRestriction::decode(restriction_str, false)
                    .map_err(|e| format!("failed to decode restriction '{}': {}", restriction_str, e))?;
            rune.add_restriction(restriction)
                .map_err(|e| format!("failed to add restriction: {}", e))?;
        }
    }

    Ok(rune.to_base64())
}

/// Decode a base64-encoded rune into a JSON restriction breakdown.
#[wasm_bindgen]
pub fn decode_rune_base64(rune_base64: &str) -> Result<String, String> {
    // Validate the rune parses correctly.
    Rune::from_base64(rune_base64)
        .map_err(|e| format!("invalid base64 rune: {}", e))?;

    // Extract the restriction string portion from the rune's string representation.
    // Rune::to_string() yields "[64-hex-authcode]:[restrictions]".
    // We re-parse via from_base64 and then decode to_string to get restriction text.
    let rune = Rune::from_base64(rune_base64)
        .map_err(|e| format!("invalid base64 rune: {}", e))?;
    let rune_str = rune.to_string();
    // The format is "64hexchars:[restrictions]"
    let rest_str = rune_str.get(65..).unwrap_or("");

    let mut restrictions: Vec<serde_json::Value> = vec![];
    let mut remaining = rest_str;
    while !remaining.is_empty() {
        let allow_idfield = restrictions.is_empty();
        let (restriction, next) = RuneauthRestriction::decode(remaining, allow_idfield)
            .map_err(|e| format!("failed to decode restriction: {}", e))?;
        let alternatives: Vec<serde_json::Value> = restriction
            .alternatives
            .iter()
            .map(|alt: &runeauth::Alternative| {
                let field = alt.get_field();
                let cond = alt.get_condition();
                let value = alt.get_value();
                let (op_char, op_name) = runeauth_condition_to_op(cond);
                serde_json::json!({
                    "field": field,
                    "op": op_char,
                    "op_name": op_name,
                    "value": value,
                })
            })
            .collect();
        restrictions.push(serde_json::json!({ "alternatives": alternatives }));
        remaining = next;
    }

    serde_json::to_string(&serde_json::json!({ "restrictions": restrictions }))
        .map_err(|e| e.to_string())
}

fn runeauth_condition_to_op(cond: runeauth::Condition) -> (&'static str, &'static str) {
    match cond {
        runeauth::Condition::Missing => ("!", "missing"),
        runeauth::Condition::Equal => ("=", "equal"),
        runeauth::Condition::NotEqual => ("/", "not equal"),
        runeauth::Condition::BeginsWith => ("^", "starts with"),
        runeauth::Condition::EndsWith => ("$", "ends with"),
        runeauth::Condition::Contains => ("~", "contains"),
        runeauth::Condition::IntLT => ("<", "less than"),
        runeauth::Condition::IntGT => (">", "greater than"),
        runeauth::Condition::LexLT => ("{", "lex less than"),
        runeauth::Condition::LexGT => ("}", "lex greater than"),
        runeauth::Condition::Comment => ("#", "comment"),
    }
}
