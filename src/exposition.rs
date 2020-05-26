use prometrics::metrics;
use prometrics::Gatherer;

use crate::producer::LightningMetrics;

fn metric_gauge(
    reg: &prometrics::Registry,
    val: &u64,
    name: &str,
    help: &str,
    resv: &mut Vec<metrics::Gauge>,
) -> anyhow::Result<()> {
    let mut m = metrics::GaugeBuilder::new(name);
    m.registry(reg.clone());
    m.namespace("clightning");
    if !help.is_empty() {
        m.help(help);
    }
    let m = m.finish()?;
    m.set(*val as f64);
    resv.push(m);
    Ok(())
}

pub fn prometheus_format(lm: LightningMetrics) -> anyhow::Result<String> {
    let mut gatherer = Gatherer::new();
    let reg = gatherer.registry();
    let mut gauges = std::vec::Vec::new();

    metric_gauge(&reg, &1, "up", "", &mut gauges)?;
    metric_gauge(
        &reg,
        &lm.getinfo.blockheight,
        "blockheight",
        "",
        &mut gauges,
    )?;
    metric_gauge(&reg, &lm.getinfo.num_peers, "num_peers", "", &mut gauges)?;
    metric_gauge(
        &reg,
        &lm.getinfo.num_pending_channels,
        "num_pending_channels",
        "",
        &mut gauges,
    )?;
    metric_gauge(
        &reg,
        &lm.getinfo.num_active_channels,
        "num_active_channels",
        "",
        &mut gauges,
    )?;
    metric_gauge(
        &reg,
        &lm.getinfo.num_inactive_channels,
        "num_inactive_channels",
        "",
        &mut gauges,
    )?;
    metric_gauge(&reg, &lm.num_nodes, "num_nodes", "", &mut gauges)?;
    metric_gauge(&reg, &lm.num_channels, "num_channels", "", &mut gauges)?;

    for out in lm.listfunds.outputs {
        let mut m = metrics::GaugeBuilder::new("funds_output_sat");
        m.registry(reg.clone());
        m.namespace("clightning");
        m.label("address", &out.address);
        let m = m.finish()?;
        m.set(out.value as f64);
        gauges.push(m);
    }

    for chan in lm.listfunds.channels {
        let mut m = metrics::GaugeBuilder::new("funds_channel_sat");
        m.registry(reg.clone());
        m.namespace("clightning");
        m.label(
            "short_channel_id",
            &chan.short_channel_id.unwrap_or(chan.peer_id),
        );
        let m = m.finish()?;
        m.set(chan.channel_sat as f64);
        gauges.push(m);
    }

    let mut b = metrics::GaugeBuilder::new("info");
    b.registry(reg.clone()).namespace("clightning");
    b.label("id", &lm.getinfo.id);
    b.label("alias", &lm.getinfo.alias);
    b.label("network", &lm.getinfo.network);
    b.label("version", &lm.getinfo.version);
    gauges.push(b.finish()?);

    Ok(gatherer.gather().to_text())
}

pub fn prometheus_format_down() -> anyhow::Result<String> {
    let mut gatherer = Gatherer::new();
    let reg = gatherer.registry();
    let mut gauges = std::vec::Vec::new();
    metric_gauge(&reg, &0, "up", "", &mut gauges)?;
    Ok(gatherer.gather().to_text())
}
