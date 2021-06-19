#!/usr/bin/env bash
set -e

nice cargo build --release

# workdir
sudo mkdir -p /var/local/lib/luicellas-stalker/
sudo chown -R pi:pi /var/local/lib/luicellas-stalker/

# systemd
sudo mkdir -p /usr/local/lib/systemd/system/
sudo cp -uv ./*.service ./*.timer /usr/local/lib/systemd/system/
sudo systemctl daemon-reload

# stop, replace and start new version
sudo systemctl stop luicellas-stalker.service luicellas-stalker.timer
sudo cp -v target/release/luicellas-stalker /usr/local/bin
sudo systemctl enable --now luicellas-stalker.timer
