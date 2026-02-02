use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;

#[derive(Deserialize)]
pub struct WebSearchArgs {
    pub query: String,
}

#[derive(Serialize)]
pub struct WebSearchResult {
    pub query: String,
    pub results: Vec<SearchResultItem>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Web search error: {0}")]
pub struct WebSearchError(String);

pub struct WebSearch {
    client: reqwest::Client,
}

impl WebSearch {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn search_duckduckgo(&self, query: &str) -> Result<Vec<SearchResultItem>, WebSearchError> {
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| WebSearchError(format!("请求失败: {}", e)))?;

        let html = response
            .text()
            .await
            .map_err(|e| WebSearchError(format!("读取响应失败: {}", e)))?;

        let results = Self::parse_duckduckgo_html(&html);

        if results.is_empty() {
            return Err(WebSearchError("未找到搜索结果".to_string()));
        }

        Ok(results)
    }

    fn parse_duckduckgo_html(html: &str) -> Vec<SearchResultItem> {
        let mut results = Vec::new();

        let result_divs: Vec<&str> = html.split("result__body").collect();

        for (_i, div) in result_divs.iter().enumerate().skip(1).take(5) {
            if let Some(title) = Self::extract_between(div, "result__a\">", "</a>") {
                let url = Self::extract_between(div, "uddg=", "\"")
                    .and_then(|u| urlencoding::decode(&u).ok().map(|s| s.to_string()))
                    .unwrap_or_default();

                let snippet = Self::extract_between(div, "result__snippet\">", "</a>")
                    .unwrap_or_default();

                if !title.is_empty() && !url.is_empty() {
                    results.push(SearchResultItem {
                        title: Self::clean_html(&title),
                        url,
                        snippet: Self::clean_html(&snippet),
                    });
                }
            }
        }

        results
    }

    fn extract_between<'a>(text: &'a str, start: &str, end: &str) -> Option<String> {
        let start_idx = text.find(start)? + start.len();
        let remaining = &text[start_idx..];
        let end_idx = remaining.find(end)?;
        Some(remaining[..end_idx].to_string())
    }

    fn clean_html(text: &str) -> String {
        text.replace("<b>", "")
            .replace("</b>", "")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .trim()
            .to_string()
    }
}

impl Tool for WebSearch {
    const NAME: &'static str = "web_search";
    type Error = WebSearchError;
    type Args = WebSearchArgs;
    type Output = WebSearchResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "使用DuckDuckGo搜索引擎在互联网上搜索信息。当用户询问需要实时信息、最新资讯、或你不确定的知识时使用此工具。返回搜索结果包括标题、链接和摘要。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "搜索查询关键词"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        debug!("[Tool] web_search called: query={}", args.query);

        let results = self.search_duckduckgo(&args.query).await?;

        debug!("[Tool] web_search completed: found {} results", results.len());

        Ok(WebSearchResult {
            query: args.query,
            results,
        })
    }
}