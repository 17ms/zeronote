#!/usr/bin/env bash

# Create self-signed SSL cert for development and automated tests
setup_ssl_cert () {
    openssl req \
        -newkey rsa:2048 \
        -nodes \
        -keyout key.pem \
        -x509 \
        -days 365 \
        -out cert.pem \
        -addext "subjectAltName = DNS:localhost" \
        -subj "/C=US/ST=Oregon/L=Portland/O=Company Name/OU=Org/CN=www.example.com"
}

# Port forwarding from :3000 to :443 allows HTTPS without sudo privileges (generally only for linux)
setup_forwarding () {
    sudo iptables -t nat -I PREROUTING -p tcp --dport 80 -j REDIRECT --to-ports 3000
}

while getopts cr FLAG
do
    case "$FLAG" in
        c)
            echo "Creating SSL certificate to cert.pem (& key to key.pem)"
            setup_ssl_cert
            ;;
        r)
            echo "Setting up port forwarding from 3000 to 443 (for localhost)"
            setup_forwarding
            ;;
        *)
            echo "Invalid flags, aborting..."
            ;;
    esac
done
