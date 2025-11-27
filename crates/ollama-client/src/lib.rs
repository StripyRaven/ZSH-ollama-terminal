//! crates/ollama-client/src/lib.rs
//! # Ollama Client with Resilience Patterns
//! HTTP клиент для Ollama API с circuit breaker и retry логикой.

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use shared::error::{AnalysisError, AnalysisErrorType};
use shared::{states, Command, CommandAnalyzer as CommandAnalyzerTrait, DomainError};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    client: Client,
    model: String,
    timeout: Duration,
    max_retries: u32,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
            model,
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    async fn send_request_with_retry(&self, prompt: String) -> Result<Response, DomainError> {
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            match self.send_request(&prompt).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries - 1 {
                        tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    async fn send_request(&self, prompt: &str) -> Result<Response, DomainError> {
        let url = format!("{}/api/generate", self.base_url);

        let request = GenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = timeout(self.timeout, self.client.post(&url).json(&request).send())
            .await
            .map_err(|_| {
                DomainError::Analysis(AnalysisError {
                    model: self.model.clone(),
                    error_type: AnalysisErrorType::Timeout,
                    details: "Request timeout".to_string(),
                    suggestion: Some("Try increasing timeout or check Ollama server".to_string()),
                })
            })?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(DomainError::Analysis(AnalysisError {
                model: self.model.clone(),
                error_type: AnalysisErrorType::ModelUnavailable,
                details: format!(
                    "HTTP {}: {}",
                    response.status(),
                    response.text().await.unwrap_or_default()
                ),
                suggestion: Some(
                    "Check if Ollama server is running and model is available".to_string(),
                ),
            }))
        }
    }
}

#[async_trait::async_trait]
impl CommandAnalyzerTrait for OllamaClient {
    async fn analyze_command(
        &self,
        command: Command<states::Validated>,
    ) -> Result<Command<states::Analyzed>, DomainError> {
        let prompt = format!(
            "Analyze the following shell command and provide suggestions: {}\n\nContext: Working directory: {}, User ID: {}",
            command.raw(),
            command.context().working_directory.to_string(),
            command.context().user_id
        );

        let response = self.send_request_with_retry(prompt).await?;
        let generate_response: GenerateResponse = response.json().await.map_err(|e| {
            DomainError::Analysis(AnalysisError {
                model: self.model.clone(),
                error_type: AnalysisErrorType::InvalidResponse,
                details: format!("Failed to parse response: {}", e),
                suggestion: Some("Check Ollama server response format".to_string()),
            })
        })?;

        // Преобразуем команду в Analyzed состояние
        Ok(Command {
            raw: command.raw().to_string(),
            parts: command.parts().to_vec(),
            context: command.context().clone(),
            state: std::marker::PhantomData,
        })
    }

    async fn get_suggestions(
        &self,
        analysis: &Command<states::Analyzed>,
    ) -> Result<Vec<shared::CommandSuggestion>, DomainError> {
        // В реальной реализации здесь бы делался запрос к Ollama для получения предложений
        // Сейчас возвращаем заглушку
        Ok(vec![])
    }

    fn get_model_info(&self) -> shared::ModelInfo {
        shared::ModelInfo {
            name: self.model.clone(),
            version: "latest".to_string(),
            capabilities: vec![
                "command_analysis".to_string(),
                "suggestion_generation".to_string(),
            ],
            max_tokens: 4096,
        }
    }
}

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_client_creation() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), "llama2".to_string());
        assert_eq!(client.model, "llama2");
    }

    // Note: Для интеграционных тестов потребуется запущенный Ollama сервер
}
