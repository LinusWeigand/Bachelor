use charming::component::FilterMode;
use charming::df;
use charming::element::ItemStyle;
use charming::{component::DataZoom, datatype::DataPointItem, element::Formatter};

use charming::{element::AxisType, series::Bar};

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use charming::{component::Axis, element::Tooltip, series::Scatter, Chart, HtmlRenderer};
use tokio::main;
use tower_http::compression::CompressionLayer;
use utils::Data;
mod utils;

const CHUNK_SIZE: usize = 1000;

#[main]
async fn main() {
    println!("Runinng: http://127.0.0.1:5555");

    let compression_layer = CompressionLayer::new().br(true).gzip(true).deflate(true);

    let app = Router::new().route("/", get(index))
    .layer(compression_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5555").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn get_scatter_throughput_chart(data: &Data) -> String {
    let data = data.get_cost_throughput();
    let chart = Chart::new()
        .x_axis(Axis::new().name("Linux Reserved Price"))
        .y_axis(Axis::new().name("Baseline Throughput (Mbps)"))
        .tooltip(Tooltip::new().formatter(Formatter::String("{b}: ({c0})".into())))
        .data_zoom(DataZoom::new()
            .x_axis_index(0)
            .y_axis_index(0)
            .realtime(false)
        )
        .series(
            Scatter::new()
            .symbol_size(5)
            .data(
                data.into_iter()
                    .map(|(label, x, y)| DataPointItem::new(vec![x, y]).name(label))
                    .collect::<Vec<_>>(),
            ),
        );
    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

fn get_scatter_efficient_frontier(data: &Data) -> String {
    let data = data.get_efficient_frontier();
    let chart = Chart::new()
        .x_axis(Axis::new().name("Total Cost Per Month"))
        .y_axis(Axis::new().name("Baseline Throughput (Mbps)"))
        .tooltip(Tooltip::new().formatter(Formatter::String("{b}: ({c0})".into())))
        .data_zoom(DataZoom::new()
            .x_axis_index(0)
            .y_axis_index(0)
            .realtime(false)
        )
        .series(
            Scatter::new().symbol_size(5).data(
                data.into_iter()
                    .map(|(label, x, y)| DataPointItem::new(vec![x, y]).name(label))
                    .collect::<Vec<_>>(),
            ),
        );
    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

fn get_bar_throughput_chart(data: &Data) -> String {
    let data = data.get_cost_per_throughput();
    let (names, values): (Vec<String>, Vec<f64>) = data.into_iter().map(|(s, f)| (s, f)).unzip();
    let chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .name("EC2 Instance Type")
                .data(names),
        )
        .y_axis(
            Axis::new()
                .type_(AxisType::Value)
                .name("Baseline Throughput Per Dollar (Mbps)")
                .scale(true),
        )
        .tooltip(Tooltip::new())
        .data_zoom(
            DataZoom::new()
                .x_axis_index(0)
                .y_axis_index(0)
                .realtime(false)
                .filter_mode(FilterMode::None)
                .start(0)
                .end(100),
        )
        .series(Bar::new().data(values));

    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

fn get_bar_ebs_chart() -> String {
    let chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .name("Name")
                .data(vec![
                    "Io1",
                    "Io2",
                    "Gp2",
                    "Gp3",
                    "St1",
                    "S3 bis 50TB",
                    "S3 weitere 450TB",
                    "S3 ab 500TB",
                    "Sc1",
                ]),
        )
        .y_axis(Axis::new().type_(AxisType::Value).name("Cost per GB"))
        .tooltip(Tooltip::new())
        .data_zoom(DataZoom::new()
            .x_axis_index(0)
            .y_axis_index(0)
            .realtime(false))
        .series(Bar::new().data(df![
            0.1311,
            0.1311,
            0.1045,
            0.0836,
            0.0475,
            DataPointItem::new(0.023).item_style(ItemStyle::new().color("#a90000")),
            DataPointItem::new(0.022).item_style(ItemStyle::new().color("#a90000")),
            DataPointItem::new(0.021).item_style(ItemStyle::new().color("#a90000")),
            0.01596,
        ]));

    let options = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".to_string());
    options
}

fn get_bar_storage_chart(data: &Data) -> String {
    let data = data.get_cost_per_gb();
    let (names, values): (Vec<String>, Vec<f64>) = data.into_iter().map(|(s, f)| (s, f)).unzip();
    let chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .name("EC2 Instance Type")
                .data(names),
        )
        .y_axis(Axis::new().type_(AxisType::Value).name("Storage (GB)"))
        .tooltip(Tooltip::new())
        .data_zoom(DataZoom::new()
            .x_axis_index(0)
            .y_axis_index(0)
            .realtime(true)
        )
        .series(Bar::new().data(values));

    let options = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".to_string());
    options
}

async fn index() -> impl IntoResponse {
    let data = Data::new();
    let scatter_options = get_scatter_throughput_chart(&data);
    let bar_throughput_options = get_bar_throughput_chart(&data);
    let bar_storage_options = get_bar_storage_chart(&data);
    let ebs_options = get_bar_ebs_chart();
    let scatter_efficient_frontier = get_scatter_efficient_frontier(&data);

    let combined_html = format!(
        r#"
        <html>
        <head>
            <meta charset="UTF-8">
            <title>Combined Charts</title>
            <!-- Include ECharts library -->
            <script src="https://cdn.jsdelivr.net/npm/echarts/dist/echarts.min.js"></script>
            <style>
                html, body {{
                    margin: 0;
                    padding: 0;
                    height: 100%;
                }}
                .chart-container {{
                    width: 100%;
                    height: 100%; 
                }}
                h1 {{
                    text-align: center;
                    font-family: Arial, sans-serif;
                }}
                p {{
                    text-align: left;
                    font-family: Arial, sans-serif;
                    text-size: 15px;
                    margin-top: 50px;
                }}
            </style>
        </head>
        <body>
            <h1>Throughput and Price</h1>
            <div id="chart1-container" class="chart-container"></div>
            <h1>Throughput per Dollar (1 MB/s)</h1>
            <div id="chart2-container" class="chart-container"></div>
            <h1>EC2 Cost per GB</h1>
            <div id="chart3-container" class="chart-container"></div>
            <h1>EBS Cost per GB</h1>
            <div id="chart4-container" class="chart-container"></div>
            <h1>Throughput & Cost Efficient Frontier</h1>
            <div id="chart5-container" class="chart-container"></div>
            

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

                    chart.setOptions(options);
                    return chart
                }}
                const chart1 = initChart('chart1-container', {scatter_options});
                const chart2 = initChart('chart2-container', {bar_throughput_options});
                const chart3 = initChart('chart3-container', {bar_storage_options});
                const chart4 = initChart('chart4-container', {ebs_options});
                const chart5 = initChart('chart5-container', {scatter_efficient_frontier});

                let resizeTimeout;
                window.addEventListener('resize', function() {{
                    if(resizeTimeout) {{
                        clearTimeout(resizeTimout);
                    }}
                    resizeTimout = setTimeout(() => {{
                        chart1.resize();
                        chart2.resize();
                        chart3.resize();
                        chart4.resize();
                        chart5.resize();
                    }}, 100)
                }});
            </script>
        </body>
        </html>
        "#
    );

    Html(combined_html).into_response()
}
