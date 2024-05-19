use axum::{
    extract::{MatchedPath, Query, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use material_colors::{color::Argb, theme::ThemeBuilder};
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use std::str::FromStr;
use tracing::{error, info};

#[derive(Deserialize)]
struct PaletteQuery {
    base_color: String,
}

async fn get_palette(pagination: Query<PaletteQuery>) -> Result<String, AppError> {
    let query: PaletteQuery = pagination.0;

    info!("Generating theme with color string {:?}", &query.base_color);

    let theme = ThemeBuilder::with_source(Argb::from_str(&query.base_color)?).build();

    let final_string = theme
        .schemes
        .light
        .into_iter()
        .map(|x| x.1.to_hex())
        .collect::<Vec<_>>()
        .join("");
    
    info!("Generated theme: {:?}", final_string);

    Ok(final_string)
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/getPalette", get(get_palette))
        .route("/", get(hello_world))
        // !!! From https://github.com/tokio-rs/axum/blob/main/examples/error-handling/src/main.rs !!!
        .layer(
            TraceLayer::new_for_http()
                // Create our own span for the request and include the matched path. The matched
                // path is useful for figuring out which handler the request was routed to.
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();

                    // axum automatically adds this extension.
                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());

                    tracing::debug_span!("request", %method, %uri, matched_path)
                })
        );

    Ok(router.into())
}

// !!! From https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs !!!
// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Error occurred: {}", self.0);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
