use axum::{
    extract::{MatchedPath, Query, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use material_colors::{
    color::Argb,
    theme::{CustomColor, ThemeBuilder},
};
use serde::Deserialize;
use std::str::FromStr;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
enum ThemeType {
    Dark,
    Light,
}

#[derive(Deserialize)]
struct PaletteQuery {
    base_color: String,
    theme_type: ThemeType,
}

async fn get_palette(pagination: Query<PaletteQuery>) -> Result<String, AppError> {
    let query: PaletteQuery = pagination.0;

    info!(
        "Generating {:?} theme with color string {:?}",
        &query.theme_type, &query.base_color
    );

    // Define some fixed colors to make people's lives easier
    let red = CustomColor {
        value: Argb::from_str("FF7676")?,
        name: "red".to_string(),
        blend: true,
    };
    let green = CustomColor {
        value: Argb::from_str("59EB5C")?,
        name: "green".to_string(),
        blend: true,
    };
    let blue = CustomColor {
        value: Argb::from_str("0000FF")?,
        name: "blue".to_string(),
        blend: true,
    };
    let yellow = CustomColor {
        value: Argb::from_str("F8F770")?,
        name: "yellow".to_string(),
        blend: false,
    };
    let purple = CustomColor {
        value: Argb::from_str("BA64F2")?,
        name: "purple".to_string(),
        blend: true,
    };
    let cyan = CustomColor {
        value: Argb::from_str("61D1FA")?,
        name: "cyan".to_string(),
        blend: true,
    };
    let orange = CustomColor {
        value: Argb::from_str("E69E50")?,
        name: "orange".to_string(),
        blend: false,
    };

    let custom_colors: Vec<CustomColor> = vec![red, green, blue, yellow, purple, cyan, orange];
    let theme = ThemeBuilder::with_source(Argb::from_str(&query.base_color)?)
        .custom_colors(custom_colors)
        .build();

    let base_theme_string = match query.theme_type {
        ThemeType::Dark => theme
            .schemes
            .dark
            .into_iter()
            .map(|x| x.1.to_hex())
            .collect::<Vec<_>>()
            .join(""),
        ThemeType::Light => theme
            .schemes
            .light
            .into_iter()
            .map(|x| x.1.to_hex())
            .collect::<Vec<_>>()
            .join(""),
    };

    let custom_colors_string = theme
        .custom_colors
        .iter()
        .map(|x| match query.theme_type {
            ThemeType::Dark => format!(
                "{}{}{}{}",
                x.dark.color.to_hex(),
                x.dark.color_container.to_hex(),
                x.dark.on_color.to_hex(),
                x.dark.on_color_container.to_hex()
            ),
            ThemeType::Light => format!(
                "{}{}{}{}",
                x.light.color.to_hex(),
                x.light.color_container.to_hex(),
                x.light.on_color.to_hex(),
                x.light.on_color_container.to_hex()
            ),
        })
        .collect::<Vec<_>>()
        .join("");

    let final_string = format!("{}{}", base_theme_string, custom_colors_string);

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
                }),
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
