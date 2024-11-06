pub mod fio_etl;
pub mod fio_plot;

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use fio_plot::{get_instance_metric_data, TestType};
use tokio::main;
use tower_http::compression::CompressionLayer;

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
    data: &fio_plot::MetricDataJson,
) -> String {
    format!(
        r#"
        <h2>{}</h2>
        <div class="grid-container">
            <div class="chart-item">
                <h2>Throughput (MB/s)</h2>
                <div id="{chart_prefix}-bw" class="chart-container"></div>
            </div>
            <div class="chart-item">
                <h2>IOPS</h2>
                <div id="{chart_prefix}-iops" class="chart-container"></div>
            </div>
            <div class="chart-item">
                <h2>Total Latency (millis)</h2>
                <div id="{chart_prefix}-lat" class="chart-container"></div>
            </div>
            <div class="chart-item">
                <h2>Completion Latency (millis)</h2>
                <div id="{chart_prefix}-clat" class="chart-container"></div>
            </div>
        </div>
        <script>
            const chart{chart_prefix}_bw = initChart('{chart_prefix}-bw', {bw});
            const chart{chart_prefix}_iops = initChart('{chart_prefix}-iops', {iops});
            const chart{chart_prefix}_lat = initChart('{chart_prefix}-lat', {lat});
            const chart{chart_prefix}_clat = initChart('{chart_prefix}-clat', {clat});

            window.addEventListener('resize', function() {{
                chart{chart_prefix}_bw.resize();
                chart{chart_prefix}_iops.resize();
                chart{chart_prefix}_lat.resize();
                chart{chart_prefix}_clat.resize();
            }});
        </script>
        "#,
        title,
        chart_prefix = chart_prefix,
        bw = data.bw,
        iops = data.iops,
        lat = data.lat,
        clat = data.clat,
    )
}

async fn index() -> impl IntoResponse {
    let sequential_read_data = get_instance_metric_data("d3en.xlarge", 0, TestType::SequentialRead);
    let sequential_write_data =
        get_instance_metric_data("d3en.xlarge", 0, TestType::SequentialWrite);

    let chart_sets_html = vec![
        generate_chart_set_html(
            "d3en xlarge Sequential Reads, 4MB Block size",
            "chart1",
            &sequential_read_data,
        ),
        generate_chart_set_html(
            "d3en xlarge Sequential Writes, 4MB Block size",
            "chart2",
            &sequential_write_data,
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
            <h1>RAID 0</h1>
            {chart_sets_html}


        </body>
        </html>
        "#,
        chart_sets_html = chart_sets_html
    );

    Html(combined_html).into_response()
}
