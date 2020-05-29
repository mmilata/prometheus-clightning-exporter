use std::vec::Vec;

use prometrics::metrics;

use crate::producer::LightningMetrics;

struct Gatherer {
    gauges: Vec<metrics::Gauge>,
    gatherer: prometrics::Gatherer,
}

impl Gatherer {
    fn new() -> Self {
        Gatherer {
            gauges: Vec::new(),
            gatherer: prometrics::Gatherer::new(),
        }
    }

    fn gauge_label(
        &mut self,
        name: &str,
        val: u64,
        help: &str,
        labels: &Vec<(&str, &str)>,
    ) -> anyhow::Result<()> {
        let mut m = metrics::GaugeBuilder::new(name);
        m.registry(self.gatherer.registry());
        m.namespace("lightning");
        if !help.is_empty() {
            m.help(help);
        }
        for l in labels {
            m.label(l.0, l.1);
        }
        let m = m.finish()?;
        m.set(val as f64);
        self.gauges.push(m);
        Ok(())
    }

    fn gauge(&mut self, name: &str, val: u64, help: &str) -> anyhow::Result<()> {
        self.gauge_label(name, val, help, &vec![])
    }

    fn to_text(mut self) -> String {
        self.gatherer.gather().to_text()
    }
}

fn collect_channel(
    g: &mut Gatherer,
    peer: &str,
    chan: &clightningrpc::responses::Channel,
) -> anyhow::Result<()> {
    let pending = "pending".to_string();
    let scid = chan.short_channel_id.as_ref().unwrap_or(&pending);
    let labels: Vec<(&str, &str)> = vec![("id", peer), ("scid", scid)];
    g.gauge_label(
        "channel_balance",
        chan.to_us_msat.0,
        "How many funds are at our disposal?",
        &labels,
    )?;
    g.gauge_label(
        "channel_spendable",
        chan.spendable_msat.0,
        "How much can we currently send over this channel?",
        &labels,
    )?;
    g.gauge_label(
        "channel_capacity",
        chan.total_msat.0,
        "How many funds are in this channel in total?",
        &labels,
    )?;
    g.gauge_label(
        "channel_htlcs",
        chan.htlcs.len() as u64,
        "How many HTLCs are currently active on this channel?",
        &labels,
    )?;

    g.gauge_label(
        "channel_in_payments_offered",
        chan.in_payments_offered,
        "How many incoming payments did we try to forward?",
        &labels,
    )?;
    g.gauge_label(
        "channel_in_payments_fulfilled",
        chan.in_payments_fulfilled,
        "How many incoming payments did we succeed to forward?",
        &labels,
    )?;
    g.gauge_label(
        "channel_in_msatoshi_offered",
        chan.in_offered_msat.0,
        "How many incoming msats did we try to forward?",
        &labels,
    )?;
    g.gauge_label(
        "channel_in_msatoshi_fulfilled",
        chan.in_fulfilled_msat.0,
        "How many incoming msats did we succeed to forward?",
        &labels,
    )?;

    g.gauge_label(
        "channel_out_payments_offered",
        chan.out_payments_offered,
        "How many outgoing payments did we try to forward?",
        &labels,
    )?;
    g.gauge_label(
        "channel_out_payments_fulfilled",
        chan.out_payments_fulfilled,
        "How many outgoing payments did we succeed to forward?",
        &labels,
    )?;
    g.gauge_label(
        "channel_out_msatoshi_offered",
        chan.out_offered_msat.0,
        "How many outgoing msats did we try to forward?",
        &labels,
    )?;
    g.gauge_label(
        "channel_out_msatoshi_fulfilled",
        chan.out_fulfilled_msat.0,
        "How many outgoing msats did we succeed to forward?",
        &labels,
    )?;

    Ok(())
}

pub fn prometheus_format(lm: LightningMetrics) -> anyhow::Result<String> {
    let mut g = Gatherer::new();
    g.gauge("up", 1, "Is the node running?")?;


    g.gauge_label(
        "node_info",
        1,
        "Static node information",
        &vec![
            ("id", &lm.getinfo.id),
            ("alias", &lm.getinfo.alias),
            ("network", &lm.getinfo.network),
            ("version", &lm.getinfo.version),
            ("color", &lm.getinfo.color),
        ],
    )?;
    g.gauge(
        "node_blockheight",
        lm.getinfo.blockheight,
        "Current Bitcoin blockheight on this node",
    )?;
    g.gauge(
        "fees_collected_msat",
        lm.getinfo.fees_collected_msat.0 as u64,
        "How much have we been paid to route payments?",
    )?;

    let output_funds = lm
        .listfunds
        .outputs
        .into_iter()
        .map(|chan| chan.amount_msat.0)
        .sum::<u64>()
        / 1000;
    g.gauge(
        "funds_output",
        output_funds,
        "On-chain satoshis at our disposal",
    )?;

    let channel_funds = lm
        .listfunds
        .channels
        .into_iter()
        .map(|chan| chan.our_amount_msat.0)
        .sum::<u64>()
        / 1000;
    g.gauge("funds_channel", channel_funds, "Satoshis in channels")?;

    g.gauge(
        "funds_total",
        channel_funds + output_funds,
        "Total satoshis we own on this node",
    )?;

    for peer in lm.listpeers {
        g.gauge_label(
            "peer_channels",
            peer.channels.len() as u64,
            "The number of channels with the peer",
            &vec![("id", &peer.id)],
        )?;

        g.gauge_label(
            "peer_connected",
            peer.connected as u64,
            "Is the peer currently connected?",
            &vec![("id", &peer.id)],
        )?;

        for chan in peer.channels {
            collect_channel(&mut g, &peer.id, &chan)?;
        }
    }

    Ok(g.to_text())
}

pub fn prometheus_format_down() -> anyhow::Result<String> {
    let mut g = Gatherer::new();
    g.gauge("up", 0, "Is the node running?")?;
    Ok(g.to_text())
}
