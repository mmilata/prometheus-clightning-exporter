use prometrics::metrics;
use prometrics::Gatherer;

use crate::producer::GetInfoResult;

fn metric_gauge(
    reg: &prometrics::Registry,
    val: &Option<u32>,
    name: &str,
    help: &str,
    resv: &mut Vec<metrics::Gauge>,
) -> anyhow::Result<()> {
    if let Some(ref v) = val {
        let mut m = metrics::GaugeBuilder::new(name);
        m.registry(reg.clone());
        m.namespace("clightning");
        if !help.is_empty() {
            m.help(help);
        }
        let m = m.finish()?;
        m.set(*v as f64);
        resv.push(m);
    }
    Ok(())
}

fn metric_label(b: &mut metrics::GaugeBuilder, val: &Option<String>, name: &str) {
    if let Some(ref v) = val {
        b.label(name, v);
    }
}

pub fn prometheus_format(gir: GetInfoResult) -> anyhow::Result<String> {
    let mut gatherer = Gatherer::new();
    let reg = gatherer.registry();
    let mut gauges = std::vec::Vec::new();

    metric_gauge(&reg, &gir.blockheight, "blockheight", "", &mut gauges)?;
    metric_gauge(&reg, &gir.num_peers, "num_peers", "", &mut gauges)?;
    metric_gauge(
        &reg,
        &gir.num_pending_channels,
        "num_pending_channels",
        "",
        &mut gauges,
    )?;
    metric_gauge(
        &reg,
        &gir.num_active_channels,
        "num_active_channels",
        "",
        &mut gauges,
    )?;
    metric_gauge(
        &reg,
        &gir.num_inactive_channels,
        "num_inactive_channels",
        "",
        &mut gauges,
    )?;
    metric_gauge(&reg, &Some(1), "up", "", &mut gauges)?;

    let mut b = metrics::GaugeBuilder::new("info");
    b.registry(reg.clone()).namespace("clightning");
    metric_label(&mut b, &gir.id, "id");
    metric_label(&mut b, &gir.alias, "alias");
    metric_label(&mut b, &gir.network, "network");
    metric_label(&mut b, &gir.version, "version");
    gauges.push(b.finish()?);

    Ok(gatherer.gather().to_text())
}

pub fn prometheus_format_down() -> anyhow::Result<String> {
    let mut gatherer = Gatherer::new();
    let reg = gatherer.registry();
    let mut gauges = std::vec::Vec::new();
    metric_gauge(&reg, &Some(0), "up", "", &mut gauges)?;
    Ok(gatherer.gather().to_text())
}
