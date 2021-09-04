FROM rust:latest

RUN apt update -y
RUN apt install -y nano cmake 

WORKDIR /setup
RUN git clone https://github.com/unicorn-engine/unicorn/
WORKDIR /setup/unicorn/
RUN ./make.sh
RUN ./make.sh install

WORKDIR /setup/
RUN git clone https://github.com/keystone-engine/keystone/
RUN mkdir build
WORKDIR /setup/keystone/build
RUN ../make-share.sh
RUN make install

RUN cp /usr/local/lib/libkeystone.so* /usr/lib/

RUN apt-get install -y clang llvm binutils-dev libunwind-dev
WORKDIR /home/
