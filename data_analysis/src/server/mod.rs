pub mod server_etl;
pub mod server_plot;
pub mod utils;

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use server_plot::{get_instance_metric_data};
use server_plot::MetricDataJson;
use tokio::main;
use tower_http::compression::CompressionLayer;

const DURATION: f64 = 479.;

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
fn generate_chart_set_html(
    title: &str,
    chart_prefix: &str,
    data: &MetricDataJson,
) -> String {
    format!(
        r#"
        <h2>{}</h2>
        <div class="grid-container">
            <div class="chart-item">
                <h2>vCPU </h2>
                <div id="{chart_prefix}-cpu" class="chart-container"></div>
            </div>
            <div class="chart-item">
                <h2>RAM</h2>
                <div id="{chart_prefix}-ram" class="chart-container"></div>
            </div>
            <div class="chart-item">
                <h2>Bytes Send</h2>
                <div id="{chart_prefix}-send" class="chart-container"></div>
            </div>
            <div class="chart-item">
                <h2>Bytes Received</h2>
                <div id="{chart_prefix}-received" class="chart-container"></div>
            </div>
           
        </div>
        <script>
            const chart{chart_prefix}_cpu = initChart('{chart_prefix}-cpu', {cpu});
            const chart{chart_prefix}_ram = initChart('{chart_prefix}-ram', {ram});
            const chart{chart_prefix}_send = initChart('{chart_prefix}-send', {send});
            const chart{chart_prefix}_received = initChart('{chart_prefix}-received', {received});

            window.addEventListener('resize', function() {{
                chart{chart_prefix}_cpu.resize();
                chart{chart_prefix}_ram.resize();
                chart{chart_prefix}_send.resize();
                chart{chart_prefix}_received.resize();
            }});
        </script>
        "#,
        title,
        chart_prefix = chart_prefix,
        cpu = data.cpu,
        ram = data.ram,
        send = data.send,
        received = data.received,
    )
}

async fn index() -> impl IntoResponse {
    let v0_send = get_instance_metric_data(0, "test2");

    let chart_sets_html = vec![
        generate_chart_set_html(
            "v0: Receive Package into RAM -> write into file",
            "chart1",
            &v0_send,
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
                    if (options === undefined) {{

                        console.log('options undefined for chart: ', containerID);
                    }}
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
