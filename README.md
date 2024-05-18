# Consul KV Sync

This command line tool synchronizes your local files to a Consul key-value store.

It keeps track of changes by storing the hashes of values and files in the consul KV store under prefix **medatada/**.
By doing so it will only upload the content if there is a change in the file.

You can prevent content of a specific key from being overwritten even if changed on the source by adding the key "permitOverride" to false in the metadata json. For example:

For example to prevent the key `config/application/test` you should modify the key `metadata/config/application/test` and change the **permitOverride** to `false`

```json
{
  "hash": "1b0a8af510c5adcb7dcfece3633c3227c181de5f17240c052f667d6289fe9258",
  "permitOverride": false
}
```

This cli will ignore files under .git folder (root and not recursive). You can add more files to ignore using multiple `--ignore` parameters.

## Usage

```text
Usage: consul-kv-sync.exe [OPTIONS]

Options:
 -d, --directory <DIRECTORY>
          Files Directory [default: .]
      --consul-address <CONSUL_ADDRESS>
          Address for consul server agent. If not set or empty will use the default http://localhost:8500 [default: ]
      --consul-token <CONSUL_TOKEN>
          Consul token for authentication [default: ]
  -i, --ignore <IGNORE>...
          Ignore files with that name
  -h, --help
          Print help
  -V, --version
          Print version

```

Example:

```bash
consul-kv-sync -d  /config --consul-address http://127.0.0.1:8500 --consul-token e95b599e-166e-7d80-08ad-aee76e7ddf19
```

or

```bash
export CONSUL_TOKEN=e95b599e-166e-7d80-08ad-aee76e7ddf19
consul-kv-sync -d /config --consul-address http://127.0.0.1:8500
```

or with default consul address (http://localhost:8500)

```bash
export CONSUL_TOKEN=e95b599e-166e-7d80-08ad-aee76e7ddf19
consul-kv-sync -d /config
```

or with env variables

```bash
export CONSUL_TOKEN=e95b599e-166e-7d80-08ad-aee76e7ddf19
export CONSUL_ADDRESS=http://server.consul.svc.cluster.local
consul-kv-sync -d /config
```

The cli parameters has precedence over environment variables.

**Consul token is required.**

## System Requirements

- [Rust 1.78.0](https://www.rust-lang.org/tools/install)
- [Consul](https://www.consul.io/downloads)

## Project Dependencies

The project utilizes the following Rust crates:

- tokio 1.37.0
- serde 1.0.202
- sha2 0.9.9
- base64 0.22.1
- serde_json 1.0.117
- reqwest 0.12.4
- walkdir 2.5.0

Use Rust Package Manager to download and manage these packages.

## Cloning the Repository

To clone the repository, run the following git command:

```bash
git clone git@github.com:kugelbit-com/consul-kv-sync.git
```

## License

[Apache License 2.0](LICENSE)

## Copyright

[Copyright 2024 Kugelbit Inc.](AUTHORS)