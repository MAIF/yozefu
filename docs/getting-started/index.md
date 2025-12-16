# Getting started


Give Yōzefu a try in your terminal with the following command:

```shell
# It clones this repository, starts a docker kafka node and produces json records
curl -L "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/try-it.sh" | bash
yozf -c localhost
```


## Install


### Prerequisites



::: code-group

```sh [Cargo]
RUSTFLAGS="--cfg tokio_unstable" cargo install --locked yozefu
```

```sh [Git]
git clone git@github.com:MAIF/yozefu.git
cd yozefu
cargo install --path crates/bin
```

```sh [Brew]
brew install yozefu
```

```sh [Yay]
yay -S yozefu
```

```sh [Nix]
nix run github:MAIF/yozefu
```
:::


### Configuration

Once installed, you can configure Yōzefu to connect to your Kafka clusters.


::: tip NOTE

Yōzefu has a default cluster named `localhost` pointing to `localhost:9092`. You can use it to quickly try Yōzefu with a local Kafka instance.

:::

```shell
# Path of the configuration file
yozf config get path

# Open the configuration file with your favorite editor
yozf configure --editor 'vim'
```

And then add your cluster configuration. Here is an example for a Aiven cluster:
```json{8-18}
{
  "default_url_template": "...",
  "initial_query": "from end - 10",
  "clusters": {
    "localhost": {
        ...
    },
    "production": {
      "url_template": "https://console.aiven.io/<acme>/topics/{topic}/messages?offset={offset}&partition={partition}&format=json",
      "kafka": {
        "bootstrap.servers": "kafka.aivencloud.com:24624",
        "security.protocol": "sasl_ssl",
        "sasl.mechanism": "SCRAM-SHA-256",
        "sasl.username": "user",
        "sasl.password": "password",
        "ssl.ca.location": "~/path/to/ca.pem"
      }
    }
  }
}
```

## Run it

Once configured, you can start using Yōzefu to query your Kafka topics.

```shell
# Connect to the 'production' cluster
yozf --cluster production
```