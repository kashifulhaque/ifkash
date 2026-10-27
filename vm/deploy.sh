#!/bin/sh
# Copies the VM agent to the VM and (re)starts its systemd services.
# Usage: vm/deploy.sh   (run from anywhere; needs the ifkash SSH key)
set -eu
here=$(cd "$(dirname "$0")" && pwd)
ssh_cmd="ssh -i $HOME/.ssh/id_ed25519_kh"
target=ifkash@vm.ifkash.dev
dest=docs/infra/claude-remote

$ssh_cmd "$target" "mkdir -p ~/$dest"
rsync -az -e "$ssh_cmd" "$here/agent/" "$here/systemd" "$target:$dest/"
$ssh_cmd "$target" "set -e
  cd ~/$dest
  [ -f .env ] || install -m 600 env.example .env
  command -v ttyd >/dev/null || sudo apt-get install -y ttyd
  sudo install -m 644 systemd/*.service /etc/systemd/system/
  sudo systemctl daemon-reload
  sudo systemctl enable --now claude-remote-ttyd claude-remote-agent
  sudo systemctl restart claude-remote-ttyd claude-remote-agent
  sleep 1
  systemctl --no-pager --lines=3 status claude-remote-ttyd claude-remote-agent | grep -E 'Active|vm-agent|ttyd'"
