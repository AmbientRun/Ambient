#!/bin/sh
# use mkcert to generate certs for localhost
mkcert -key-file localhost-key.pem -cert-file localhost.pem localhost 127.0.0.1 ::1

# convert certs from pem to der
openssl x509 -inform pem -outform der -in localhost.pem -out localhost.crt
openssl rsa -inform pem -outform der -in localhost-key.pem -out localhost.key

rm localhost.pem localhost-key.pem
