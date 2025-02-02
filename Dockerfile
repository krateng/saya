FROM rust:1.83-alpine3.21 AS builder

WORKDIR /source

# system requirements
RUN apk add --no-cache \
    build-base \
    curl \
    libressl-dev \
    pkgconfig \
    git \
    bash

ENV CARGO_TARGET_DIR=/target

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src
RUN cargo build --release


FROM lsiobase/ubuntu:jammy-version-c295ed29 AS main

# first pretty much do what cm2network/steamcmd does
ARG PUID=1000

# HOME is also used by steam for logs and stuff
ENV USER=steam
ENV HOME=/home/$USER
ENV STEAMCMDDIR=$HOME/steamcmd
ENV PALSERVERDIR=$HOME/palworld

ENV INTERNAL_SERVER_PORT=8215
ENV COMMUNITY_SERVER=0

RUN apt update && apt install -y --no-install-recommends --no-install-suggests \
  lib32stdc++6 \
  lib32gcc-s1 \
  ca-certificates \
  nano \
  curl \
  locales \
  xdg-user-dirs \
  python3 \
  python3-pip \
  socat
RUN pip install toml
RUN sed -i -e 's/# en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen \
  && dpkg-reconfigure --frontend=noninteractive locales

RUN useradd -u $PUID -m $USER
RUN chown $USER $HOME

USER $USER
RUN \
  mkdir -p $STEAMCMDDIR && \
  mkdir -p $HOME/.steam/sdk32 && \
  mkdir -p $HOME/.steam/sdk64
RUN curl -fsSL 'https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz' | tar xvzf - -C ${STEAMCMDDIR}
RUN $STEAMCMDDIR/steamcmd.sh +quit
RUN \
  ln -s $STEAMCMDDIR/linux32/steamclient.so $STEAMCMDDIR/steamservice.so  && \
  ln -s $STEAMCMDDIR/linux32/steamclient.so $HOME/.steam/sdk32/steamclient.so && \
  ln -s $STEAMCMDDIR/linux32/steamcmd $STEAMCMDDIR/linux32/steam && \
  ln -s $STEAMCMDDIR/linux64/steamclient.so $HOME/.steam/sdk64/steamclient.so && \
  ln -s $STEAMCMDDIR/linux64/steamcmd $STEAMCMDDIR/linux64/steam && \
  ln -s $STEAMCMDDIR/steamcmd.sh $STEAMCMDDIR/steam.sh
 
USER root
RUN ln -s $STEAMCMDDIR/linux64/steamclient.so /usr/lib/x86_64-linux-gnu/steamclient.so

USER $USER
WORKDIR $STEAMCMDDIR
RUN ./steamcmd.sh +quit


# dev
ADD --chown=$USER ./serverfiles $PALSERVERDIR
# prod
#RUN mkdir $PALSERVERDIR && chown $USER $PALSERVERDIR && chmod -R 755 $PALSERVERDIR
#RUN ./steamcmd.sh +force_install_dir $PALSERVERDIR +login anonymous +app_update 2394010 validate +quit

WORKDIR $PALSERVERDIR

# this replaces the PalServer.sh pre work
RUN cp $PALSERVERDIR/linux64/steamclient.so $PALSERVERDIR/Pal/Binaries/Linux/steamclient.so
RUN chmod +x $PALSERVERDIR/Pal/Binaries/Linux/PalServer-Linux-Shipping

RUN mkdir -p $PALSERVERDIR/Pal/Saved/Config/LinuxServer && chown $USER $PALSERVERDIR/Pal/Saved/Config/LinuxServer

ADD --chown=$USER --chmod=755 ./scripts/init.sh $PALSERVERDIR/init.sh
ADD --chown=$USER --chmod=755 ./scripts/generate_config.py $PALSERVERDIR/generate_config.py

COPY --chown=$USER --from=builder --chmod=755 /target/release/saya $PALSERVERDIR/saya

# server
EXPOSE 8211
# RCON
EXPOSE 25575

# ignore s6 overlay stuff for now, do it manual

ENTRYPOINT ["./saya"]
