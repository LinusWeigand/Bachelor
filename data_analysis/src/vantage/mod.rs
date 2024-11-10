pub mod vantage_etl;
pub mod vantage_plot;

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tokio::main;
use tower_http::compression::CompressionLayer;
use vantage_etl::InstanceData;
use vantage_plot::{get_bar_ebs_chart, get_bar_network_performance_per_gb_chart, get_bar_storage_chart, get_bar_throughput_chart, get_scatter_efficient_frontier, get_scatter_throughput_chart};

#[main]
async fn main() {
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
    let data = InstanceData::new();

    let throughput_per_dollar = get_bar_throughput_chart(&data);
    let ec2_price_per_gb = get_bar_storage_chart(&data);
    let ebs_price_per_gb = get_bar_ebs_chart();
    let network_performance_per_gb = get_bar_network_performance_per_gb_chart(&data);
    let chart_sets_html = vec![
        generate_chart_set_html(
            "Throughput per Dollar (1 MB/s)",
            "chart2",
            &throughput_per_dollar
        ),
        generate_chart_set_html(
            "EC2 Cost per GB",
            "chart3",
            &ec2_price_per_gb
        ),
        generate_chart_set_html(
            "EBS Cost per GB",
            "chart4",
            &ebs_price_per_gb
        ),
        generate_chart_set_html(
            "Network Performance per GB",
            "chart6",
            &network_performance_per_gb
        ),
    ]
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
