**Work in progress. Current choice of metrics is likely useless.**

```
Prometheus exporter for monitoring c-lightning node

USAGE:
    prometheus-clightning-exporter [FLAGS] [OPTIONS] --rpc-socket <PATH>

FLAGS:
    -v, --verbose              Enable debug log messages
    -T, --no-log-timestamps    Do not prepend timestamps to log output
    -h, --help                 Prints help information
    -V, --version              Prints version information

OPTIONS:
    -s, --rpc-socket <PATH>       Path to lightning-rpc socket
    -l, --listen <ADDR:PORT>      Address:port on which to expose metrics [default: 127.0.0.1:9393]
    -r, --rate-limit <SECONDS>    Minimal period between lightningd scrapes [default: 1]
    -t, --timeout <SECONDS>       Timeout for socket operations [default: 5]
```

TODO
----

* actual useful metrics, correct names/labels
* tests
* allocate port number 9393
* provide Grafana dashboard
* write README

Links
-----

C-lightning:
* https://github.com/ElementsProject/lightning

Prometheus:
* [https://prometheus.io/docs/instrumenting/writing_exporters/](https://prometheus.io/docs/instrumenting/writing_exporters/)
* List: https://github.com/prometheus/docs/blob/master/content/docs/instrumenting/exporters.md
* Port: https://github.com/prometheus/prometheus/wiki/Default-port-allocations

Bitcoind exporters:
* https://github.com/jvstein/bitcoin-prometheus-exporter
* https://github.com/LePetitBloc/bitcoind-exporter

Tokio:
* https://docs.rs/tokio/0.2/tokio/
* https://tokio.rs/

Hyper:
* https://hyper.rs/
