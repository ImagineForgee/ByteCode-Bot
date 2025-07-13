use std::time::Duration;
use anyhow::{Context, Result};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, SqlitePool};

#[derive(Deserialize)]
struct LingvaTranslationInfo {
    #[serde(rename = "detectedSource")]
    detected_source: String,
}

#[derive(Deserialize)]
struct LingvaTranslationResponse {
    info: LingvaTranslationInfo,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize, Debug)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Deserialize, Debug)]
struct ChatMessageContent {
    content: String,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

fn normalize_lang_code(lang: &str) -> &str {
    lang.split('-').next().unwrap_or(lang)
}

pub async fn ai_translate(text: &str, source: &str, target: &str) -> Result<String> {
    let api_key = std::env::var("OPEN_ROUTER_TOKEN")
        .context("Missing OPEN_ROUTER_TOKEN env variable")?;

    println!("[Translate] Starting translation...");
    println!("[Translate] Source: {}, Target: {}", source, target);
    println!("[Translate] Text: {}", text);

    let prompt = format!(
        "Translate the following text strictly from {source} to {target}.\n\
         Only return the translated sentence. Do not include any explanations, markdown, or extra words.\n\n\
         Text: {text}"
    );

    println!("[Translate] Prompt:\n{}", prompt);

    let body = ChatRequest {
        model: "venice/uncensored:free".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.clone(),
        }],
        temperature: 0.0,
    };

    let client = Client::new();
    println!("[Translate] Sending request to OpenRouter...");

    let res = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
        .header(header::CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    println!("[Translate] Status: {}", res.status());

    let res = res.error_for_status()?;

    let json: ChatResponse = res.json().await?;
    println!("[Translate] Raw response: {:?}", json);

    let translated = json.choices.get(0)
        .context("No translation choice returned")?
        .message
        .content
        .trim()
        .to_string();

    println!("[Translate] Final translated result: {}", translated);
    Ok(translated)
}


pub async fn detect_language_lingva(client: &Client, text: &str) -> Result<String> {
    let from_lang = "auto";
    let to_lang = "en";
    let encoded_text = urlencoding::encode(text);
    let url = format!("https://lingva.ml/api/v1/{}/{}/{}", from_lang, to_lang, encoded_text);

    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to send request to Lingva")?;

    let body = resp
        .text()
        .await
        .context("Failed to read Lingva response body")?;

    let parsed: LingvaTranslationResponse =
        serde_json::from_str(&body).context("Failed to parse Lingva response")?;

    Ok(parsed.info.detected_source)
}


pub async fn translate_if_needed(
    text: &str,
    user_id: &str,
    pool: &SqlitePool,
) -> Result<Option<String>> {
    let pref: Option<(String,)> = sqlx::query_as(
        "SELECT lang_local FROM users WHERE id = ?",
    )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    let raw_target = match pref {
        Some((lang,)) => lang,
        None => return Ok(None),
    };

    let client = Client::new();
    let raw_source = detect_language_lingva(&client, text).await?;
    let source = normalize_lang_code(&raw_source);
    let target = normalize_lang_code(&raw_target);
    if source.to_string() == target.to_string() {
        return Ok(None);
    }

    let translated = ai_translate(text, source, target).await?;
    Ok(Some(translated))
}
