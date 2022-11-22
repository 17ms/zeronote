#!/usr/bin/env bash

# Reroute packets from localhost:3000 to localhost:443
# Allows HTTPS without superuser privileges

sudo iptables -t nat -I PREROUTING -p tcp --dport 80 -j REDIRECT --to-ports 3000