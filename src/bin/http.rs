use azvs_quote::adapter::http;
use azvs_quote::app::{AppState, HttpConfig};
use axum::http::header::HeaderName;
use axum::http::{HeaderValue, Method};
use tokio::net::TcpListener;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::new().await?;
    let http_cfg = state.config.http.clone();
    let addr = http_cfg.addr.clone();
    let mut http_app = http::router().with_state(state);

    if http_cfg.cors_enabled {
        http_app = http_app.layer(build_cors_layer(&http_cfg)?);
    }

    let listener = TcpListener::bind(&addr).await?;
    println!("quote-http listening on http://{}", addr);

    axum::serve(listener, http_app).await?;
    Ok(())
}

fn build_cors_layer(cfg: &HttpConfig) -> anyhow::Result<CorsLayer> {
    // 开启 CORS 时必须显式配置可访问来源，避免误放开。
    if cfg.cors_origins.is_empty() {
        anyhow::bail!("http.cors_enabled=true requires non-empty http.cors_origins");
    }

    // 将配置中的字符串来源解析为 HeaderValue（例如 https://example.com）。
    let origins: Vec<HeaderValue> = cfg
        .cors_origins
        .iter()
        .map(|v| v.parse::<HeaderValue>())
        .collect::<Result<_, _>>()?;

    // 将配置中的方法字符串解析为 HTTP Method（GET/POST/...）。
    let methods: Vec<Method> = cfg
        .cors_methods
        .iter()
        .map(|v| v.parse::<Method>())
        .collect::<Result<_, _>>()?;

    // 将配置中的请求头字符串解析为标准 HeaderName。
    let headers: Vec<HeaderName> = cfg
        .cors_headers
        .iter()
        .map(|v| v.parse::<HeaderName>())
        .collect::<Result<_, _>>()?;

    // 统一构建 CORS 规则。allow_credentials=true 时应始终使用来源白名单而非 '*'.
    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(AllowMethods::list(methods))
        .allow_headers(AllowHeaders::list(headers))
        .allow_credentials(cfg.cors_allow_credentials))
}
