use charming::component::FilterMode;
use charming::df;
use charming::element::{ItemStyle};
use charming::{component::Axis, element::Tooltip, series::Scatter, Chart};
use charming::{component::DataZoom, datatype::DataPointItem, element::Formatter};
use charming::{element::AxisType, series::Bar};
use super::vantage_etl::InstanceData;


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

