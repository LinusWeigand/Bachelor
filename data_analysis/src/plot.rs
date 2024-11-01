use charming::component::{FilterMode, Title};
use charming::datatype::{CompositeValue, DataPoint, Dataset, NumericValue, Source, Transform};
use charming::df;
use charming::element::{
    AreaStyle, DimensionEncode, ItemStyle, LineStyle, NameLocation, Symbol, Trigger,
};
use charming::series::Line;
use charming::{component::DataZoom, datatype::DataPointItem, element::Formatter};

use charming::{element::AxisType, series::Bar};

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use charming::{component::Axis, element::Tooltip, series::Scatter, Chart};
use serde_json::json;
use tokio::main;
use tower_http::compression::CompressionLayer;
use utils::{InstanceData, MetricData};
mod utils;

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

fn calculate_moving_average(data: &[(f64, f64)], window_size: usize) -> Vec<(f64, f64)> {
    let mut moving_average = Vec::new();
    let mut sum = 0.0;

    for (i, &(_, y)) in data.iter().enumerate() {
        sum += y;
        if i >= window_size {
            sum -= data[i - window_size].1;
            moving_average.push((data[i].0, sum / window_size as f64));
        } else {
            moving_average.push((data[i].0, sum / (i + 1) as f64));
        }
    }

    moving_average
}

pub fn get_metric_chart(metric_data: Vec<(f64, f64)>) -> String {
    let data_set: Vec<CompositeValue> = metric_data
        .clone()
        .into_iter()
        .map(|(x, y)| {
            CompositeValue::Array(vec![
                CompositeValue::Number(NumericValue::from(x)),
                CompositeValue::Number(NumericValue::from(y)),
            ])
        })
        .collect();
    let moving_average_data = calculate_moving_average(&metric_data, 60);
    let moving_avg_set: Vec<CompositeValue> = moving_average_data
        .iter()
        .map(|&(x, y)| {
            CompositeValue::Array(vec![
                CompositeValue::Number(NumericValue::from(x)),
                CompositeValue::Number(NumericValue::from(y)),
            ])
        })
        .collect();
    let chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Value)
                .boundary_gap(false)
                .min(0)
                .max(60000),
        )
        .y_axis(Axis::new().type_(AxisType::Value).boundary_gap(false))
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .series(
            Line::new()
                .name("Original")
                .area_style(AreaStyle::new())
                .data(data_set)
                .symbol(Symbol::None),
        )
        .series(
            Line::new()
                .name("Moving Average")
                .data(moving_avg_set)
                .item_style(ItemStyle::new().color("#FF0000"))
                .smooth(true)
                .symbol(Symbol::None),
        );

    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

fn get_scatter_throughput_chart(instance_data: &InstanceData) -> String {
    let data = instance_data.get_cost_throughput();
    let chart = Chart::new()
        .x_axis(Axis::new().name("Linux Reserved Price"))
        .y_axis(Axis::new().name("Baseline Throughput (MBps)"))
        .tooltip(Tooltip::new().formatter(Formatter::String("{b}: ({c0})".into())))
        .data_zoom(DataZoom::new().x_axis_index(0).y_axis_index(0))
        .series(
            Scatter::new().symbol_size(10).data(
                data.into_iter()
                    .map(|(label, x, y)| DataPointItem::new(vec![x, y]).name(label))
                    .collect::<Vec<_>>(),
            ),
        );
    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

fn get_scatter_efficient_frontier(instance_data: &InstanceData) -> String {
    let data = instance_data.get_efficient_frontier();
    let chart = Chart::new()
        .x_axis(Axis::new().name("Total Cost Per Month"))
        .y_axis(Axis::new().name("Baseline Throughput (MBps)"))
        .tooltip(Tooltip::new().formatter(Formatter::String("{b}: ({c0})".into())))
        .data_zoom(
            DataZoom::new()
                .x_axis_index(0)
                .y_axis_index(0)
                .realtime(true),
        )
        .series(
            Scatter::new().symbol_size(10).data(
                data.into_iter()
                    .map(|(label, x, y)| DataPointItem::new(vec![x, y]).name(label))
                    .collect::<Vec<_>>(),
            ),
        );
    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

fn get_bar_throughput_chart(instance_data: &InstanceData) -> String {
    let data = instance_data.get_cost_per_throughput();
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
                .name("Baseline Throughput Per Dollar (MBps)")
                .scale(true),
        )
        .tooltip(Tooltip::new())
        .data_zoom(
            DataZoom::new()
                .x_axis_index(0)
                .y_axis_index(0)
                .filter_mode(FilterMode::None)
                .start(0)
                .end(100),
        )
        .series(Bar::new().data(values));

    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}
fn get_bar_network_performance_per_gb_chart(instance_data: &InstanceData) -> String {
    let data = instance_data.get_network_performance_per_gb();
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
                .name("Network Performance per GB (GBps)")
                .scale(true),
        )
        .tooltip(Tooltip::new())
        .data_zoom(
            DataZoom::new()
                .x_axis_index(0)
                .y_axis_index(0)
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
                    "d3en",
                ]),
        )
        .y_axis(Axis::new().type_(AxisType::Value).name("Cost per GB"))
        .tooltip(Tooltip::new())
        .data_zoom(DataZoom::new().x_axis_index(0).y_axis_index(0))
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
            0.0113834
        ]));

    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

fn get_bar_storage_chart(instance_data: &InstanceData) -> String {
    let data = instance_data.get_cost_per_gb();
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
        .data_zoom(DataZoom::new().x_axis_index(0).y_axis_index(0))
        .series(Bar::new().data(values));

    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

async fn index() -> impl IntoResponse {
    let data = InstanceData::new();

    let scatter_options = get_scatter_throughput_chart(&data);
    let bar_throughput_options = get_bar_throughput_chart(&data);
    let bar_storage_options = get_bar_storage_chart(&data);
    let ebs_options = get_bar_ebs_chart();
    let scatter_efficient_frontier = get_scatter_efficient_frontier(&data);
    let network_performance_per_gb_options = get_bar_network_performance_per_gb_chart(&data);

    let raid0_seq_read_tg4nano_data = MetricData::new("tg4.nano/RAID0/seq_read");
    let raid0_seq_read_t4gnano_throughput = get_metric_chart(raid0_seq_read_tg4nano_data.bw);
    let raid0_seq_read_t4gnano_iops = get_metric_chart(raid0_seq_read_tg4nano_data.iops);
    let raid0_seq_read_t4gnano_lat = get_metric_chart(raid0_seq_read_tg4nano_data.lat);
    let raid0_seq_read_t4gnano_clat = get_metric_chart(raid0_seq_read_tg4nano_data.clat);


    // Seq Read 1
    let raid1_seq_read_d3enxlarge_data = MetricData::new("d3en.xlarge/RAID1/seq_read");
    let raid1_seq_read_d3enxlarge_throughput = get_metric_chart(raid1_seq_read_d3enxlarge_data.bw);
    let raid1_seq_read_d3enxlarge_iops = get_metric_chart(raid1_seq_read_d3enxlarge_data.iops);
    let raid1_seq_read_d3enxlarge_lat = get_metric_chart(raid1_seq_read_d3enxlarge_data.lat);
    let raid1_seq_read_d3enxlarge_clat = get_metric_chart(raid1_seq_read_d3enxlarge_data.clat);

    // Seq Write 2
    let raid1_seq_write_d3enxlarge_data = MetricData::new("d3en.xlarge/RAID1/seq_write");
    let raid1_seq_write_d3enxlarge_throughput = get_metric_chart(raid1_seq_write_d3enxlarge_data.bw);
    let raid1_seq_write_d3enxlarge_iops = get_metric_chart(raid1_seq_write_d3enxlarge_data.iops);
    let raid1_seq_write_d3enxlarge_lat = get_metric_chart(raid1_seq_write_d3enxlarge_data.lat);
    let raid1_seq_write_d3enxlarge_clat = get_metric_chart(raid1_seq_write_d3enxlarge_data.clat);

    // Rand Read 3
    let raid1_rand_read_d3enxlarge_data = MetricData::new("d3en.xlarge/RAID1/rand_read");
    let raid1_rand_read_d3enxlarge_throughput = get_metric_chart(raid1_rand_read_d3enxlarge_data.bw);
    let raid1_rand_read_d3enxlarge_iops = get_metric_chart(raid1_rand_read_d3enxlarge_data.iops);
    let raid1_rand_read_d3enxlarge_lat = get_metric_chart(raid1_rand_read_d3enxlarge_data.lat);
    let raid1_rand_read_d3enxlarge_clat = get_metric_chart(raid1_rand_read_d3enxlarge_data.clat);

    // Rand Write 4
    let raid1_rand_write_d3enxlarge_data = MetricData::new("d3en.xlarge/RAID1/rand_write");
    let raid1_rand_write_d3enxlarge_throughput = get_metric_chart(raid1_rand_write_d3enxlarge_data.bw);
    let raid1_rand_write_d3enxlarge_iops = get_metric_chart(raid1_rand_write_d3enxlarge_data.iops);
    let raid1_rand_write_d3enxlarge_lat = get_metric_chart(raid1_rand_write_d3enxlarge_data.lat);
    let raid1_rand_write_d3enxlarge_clat = get_metric_chart(raid1_rand_write_d3enxlarge_data.clat);

    // Rand Mixed 5
    let raid1_rand_mixed_d3enxlarge_data = MetricData::new("d3en.xlarge/RAID1/rand_mix");
    let raid1_rand_mixed_d3enxlarge_throughput = get_metric_chart(raid1_rand_mixed_d3enxlarge_data.bw);
    let raid1_rand_mixed_d3enxlarge_iops = get_metric_chart(raid1_rand_mixed_d3enxlarge_data.iops);
    let raid1_rand_mixed_d3enxlarge_lat = get_metric_chart(raid1_rand_mixed_d3enxlarge_data.lat);
    let raid1_rand_mixed_d3enxlarge_clat = get_metric_chart(raid1_rand_mixed_d3enxlarge_data.clat);

    // Seq Mixed 6
    let raid1_seq_mixed_d3enxlarge_data = MetricData::new("d3en.xlarge/RAID1/seq_mix");
    let raid1_seq_mixed_d3enxlarge_throughput = get_metric_chart(raid1_seq_mixed_d3enxlarge_data.bw);
    let raid1_seq_mixed_d3enxlarge_iops = get_metric_chart(raid1_seq_mixed_d3enxlarge_data.iops);
    let raid1_seq_mixed_d3enxlarge_lat = get_metric_chart(raid1_seq_mixed_d3enxlarge_data.lat);
    let raid1_seq_mixed_d3enxlarge_clat = get_metric_chart(raid1_seq_mixed_d3enxlarge_data.clat);


    // Rand Mixed
    let raid5_rand_mixed_d3enxlarge_data = MetricData::new("d3en.xlarge/RAID5/rand_mix");
    let raid5_rand_mixed_d3enxlarge_throughput = get_metric_chart(raid5_rand_mixed_d3enxlarge_data.bw);
    let raid5_rand_mixed_d3enxlarge_iops = get_metric_chart(raid5_rand_mixed_d3enxlarge_data.iops);
    let raid5_rand_mixed_d3enxlarge_lat = get_metric_chart(raid5_rand_mixed_d3enxlarge_data.lat);
    let raid5_rand_mixed_d3enxlarge_clat = get_metric_chart(raid5_rand_mixed_d3enxlarge_data.clat);

    // Seq Mixed
    let raid5_seq_mixed_d3enxlarge_data = MetricData::new("d3en.xlarge/RAID5/seq_mix");
    let raid5_seq_mixed_d3enxlarge_throughput = get_metric_chart(raid5_seq_mixed_d3enxlarge_data.bw);
    let raid5_seq_mixed_d3enxlarge_iops = get_metric_chart(raid5_seq_mixed_d3enxlarge_data.iops);
    let raid5_seq_mixed_d3enxlarge_lat = get_metric_chart(raid5_seq_mixed_d3enxlarge_data.lat);
    let raid5_seq_mixed_d3enxlarge_clat = get_metric_chart(raid5_seq_mixed_d3enxlarge_data.clat);



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

            <h1>Network Performance per GB</h1>
            <div id="chart6-container" class="chart-container"></div>


            <h1>RAID 0</h1>
            <h2>1 tg4.nano & 2 Sc1 with 125Gib each</h2>
                <div class="grid-container">
                    <div class="chart-item">
                        <h2>Throughput (kB/s)</h2>
                        <div id="chart7-container" class="chart-container"></div>
                    </div>

                    <div class="chart-item">
                        <h2>IOPS</h2>
                        <div id="chart8-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Total Latency (millis)</h2>
                        <div id="chart9-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Completion Latency (millis)</h2>
                        <div id="chart10-container" class="chart-container"></div>
                    </div>
                </div>



            <hr>
            <h1>RAID 1</h1>
            <h2> d3en.xlarge Seq Read</h2>
                <div class="grid-container">
                    <div class="chart-item">
                        <h2>Throughput (kB/s)</h2>
                        <div id="chart11-container" class="chart-container"></div>
                    </div>

                    <div class="chart-item">
                        <h2>IOPS</h2>
                        <div id="chart12-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Total Latency (millis)</h2>
                        <div id="chart13-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Completion Latency (millis)</h2>
                        <div id="chart14-container" class="chart-container"></div>
                    </div>
                </div>

            <h2> d3en.xlarge Rand Read</h2>
                <div class="grid-container">
                    <div class="chart-item">
                        <h2>Throughput (kB/s)</h2>
                        <div id="chart15-container" class="chart-container"></div>
                    </div>

                    <div class="chart-item">
                        <h2>IOPS</h2>
                        <div id="chart16-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Total Latency (millis)</h2>
                        <div id="chart17-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Completion Latency (millis)</h2>
                        <div id="chart18-container" class="chart-container"></div>
                    </div>
                </div>

            <h2> d3en.xlarge Seq Write</h2>
                <div class="grid-container">
                    <div class="chart-item">
                        <h2>Throughput (kB/s)</h2>
                        <div id="chart19-container" class="chart-container"></div>
                    </div>

                    <div class="chart-item">
                        <h2>IOPS</h2>
                        <div id="chart20-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Total Latency (millis)</h2>
                        <div id="chart21-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Completion Latency (millis)</h2>
                        <div id="chart22-container" class="chart-container"></div>
                    </div>
                </div>
            
            <h2> d3en.xlarge Rand Write</h2>
                <div class="grid-container">
                    <div class="chart-item">
                        <h2>Throughput (kB/s)</h2>
                        <div id="chart23-container" class="chart-container"></div>
                    </div>

                    <div class="chart-item">
                        <h2>IOPS</h2>
                        <div id="chart24-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Total Latency (millis)</h2>
                        <div id="chart25-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Completion Latency (millis)</h2>
                        <div id="chart26-container" class="chart-container"></div>
                    </div>
                </div>
            <h2> d3en.xlarge Rand Read/Write (80/20)</h2>
                <div class="grid-container">
                    <div class="chart-item">
                        <h2>Throughput (kB/s)</h2>
                        <div id="chart27-container" class="chart-container"></div>
                    </div>

                    <div class="chart-item">
                        <h2>IOPS</h2>
                        <div id="chart28-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Total Latency (millis)</h2>
                        <div id="chart29-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Completion Latency (millis)</h2>
                        <div id="chart30-container" class="chart-container"></div>
                    </div>
                </div>

                <h2> d3en.xlarge Seq Read/Write (80/20)</h2>
                <div class="grid-container">
                    <div class="chart-item">
                        <h2>Throughput (kB/s)</h2>
                        <div id="chart31-container" class="chart-container"></div>
                    </div>

                    <div class="chart-item">
                        <h2>IOPS</h2>
                        <div id="chart32-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Total Latency (millis)</h2>
                        <div id="chart33-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Completion Latency (millis)</h2>
                        <div id="chart34-container" class="chart-container"></div>
                    </div>
                </div>




            <hr>
            <h1>RAID 5</h1>
            <h2> d3en.xlarge Rand Read/Write (80/20)</h2>
                <div class="grid-container">
                    <div class="chart-item">
                        <h2>Throughput (kB/s)</h2>
                        <div id="chart35-container" class="chart-container"></div>
                    </div>

                    <div class="chart-item">
                        <h2>IOPS</h2>
                        <div id="chart36-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Total Latency (millis)</h2>
                        <div id="chart37-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Completion Latency (millis)</h2>
                        <div id="chart38-container" class="chart-container"></div>
                    </div>
                </div>

            <h2> d3en.xlarge Seq Read/Write (80/20)</h2>
                <div class="grid-container">
                    <div class="chart-item">
                        <h2>Throughput (kB/s)</h2>
                        <div id="chart39-container" class="chart-container"></div>
                    </div>

                    <div class="chart-item">
                        <h2>IOPS</h2>
                        <div id="chart40-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Total Latency (millis)</h2>
                        <div id="chart41-container" class="chart-container"></div>
                    </div>
                    <div class="chart-item">
                        <h2>Completion Latency (millis)</h2>
                        <div id="chart42-container" class="chart-container"></div>
                    </div>
                </div>


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
                const chart1 = initChart('chart1-container', {scatter_options});
                const chart2 = initChart('chart2-container', {bar_throughput_options});
                const chart3 = initChart('chart3-container', {bar_storage_options});
                const chart4 = initChart('chart4-container', {ebs_options});
                const chart5 = initChart('chart5-container', {scatter_efficient_frontier});
                const chart6 = initChart('chart6-container', {network_performance_per_gb_options});

                const chart7 = initChart('chart7-container', {raid0_seq_read_t4gnano_throughput});
                const chart8 = initChart('chart8-container', {raid0_seq_read_t4gnano_iops});
                const chart9 = initChart('chart9-container', {raid0_seq_read_t4gnano_lat});
                const chart10 = initChart('chart10-container', {raid0_seq_read_t4gnano_clat});


                const chart11 = initChart('chart11-container', {raid1_seq_read_d3enxlarge_throughput});
                const chart12 = initChart('chart12-container', {raid1_seq_read_d3enxlarge_iops});
                const chart13 = initChart('chart13-container', {raid1_seq_read_d3enxlarge_lat});
                const chart14 = initChart('chart14-container', {raid1_seq_read_d3enxlarge_clat});

                const chart15 = initChart('chart15-container', {raid1_rand_read_d3enxlarge_throughput});
                const chart16 = initChart('chart16-container', {raid1_rand_read_d3enxlarge_iops});
                const chart17 = initChart('chart17-container', {raid1_rand_read_d3enxlarge_lat});
                const chart18 = initChart('chart18-container', {raid1_rand_read_d3enxlarge_clat});

                const chart19 = initChart('chart19-container', {raid1_seq_write_d3enxlarge_throughput});
                const chart20 = initChart('chart20-container', {raid1_seq_write_d3enxlarge_iops});
                const chart21 = initChart('chart21-container', {raid1_seq_write_d3enxlarge_lat});
                const chart22 = initChart('chart22-container', {raid1_seq_write_d3enxlarge_clat});

                const chart23 = initChart('chart23-container', {raid1_rand_write_d3enxlarge_throughput});
                const chart24 = initChart('chart24-container', {raid1_rand_write_d3enxlarge_iops});
                const chart25 = initChart('chart25-container', {raid1_rand_write_d3enxlarge_lat});
                const chart26 = initChart('chart26-container', {raid1_rand_write_d3enxlarge_clat});

                const chart27 = initChart('chart27-container', {raid1_rand_mixed_d3enxlarge_throughput});
                const chart28 = initChart('chart28-container', {raid1_rand_mixed_d3enxlarge_iops});
                const chart29 = initChart('chart29-container', {raid1_rand_mixed_d3enxlarge_lat});
                const chart30 = initChart('chart30-container', {raid1_rand_mixed_d3enxlarge_clat});

                const chart31 = initChart('chart31-container', {raid1_seq_mixed_d3enxlarge_throughput});
                const chart32 = initChart('chart32-container', {raid1_seq_mixed_d3enxlarge_iops});
                const chart33 = initChart('chart33-container', {raid1_seq_mixed_d3enxlarge_lat});
                const chart34 = initChart('chart34-container', {raid1_seq_mixed_d3enxlarge_clat});


                const chart35 = initChart('chart35-container', {raid5_rand_mixed_d3enxlarge_throughput});
                const chart36 = initChart('chart36-container', {raid5_rand_mixed_d3enxlarge_iops});
                const chart37 = initChart('chart37-container', {raid5_rand_mixed_d3enxlarge_lat});
                const chart38 = initChart('chart38-container', {raid5_rand_mixed_d3enxlarge_clat});

                const chart39 = initChart('chart39-container', {raid5_seq_mixed_d3enxlarge_throughput});
                const chart40 = initChart('chart40-container', {raid5_seq_mixed_d3enxlarge_iops});
                const chart41 = initChart('chart41-container', {raid5_seq_mixed_d3enxlarge_lat});
                const chart42 = initChart('chart42-container', {raid5_seq_mixed_d3enxlarge_clat});

                // Resize charts when window size changes
                window.addEventListener('resize', function() {{
                    chart1.resize();
                    chart2.resize();
                    chart3.resize();
                    chart4.resize();
                    chart5.resize();
                    chart6.resize();
                    
                    chart7.resize();
                    chart8.resize();
                    chart9.resize();
                    chart10.resize();


                    chart11.resize();
                    chart12.resize();
                    chart13.resize();
                    chart14.resize();

                    chart15.resize();
                    chart16.resize();
                    chart17.resize();
                    chart18.resize();

                    chart19.resize();
                    chart20.resize();
                    chart21.resize();
                    chart22.resize();

                    chart23.resize();
                    chart24.resize();
                    chart25.resize();
                    chart26.resize();

                    chart27.resize();
                    chart28.resize();
                    chart29.resize();
                    chart30.resize();

                    chart31.resize();
                    chart32.resize();
                    chart33.resize();
                    chart34.resize();


                    chart35.resize();
                    chart36.resize();
                    chart37.resize();
                    chart38.resize();

                    chart39.resize();
                    chart40.resize();
                    chart41.resize();
                    chart42.resize();


                }});
            </script>
        </body>
        </html>
        "#
    );

    Html(combined_html).into_response()
}
