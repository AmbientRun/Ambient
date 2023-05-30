# Certificates

By default, _ambient_ bundles a self-signed certificate that is used by the server and trusted by the client.

To use your own certificate specify `--cert`, `--key`, for the server, and `--ca` for the client if the certificate authority which signed the certificate is not in the system roots. If specified, the bundled certificates will _not_ be used as a fallback.

```sh
ambient serve --cert ./localhost.crt --key ./localhost.key

```

```sh
ambient join 127.0.0.1:9000
```

**Note**: `--ca path_to_ca` must be specified if the used certificate is not in the system roots
