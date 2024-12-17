use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use charming::component::DataZoom;
use charming::component::FilterMode;
use charming::{component::Axis, element::Tooltip, Chart};
use charming::{element::AxisType, series::Bar};
use iteration::load_selectivity_map;
use tokio::main;
use tower_http::compression::CompressionLayer;

mod iteration;

const SELECTIVITY_WRITE_PATH: &str = "./data/selectivity_write";
const SELECTIVITY_READ_PATH: &str = "./data/selectivity_read";

pub fn get_bar_chart(data: Vec<(u32, u32)>, x_label: &str, y_label: &str) -> String {
    let (x_values, y_values): (Vec<String>, Vec<f64>) = data
        .into_iter()
        .map(|(s, f)| (format!("{}", s), f as f64))
        .unzip();

    let chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .name(x_label)
                .data(x_values),
        )
        .y_axis(Axis::new().type_(AxisType::Value).name(y_label).scale(true))
        .tooltip(Tooltip::new())
        .data_zoom(
            DataZoom::new()
                .x_axis_index(0)
                .y_axis_index(0)
                .filter_mode(FilterMode::None)
                .start(0)
                .end(100),
        )
        .series(Bar::new().data(y_values));

    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

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
    let selectivity_map_read = load_selectivity_map(SELECTIVITY_READ_PATH).unwrap();
    let selectivity_map_write = load_selectivity_map(SELECTIVITY_WRITE_PATH).unwrap();

    let mut selectivity_vec_read: Vec<(u32, u32)> = selectivity_map_read
        .iter()
        .map(|(&k, v)| (k, v.count))
        .collect();
    selectivity_vec_read.sort_by_key(|&(k, _)| k);

    let mut selectivity_vec_write: Vec<(u32, u32)> = selectivity_map_write
        .iter()
        .map(|(&k, v)| (k, v.count))
        .collect();
    selectivity_vec_write.sort_by_key(|&(k, _)| k);


    let mut selectivity_vec_read_bw: Vec<(u32, u32)> = selectivity_map_read
        .into_iter()
        .map(|(k, v)| (k, (v.latency as f64 / v.count as f64) as u32))
        .collect();
    selectivity_vec_read_bw.sort_by_key(|&(k, _)| k);

    let mut selectivity_vec_write_bw: Vec<(u32, u32)> = selectivity_map_write
        .into_iter()
        .map(|(k, v)| (k, (v.latency as f64 / v.count as f64) as u32))
        .collect();
    selectivity_vec_write_bw.sort_by_key(|&(k, _)| k);

    let read_count_per_selectivity =
        get_bar_chart(selectivity_vec_read, "Selectivity", "Read Request Count");
    let write_count_per_selectivity =
        get_bar_chart(selectivity_vec_write, "Selectivity", "Write Request Count");

    let read_bw_per_selectivity = get_bar_chart(
        selectivity_vec_read_bw,
        "Selectivity",
        "Read Latency per GB",
    );
    let write_bw_per_selectivity = get_bar_chart(
        selectivity_vec_write_bw,
        "Selectivity",
        "Write Latency per GB",
    );
    let chart_sets_html = vec![
        generate_chart_set_html(
            "Read Count per Selectivity",
            "chart1",
            &read_count_per_selectivity,
        ),
        generate_chart_set_html(
            "Write Count per Selectivity",
            "chart2",
            &write_count_per_selectivity,
        ),
        generate_chart_set_html(
            "Read Latency / GB per Selectivity",
            "chart4",
            &read_bw_per_selectivity,
        ),
        generate_chart_set_html(
            "Write Latency / GB per Selectivity",
            "chart3",
            &write_bw_per_selectivity,
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
