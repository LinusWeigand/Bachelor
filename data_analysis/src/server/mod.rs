pub mod server_etl;
pub mod server_plot;
pub mod utils;

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use server_plot::get_instance_metric_data;
use server_plot::MetricDataJson;
use tokio::main;
use tower_http::compression::CompressionLayer;

const DURATION: f64 = 239.;
const CORE_COUNT: usize = 16;

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
fn generate_chart_set_html(title: &str, chart_prefix: &str, data: &MetricDataJson) -> String {
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
                <h2>GB Send</h2>
                <div id="{chart_prefix}-received" class="chart-container"></div>
            </div>
            <div class="chart-item">
                <h2>GB Received</h2>
                <div id="{chart_prefix}-send" class="chart-container"></div>
            </div>

            <div class="chart-item">
                <h2>Network IO</h2>
                <div id="{chart_prefix}-net-io" class="chart-container"></div>
            </div>
            <div class="chart-item">
                <h2>Disk IO</h2>
                <div id="{chart_prefix}-disk-io" class="chart-container"></div>
            </div>
            <div class="chart-item">
                <h2>Network IO: Send</h2>
                <div id="{chart_prefix}-net-io-send" class="chart-container"></div>
            </div>
            <div class="chart-item">
                <h2>Network IO: Receive</h2>
                <div id="{chart_prefix}-net-io-receive" class="chart-container"></div>
            </div>

           
        </div>
        <script>
            const chart{chart_prefix}_cpu = initChart('{chart_prefix}-cpu', {cpu});
            const chart{chart_prefix}_ram = initChart('{chart_prefix}-ram', {ram});
            const chart{chart_prefix}_send = initChart('{chart_prefix}-send', {send});
            const chart{chart_prefix}_received = initChart('{chart_prefix}-received', {received});

            const chart{chart_prefix}_net_io = initChart('{chart_prefix}-net-io', {net_io});
            const chart{chart_prefix}_disk_io = initChart('{chart_prefix}-disk-io', {disk_io});
            const chart{chart_prefix}_net_io_send = initChart('{chart_prefix}-net-io-send', {send_io});
            const chart{chart_prefix}_net_io_receive = initChart('{chart_prefix}-net-io-receive', {receive_io});

            window.addEventListener('resize', function() {{
                chart{chart_prefix}_cpu.resize();
                chart{chart_prefix}_ram.resize();
                chart{chart_prefix}_send.resize();
                chart{chart_prefix}_received.resize();

                chart{chart_prefix}_net_io.resize();
                chart{chart_prefix}_disk_io.resize();
                chart{chart_prefix}_net_io_send.resize();
                chart{chart_prefix}_net_io_receive.resize();
            }});
        </script>
        "#,
        title,
        chart_prefix = chart_prefix,
        cpu = data.cpu,
        ram = data.ram,
        send = data.send,
        received = data.received,
        net_io = data.net_io,
        disk_io = data.disk_io,
        send_io = data.send_io,
        receive_io = data.receive_io,
    )
}

async fn index() -> impl IntoResponse {
    // let v1_send = get_instance_metric_data(1, "send");
    // let v2_send1 = get_instance_metric_data(2, "send");
    // let v2_send2 = get_instance_metric_data(2, "send2");
    // let v2_send3 = get_instance_metric_data(2, "send3");
    // let v2_send4 = get_instance_metric_data(2, "send4");
    // let idle = get_instance_metric_data(100, "idle");
    // let send4 = get_instance_metric_data(2, "send4");
    // let send6 = get_instance_metric_data(2, "send6");
    // let curl_32G = get_instance_metric_data(2, "curl_32G");
    // let gimp_100 = get_instance_metric_data(2, "gimp_100");
    // let iperf3 = get_instance_metric_data(3, "own");
    let own = get_instance_metric_data(96, "send4");

    let chart_sets_html = vec![
        // generate_chart_set_html(
        //     "v1: 5000 clients, 5.5MB file",
        //     "chart0",
        //     &v1_send,
        // ),
        // generate_chart_set_html(
        //     "v2: 5000 clients, 5.5MB file",
        //     "chart1",
        //     &v2_send1,
        // ),
        // generate_chart_set_html("v2: 5000 clients, 5.5MB file", "chart2", &v2_send4),
        generate_chart_set_html("", "chart1", &own),
        // generate_chart_set_html("v2: curl, 1 clients, 32G file", "chart1", &send6),
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
                    grid-template-rows: repeat(4, 1fr);
                    gap: 10px;
                    width: 100%;
                    height: 200%;
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
                        console.log('options undefined for chart: ', containerId);
                    }}
                    console.log(containerId);
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
