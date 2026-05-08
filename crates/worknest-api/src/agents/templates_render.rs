//! Literal `{{key}}` substitution. Missing keys raise an error rather than
//! silently leaving the placeholder behind.

use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("missing template placeholder: {{{{0}}}}")]
    MissingKey(String),
}

pub fn render(template: &str, vars: &HashMap<&str, String>) -> Result<String, RenderError> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        let end = rest
            .find("}}")
            .ok_or_else(|| RenderError::MissingKey("<unterminated>".into()))?;
        let key = rest[..end].trim();
        let value = vars
            .get(key)
            .ok_or_else(|| RenderError::MissingKey(key.to_string()))?;
        out.push_str(value);
        rest = &rest[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_known_keys() {
        let mut vars = HashMap::new();
        vars.insert("name", "alice".to_string());
        vars.insert("city", "Paris".to_string());
        let s = render("hello {{name}} from {{city}}", &vars).unwrap();
        assert_eq!(s, "hello alice from Paris");
    }

    #[test]
    fn missing_key_errors() {
        let vars = HashMap::new();
        let r = render("hi {{unknown}}", &vars);
        assert!(matches!(r, Err(RenderError::MissingKey(k)) if k == "unknown"));
    }
}
