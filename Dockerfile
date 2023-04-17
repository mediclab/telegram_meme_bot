FROM rust:latest

RUN DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get install -y cmake build-essential clang git \
      pkg-config libavcodec-dev libavformat-dev libswscale-dev libclang-dev \
      libtbb2 libtbb-dev libjpeg-dev libpng-dev libtiff-dev libdc1394-22-dev llvm-dev \
    && cargo install diesel_cli --no-default-features --features "postgres" \
    && apt-get clean

WORKDIR /opt

RUN git clone --depth 1 --branch '4.7.0' https://github.com/opencv/opencv.git && \
    cd ./opencv && mkdir build && cd build && \
    cmake \
      -D CMAKE_BUILD_TYPE=RELEASE \
      -D WITH_V4L=ON \
      -D WITH_QT=OFF \
      -D WITH_OPENGL=ON \
      -D WITH_GSTREAMER=OFF \
      -D OPENCV_GENERATE_PKGCONFIG=OFF \
      -D OPENCV_ENABLE_NONFREE=OFF \
      -D INSTALL_PYTHON_EXAMPLES=OFF \
      -D INSTALL_C_EXAMPLES=OFF \
      -D CMAKE_INSTALL_PREFIX=/usr/local \
      -D BUILD_EXAMPLES=OFF \
    .. && \
    make -j"$(nproc)" && \
    make install && \
    rm -rf /opt/opencv

WORKDIR /app

ENV LD_LIBRARY_PATH=/usr/local/lib