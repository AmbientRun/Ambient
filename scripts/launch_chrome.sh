#!/bin/bash

set -e

SPKI=`openssl x509 -inform der -in localhost.crt -pubkey -noout | openssl pkey -pubin -outform der | openssl dgst -sha256 -binary | openssl enc -base64`

echo "Got cert key $SPKI"

echo "Opening google chrome"

case `uname` in
    (*Linux*)  google-chrome --origin-to-force-quic-on=127.0.0.1:4433 --ignore-certificate-errors-spki-list=$SPKI --enable-logging --v=1 &;;
    (*Darwin*)  open -a "Google Chrome" --args --origin-to-force-quic-on=127.0.0.1:4433 --ignore-certificate-errors-spki-list=$SPKI --enable-logging --v=1 ;;
esac

