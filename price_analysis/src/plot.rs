use charming::{datatype::DataPointItem, element::Formatter};

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use charming::{component::Axis, element::Tooltip, series::Scatter, Chart, HtmlRenderer};
use tokio::main;
use utils::Data;
mod utils;

#[main]
async fn main() {
    println!("Access the chart at: http://127.0.0.1:5555");

    let app = Router::new()
        .route("/", get(index)) 
        .route("/chart", get(render_chart)); 


    let listener = tokio::net::TcpListener::bind("0.0.0.0:5555").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    HtmlTemplate {
        body: r#"
            <div>
                <h1>Chart Example</h1>
                <button onclick="window.location.href='/chart'">Show Chart</button>
                <div id="chart"></div>
            </div>
        "#,
    }
}

async fn render_chart() -> impl IntoResponse {
    let data = Data::new();
    let chart = create_chart(data.get_cost_throughput()); 

    let renderer = HtmlRenderer::new("Scatter Chart", 600, 400);
    match renderer.render(&chart) {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render chart").into_response(),
    }
}

pub fn create_chart(data: Vec<(String, f64, f64)>) -> Chart {
    Chart::new()
        .x_axis(Axis::new())
        .y_axis(Axis::new())
        .tooltip(
            Tooltip::new().formatter(Formatter::String("{b}: ({c0})".into())), 
        )
        .series(
            Scatter::new()
                .symbol_size(20)
                .data(
                    data.into_iter()
                        .map(|(label, x, y)| DataPointItem::new(vec![x, y]).name(label))  
                        .collect::<Vec<_>>(),
                ),
        )
}



struct HtmlTemplate<'a> {
    body: &'a str,
}

impl<'a> IntoResponse for HtmlTemplate<'a> {
    fn into_response(self) -> Response {
        Html(format!(
            r#"<!DOCTYPE html>
            <html lang="en">
            <head><meta charset="UTF-8"><title>Chart Example</title></head>
            <body>{}</body>
            </html>"#,
            self.body
        ))
        .into_response()
    }
}
