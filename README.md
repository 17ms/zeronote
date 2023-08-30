# RESTful API utilizing PKCE-enhanced OAuth2 via AWS Cognito

<p align="left">
<a href="https://github.com/17ms/zeronote/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/17ms/zeronote/ci.yml?branch=master"></a>
<a href="https://github.com/17ms/zeronote/tags"><img src="https://img.shields.io/github/v/tag/17ms/zeronote"></a>
<a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/github/license/17ms/zeronote"></a>
</p>

## Authentication & authorization flow

![A visualization of the OAuth2 authentication & authorization flow](./.github/docs/oauth-flow.svg)

## Setup

The repository contains a shellscript `setup_dev.sh` that can be used to create a self-signed certificate and possibly setup port forwarding with `iptables`:

```shell
chmod +x setup_dev.sh
./setup_dev.sh -c # create cert
./setup_dev.sh -r # forward :3000 -> :443
```

After creating `cert.pem` and `key.pem`:

```shell
cargo build --release
./target/release/zeronote
```

The API will be listening to `https://localhost` by default. A separate Dockerfile will be added in the future.
