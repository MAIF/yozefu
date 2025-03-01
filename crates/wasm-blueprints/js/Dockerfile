FROM ubuntu:latest
RUN rm /bin/sh && ln -s /bin/bash /bin/sh
RUN apt-get update
RUN apt-get install -y sudo build-essential curl
RUN curl -O https://raw.githubusercontent.com/extism/js-pdk/main/install.sh
RUN bash install.sh
RUN curl -fsSL https://deb.nodesource.com/setup_22.x -o nodesource_setup.sh
RUN bash nodesource_setup.sh
RUN apt-get install -y nodejs

WORKDIR /tmp/build
COPY . .
RUN make module.wasm