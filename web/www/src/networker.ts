var webtransport: WebTransport | null = null;
var datagramsReader: ReadableStreamDefaultReader<Uint8Array> | null = null;
var datagramsWriter: WritableStreamDefaultWriter<Uint8Array> | null = null;
var incomingUnidirectionalStreams: ReadableStreamDefaultReader<ReadableStream> | null = null;
var incomingBidirectionalStreams: ReadableStreamDefaultReader<ReadableStream> | null = null;

self.onmessage = function (e) {
    if (e.data instanceof Array) {
        switch (e.data[0]) {
            case "connect": {
                console.log("Connecting to: ", e.data[1]);
                webtransport = new WebTransport(e.data[1]);
                webtransport.ready.then(() => {
                    console.info("WebTransport ready");
                    if (!webtransport) {
                        self.postMessage(["connect_error", "WebTransport is null"]);
                        return;
                    }
                    datagramsReader = webtransport.datagrams.readable.getReader();
                    datagramsWriter = webtransport.datagrams.writable.getWriter();
                    incomingUnidirectionalStreams = webtransport.incomingUnidirectionalStreams.getReader();
                    incomingBidirectionalStreams = webtransport.incomingBidirectionalStreams.getReader();
                    self.postMessage(["ready"]);
                }, (e) => { 
                    self.postMessage(["connect_error", e.message]);
                });
                break;
            }
            case "poll_datagrams": {
                if (datagramsReader) {
                    datagramsReader.read().then(({ value, done }) => {
                        if (done) {
                            console.info("Datagrams reader done");
                            self.postMessage(["datagram", null]);
                        } else {
                            self.postMessage(["datagram", value]);
                        }
                    }, (e) => {
                        console.error("Error reading datagrams: ", e);
                    });
                } else {
                    console.error("No datagrams reader");
                }
                break;
            }
            case "send_datagram": {
                if (datagramsWriter) {
                    datagramsWriter.write(e.data[1]).then(() => {
                        // TODO: measure latency?
                    }, (e) => {
                        console.error("Error writing datagrams: ", e);
                    });
                } else {
                    console.error("No datagrams writer");
                }
                break;
            }
            case "open_uni": {
                if (webtransport) {
                    webtransport.createUnidirectionalStream().then((stream) => {
                        console.info("Opened unidirectional stream: ", stream);
                        //@ts-ignore
                        postMessage(["opened_uni", stream], [stream]);
                    }, (e) => {
                        console.error("Error opening unidirectional stream: ", e);
                        self.postMessage(["opened_uni", e.message]);
                    });
                } else {
                    console.error("No WebTransport");
                }
                break;
            }
            case "accept_uni": {
                if (incomingUnidirectionalStreams) {
                    incomingUnidirectionalStreams.read().then(({ value, done }) => {
                        if (done) {
                            console.info("No more incoming unidirectional streams");
                            self.postMessage(["accepted_uni", null]);
                        } else {
                            console.info("Accepted unidirectional stream: ", value);
                            //@ts-ignore
                            postMessage(["accepted_uni", value], [value]);
                        }
                    }, (e) => {
                        console.error("Error accepting unidirectional stream: ", e);
                        self.postMessage(["accepted_uni", e.message]);
                    });
                } else {
                    console.error("No WebTransport");
                }
                break;
            }
            case "accept_bi": {
                if (incomingBidirectionalStreams) {
                    incomingBidirectionalStreams.read().then(({ value, done }) => {
                        if (done) {
                            console.info("No more incoming bidirectional streams");
                            self.postMessage(["accepted_bi", null]);
                        } else {
                            console.info("Accepted bidirectional stream: ", value);
                            //@ts-ignore
                            let { writable, readable } = value;
                            //@ts-ignore
                            postMessage(["accepted_bi", writable, readable], [writable, readable]);
                        }
                    }, (e) => {
                        console.error("Error accepting bidirectional stream: ", e);
                        self.postMessage(["accepted_bi", e.message]);
                    });
                } else {
                    console.error("No WebTransport");
                }
                break;
            }
            default: {
                console.error("Unknown request: ", e.data[0]);
            }
        }
    } else {
        console.error("Wrong type of message");
    }
};

console.info("Network worker started");
