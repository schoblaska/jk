use serde::Deserialize;

const OLLAMA_URL: &str = "http://localhost:11434/api/embed";
const MODEL: &str = "nomic-embed-text";

#[derive(Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f64>>,
}

/// Embed texts via Ollama. Returns None on failure.
pub fn embed(texts: &[&str]) -> Option<Vec<Vec<f64>>> {
    let body = serde_json::json!({
        "model": MODEL,
        "input": texts,
    });

    let mut resp = ureq::post(OLLAMA_URL)
        .header("Content-Type", "application/json")
        .send(body.to_string().as_bytes())
        .ok()?;

    let data: EmbedResponse = resp.body_mut().read_json().ok()?;
    Some(data.embeddings)
}
