use super::types::CodeSnippet;

pub fn build_snippets(endpoint_url: &str, secret_key: &str, model_id: &str) -> Vec<CodeSnippet> {
    vec![
        CodeSnippet {
            language: "bash".into(),
            label: "curl".into(),
            code: build_curl_snippet(endpoint_url, secret_key, model_id),
        },
        CodeSnippet {
            language: "python".into(),
            label: "Python".into(),
            code: build_python_snippet(endpoint_url, secret_key, model_id),
        },
        CodeSnippet {
            language: "javascript".into(),
            label: "JavaScript".into(),
            code: build_javascript_snippet(endpoint_url, secret_key, model_id),
        },
        CodeSnippet {
            language: "python".into(),
            label: "Embeddings".into(),
            code: build_embeddings_snippet(endpoint_url, secret_key, model_id),
        },
        CodeSnippet {
            language: "yaml".into(),
            label: "Docker Compose".into(),
            code: build_docker_snippet(endpoint_url, secret_key, model_id),
        },
    ]
}

fn build_curl_snippet(endpoint_url: &str, secret_key: &str, model_id: &str) -> String {
    format!(
        r#"curl "{endpoint_url}/chat/completions" \
  -H "Authorization: Bearer {secret_key}" \
  -H "Content-Type: application/json" \
  -d '{{
    "model": "{model_id}",
    "messages": [
      {{ "role": "user", "content": "Hello from SelfAPI" }}
    ]
  }}'"#
    )
}

fn build_python_snippet(endpoint_url: &str, secret_key: &str, model_id: &str) -> String {
    format!(
        r#"from openai import OpenAI

client = OpenAI(
    base_url="{endpoint_url}",
    api_key="{secret_key}",
)

response = client.chat.completions.create(
    model="{model_id}",
    messages=[
        {{"role": "user", "content": "Hello from SelfAPI"}},
    ],
)

print(response.choices[0].message.content)"#
    )
}

fn build_javascript_snippet(endpoint_url: &str, secret_key: &str, model_id: &str) -> String {
    format!(
        r#"import OpenAI from "openai";

const client = new OpenAI({{
  baseURL: "{endpoint_url}",
  apiKey: "{secret_key}",
}});

const response = await client.chat.completions.create({{
  model: "{model_id}",
  messages: [
    {{ role: "user", content: "Hello from SelfAPI" }},
  ],
}});

console.log(response.choices[0].message.content);"#
    )
}

fn build_embeddings_snippet(endpoint_url: &str, secret_key: &str, model_id: &str) -> String {
    format!(
        r#"from openai import OpenAI

client = OpenAI(
    base_url="{endpoint_url}",
    api_key="{secret_key}",
)

res = client.embeddings.create(
    model="{model_id}",
    input="SelfAPI local vector embeddings test",
)

print(f"Embedding vector dimensions: {{len(res.data[0].embedding)}}")"#
    )
}

fn build_docker_snippet(endpoint_url: &str, secret_key: &str, _model_id: &str) -> String {
    format!(
        r#"version: '3.8'
services:
  open-webui:
    image: ghcr.io/open-webui/open-webui:main
    ports:
      - "3000:8080"
    environment:
      - OPENAI_API_BASE_URL={endpoint_url}
      - OPENAI_API_KEY={secret_key}
    restart: always"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snippets_include_model_and_key() {
        let snippets = build_snippets(
            "http://127.0.0.1:8787/v1",
            "sk-selfapi-test",
            "qwen2.5-7b-instruct",
        );

        assert_eq!(snippets.len(), 5);
        assert!(snippets[0].code.contains("sk-selfapi-test"));
        assert!(snippets[1].code.contains("qwen2.5-7b-instruct"));
        assert!(snippets[3].code.contains("embeddings"));
        assert!(snippets[4].code.contains("OPENAI_API_BASE_URL"));
    }
}
