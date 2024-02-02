FROM rust:bullseye

COPY --from=medic84/opencv:4.8.0 /usr/local/lib/libopencv_* /usr/local/lib/
COPY --from=medic84/opencv:4.8.0 /usr/local/bin/opencv_* /usr/local/bin/
COPY --from=medic84/opencv:4.8.0 /usr/local/include/opencv4 /usr/local/include/opencv4/
COPY --from=medic84/opencv:4.8.0 /usr/local/lib/cmake/opencv4 /usr/local/lib/cmake/opencv4/
COPY --from=medic84/opencv:4.8.0 /usr/local/share/opencv4 /usr/local/share/opencv4/

RUN DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get install -y cmake build-essential clang pkg-config \
      libavcodec-dev libavformat-dev libswscale-dev libtbb-dev libjpeg-dev libpng-dev libtiff-dev libpq-dev llvm-dev \
      libgtk2.0-dev libdc1394-22-dev libssl-dev \
    && cargo install diesel_cli --no-default-features --features "postgres" --target-dir /usr/local/bin \
    && apt-get clean

WORKDIR /app

ENV LD_LIBRARY_PATH=/usr/local/lib