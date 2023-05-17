#!/bin/sh
rcgen

mv ./certs/cert.der localhost.crt
mv ./certs/key.der localhost.key

rm -r ./certs
