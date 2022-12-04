## nsq

async DNS client that party implements [RFC 1035](https://www.ietf.org/rfc/rfc1035.txt)

## How to install

```
cargo install --path .
```

## How to use

- You can pass a host to be resolved, for instance `nsq www.crates.io`:

```
❯ nsq www.crates.io
Server: "192.168.15.1:53"
Answers:
QueryAnswer { host: "www.crates.io", address: "2600:9000:20fa:5c00:2:7350:16c0:93a1", query_type: AAAA, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "2600:9000:20fa:7600:2:7350:16c0:93a1", query_type: AAAA, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "2600:9000:20fa:2000:2:7350:16c0:93a1", query_type: AAAA, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "2600:9000:20fa:c600:2:7350:16c0:93a1", query_type: AAAA, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "2600:9000:20fa:5000:2:7350:16c0:93a1", query_type: AAAA, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "2600:9000:20fa:d000:2:7350:16c0:93a1", query_type: AAAA, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "2600:9000:20fa:2e00:2:7350:16c0:93a1", query_type: AAAA, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "2600:9000:20fa:a00:2:7350:16c0:93a1", query_type: AAAA, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "52.85.213.55", query_type: A, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "52.85.213.92", query_type: A, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "52.85.213.8", query_type: A, class_type: IN }
QueryAnswer { host: "www.crates.io", address: "52.85.213.35", query_type: A, class_type: IN }
```

## Getting help

```
❯ nsq -h
Usage: nsq [OPTIONS] [HOSTS]...

Arguments:
  [HOSTS]...  Hostname to resolve

Options:
  -s, --server <SERVER>  [default: ]
  -h, --help             Print help information
  -V, --version          Print version information
```
