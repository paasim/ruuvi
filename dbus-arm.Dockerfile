ARG CROSS_BASE_IMAGE
FROM $CROSS_BASE_IMAGE

RUN dpkg --add-architecture arm64
RUN apt-get update && apt-get -y --no-install-recommends install pkg-config libdbus-1-dev:arm64
