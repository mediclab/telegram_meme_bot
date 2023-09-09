FROM rust:latest

COPY --from=medic84/opencv:4.8.0 /usr/local/lib/libopencv_* /usr/local/lib/
COPY --from=medic84/opencv:4.8.0 /usr/local/bin/opencv_* /usr/local/bin/
COPY --from=medic84/opencv:4.8.0 /usr/local/include/opencv4 /usr/local/include/opencv4/
COPY --from=medic84/opencv:4.8.0 /usr/local/lib/cmake/opencv4 /usr/local/lib/cmake/opencv4/
COPY --from=medic84/opencv:4.8.0 /usr/local/share/opencv4 /usr/local/share/opencv4/

RUN DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get install -y cmake build-essential clang \
    && cargo install diesel_cli --no-default-features --features "postgres" \
    && apt-get clean

WORKDIR /app

ENV LD_LIBRARY_PATH=/usr/local/lib