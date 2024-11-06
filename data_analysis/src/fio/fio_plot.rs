use charming::component::FilterMode;
use charming::datatype::{CompositeValue, NumericValue};
use charming::df;
use charming::element::{AreaStyle, ItemStyle, Symbol, Trigger};
use charming::series::Line;
use charming::{component::Axis, element::Tooltip, series::Scatter, Chart};
use charming::{component::DataZoom, datatype::DataPointItem, element::Formatter};
use charming::{element::AxisType, series::Bar};

use super::ec2_etl::{InstanceData, MetricData};

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

pub enum TestType {
    SequentialRead,
    SequentialWrite,
    RealWorld,
}

pub struct MetricDataJson {
    pub bw: String,
    pub iops: String,
    pub lat: String,
    pub clat: String,
}

pub fn get_instance_metric_data(
    instance: &str,
    raid_level: u8,
    test_type: TestType,
) -> MetricDataJson {
    let test_type_str = match test_type {
        TestType::SequentialRead => "seq_read",
        TestType::SequentialWrite => "seq_write",
        TestType::RealWorld => "real_world",
    };
    let instance = "d3en.xlarge";
    let data = MetricData::new(&format!(
        "{}/RAID{}/{}",
        instance, raid_level, test_type_str
    ));
    let bw = get_metric_chart(data.bw);
    let iops = get_metric_chart(data.iops);
    let lat = get_metric_chart(data.lat);
    let clat = get_metric_chart(data.clat);
    MetricDataJson {
        bw,
        iops,
        lat,
        clat,
    }
}
