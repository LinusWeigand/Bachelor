pub mod snowset_etl;
pub mod snowset_plot;

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
// use snowset_etl::get_avg_scan_bytes_per_estimated_size;
use snowset_plot::{get_avg_query_size_chart, get_buckets};
use tokio::main;
use tower_http::compression::CompressionLayer;

#[main]
async fn main() {
    println!("cargo::rustc-link-searh=native=/opt/homebrew/opt/duckdb/lib/");
    println!("cargo::rustc-link-lib=dylib=duckdb");
    println!("Runinng: http://127.0.0.1:5555");

    let compression_layer = CompressionLayer::new().br(true).gzip(true).deflate(true);
    let app = Router::new()
        .route("/", get(index))
        .layer(compression_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5555").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
fn generate_chart_set_html(title: &str, chart_prefix: &str, data: &str) -> String {
    format!(
        r#"
        <h1>{}</h1>
        <div id="{chart_prefix}-container" class="chart-container"></div>

       <script>
            const chart{chart_prefix} = initChart('{chart_prefix}-container', {data});

            window.addEventListener('resize', function() {{
                chart{chart_prefix}.resize();
            }});
        </script>
        "#,
        title,
        chart_prefix = chart_prefix,
        data = data,
    )
}

async fn index() -> impl IntoResponse {
    // let avg_query_size = get_avg_scan_bytes_per_estimated_size().unwrap();
    // let avg_query_size_chart = get_avg_query_size_chart(avg_query_size);
    let bucket_chart = get_buckets();

    let chart_sets_html = vec![generate_chart_set_html(
        "Queries in each Percentile",
        "chart1",
        &bucket_chart,
    )]
    .join("\n");

    let combined_html = format!(
        r#"
        <html>
        <head>
            <meta charset="UTF-8">
            <title>Combined Charts</title>
            <script src="https://cdn.jsdelivr.net/npm/echarts/dist/echarts.min.js"></script>
            <style>
                html, body {{
                    margin: 0;
                    padding: 0;
                    height: 100%;
                }}
                .grid-container {{
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    grid-template-rows: 1fr 1fr;
                    gap: 10px;
                    width: 100%;
                    height: 100%;
                }}
                .chart-item {{
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    width: 100%;
                    height: 100%;
                }}
                .chart-container {{
                    height: 100%;
                    width: 100%;
                }}
                h1 {{
                    text-align: center;
                    font-family: Arial, sans-serif;
                }}
                h2 {{
                    margin: 10px 0;
                    font-family: Arial, sans-serif;
                    text-align: center;
                }}
            </style>
        </head>
        <body>
                       <script>
                const initChart = (containerId, options) => {{
                    const chart = echarts.init(document.getElementById(containerId), null, {{
                        renderer: 'canvas',
                        useDirtyRect: true
                    }});
                    options.series[0].large = true;
                    options.series[0].largeThreshold = 1000;
                    options.series[0].progressive = 1000;
                    options.series[0].progressiveThreshold = 1000;
                    chart.setOption(options);
                    return chart;
                }};
            </script>
            {chart_sets_html}


        </body>
        </html>
        "#,
        chart_sets_html = chart_sets_html
    );

    Html(combined_html).into_response()
}
