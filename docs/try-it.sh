#!/usr/bin/env bash


# This script starts a kafka docker container and publishes data coming from a json API to a topic.
#
# Examples
#
# bash docs/try-it.sh
# bash docs/try-it.sh "Nantes" "json" "public-french-addresses-json"
# bash docs/try-it.sh "Narbonne" "jsonSchema" "public-french-addresses-json-schema"
# bash docs/try-it.sh "Niort" "avro" "public-french-addresses-avro"
# bash docs/try-it.sh "Nancy" "text" "public-french-addresses-text"
# bash docs/try-it.sh "Nimes" "malformed" "public-french-addresses-malformed"
# 
# jbang run ./docs/schemas/MyConsumer.java public-french-addresses


set -eo pipefail


# When jbang is not installed,
# it uses the kafka-console-producer to produce records
function fallback_produce {
    local topic="$1"
    local query="$2"
    
    IFS=$'\n'
    result=$(curl -s "https://api-adresse.data.gouv.fr/search/?q=$(echo -n "$query" | sed 's/ /%20/g')&limit=10" | jq -c '.features[]')
    for item in $result; do
        local key
        key=$(uduidgen 2> /dev/null || true)
        if [ -z "$key" ]; then
            key="$RANDOM"
        fi
        echo "${key}##$item" | docker compose exec -T kafka /usr/bin/kafka-console-producer --broker-list localhost:9092 --topic "${topic}" --property parse.key="true" --property key.separator="##" &
        echo "    A new record has been produced."
    done
    wait
}

# Make sure these commands are installed
function check_commands_are_installed {
    for cmd in docker git bash sed jq; do
        if ! command -v $cmd &> /dev/null; then
            echo " ❌ This script requires programs to be installed on your machine. Unfortunately, I was not able to find '$cmd'. Install '$cmd' and try again."
            exit 1
        fi
    done    
}


check_commands_are_installed


if [ "$BASH_SOURCE" = "" ]; then
    if [ ! -d /tmp/yozefu ]; then
        echo " 🪂 Cloning 'git@github.com:MAIF/yozefu.git' to '/tmp/yozefu'"
        git clone git@github.com:MAIF/yozefu.git --depth 1 /tmp/yozefu
    else
        git -C /tmp/yozefu pull
    fi
    bash /tmp/yozefu/docs/try-it.sh
    exit 0
fi

repo=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )/.." &> /dev/null && pwd )
topic="public-french-addresses"
query="kafka"
type="json"

if [ $# -ge 1 ]; then
    query="$1"
fi

if [ $# -ge 2 ]; then
    type="$2"
fi

if [ $# -ge 3 ]; then
    topic="$3"
fi

if [ $# -ge 4 ]; then
    url="$4"
fi


echo " 🐋 Starting kafka"
docker compose -f "${repo}/compose.yml" up kafka schema-registry -d --wait --no-recreate
docker compose -f "${repo}/compose.yml" exec -T kafka \
  /usr/bin/kafka-topics \
  --create --if-not-exists          \
  --bootstrap-server localhost:9092 \
  --partitions 3                    \
  --topic "${topic}"

if jbang --version &> /dev/null; then
    echo " 🤖 jbang run ${repo}/docs/schemas/MyProducer.java --type "$type" --topic "$topic" "$query""
    jbang run ${repo}/docs/schemas/MyProducer.java --type "$type" --topic "$topic" "$query"
else
    echo " ℹ️ About to use the default producer 'kafka-console-producer.sh'. Install jbang to create a kafka producer using the schema registry."
    echo " 🏡 Searching french addresses matching the query '${query}'"
    echo " 📣 About to producing records to topic '${topic}'"
    fallback_produce "$topic" "$query"
fi


# Invite to try the tool
if ! command -v cargo &> /dev/null
then
    if command -v yozf &> /dev/null
    then
        echo " 🎉 Finally, start the tool"
        echo "    yozf -c localhost"
    else
        echo -e "    It looks like you haven't installed \033[1myozefu\033[0m yet:"
        echo "       1. Go to https://github.com/MAIF/yozefu/releases/latest"
        echo "       2. Download the binary that matches your operating system"
        echo "       3. curl -L 'https://github.com/MAIF/yozefu/releases/download/<version>/yozefu-<target>-<version>.tar.gz' | tar xvz"
        echo "       4. mv yozefu-* yozf"
        echo "       5. Run './yozf -c localhost'"
    fi
else
    echo " 🎉 Finally, start the tool"
    echo "    cargo run --manifest-path \"${repo}/Cargo.toml\" -- -c localhost"
    echo "    or"
    echo "    cargo run --manifest-path \"${repo}/Cargo.toml\" -- -c localhost --headless --topics ${topic} 'from begin'"
fi