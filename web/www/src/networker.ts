var webtransport: WebTransport | null = null;
var datagramsReader: ReadableStreamDefaultReader<Uint8Array> | null = null;
var datagramsWriter: WritableStreamDefaultWriter<Uint8Array> | null = null;
var incomingUnidirectionalStreams: ReadableStreamDefaultReader<ReadableStream> | null = null;
var incomingBidirectionalStreams: ReadableStreamDefaultReader<ReadableStream> | null = null;
var writeStreams: { [id: number]: WritableStreamDefaultWriter<Uint8Array> } = {};
var readStreams: { [id: number]: ReadableStreamDefaultReader<Uint8Array> } = {};
var nextStreamId = 0;

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
                        const streamId = nextStreamId++;
                        console.info("Opened unidirectional stream: ", streamId);
                        writeStreams[streamId] = stream.getWriter();
                        postMessage(["opened_uni", streamId]);
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
                            const streamId = nextStreamId++;
                            console.info("Accepted unidirectional stream: ", streamId);
                            readStreams[streamId] = value.getReader();
                            postMessage(["accepted_uni", streamId]);
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
                            const streamId = nextStreamId++;
                            console.info("Accepted bidirectional stream: ", streamId);
                            //@ts-ignore
                            let { writable, readable } = value;
                            writeStreams[streamId] = writable.getWriter();
                            readStreams[streamId] = readable.getReader();
                            postMessage(["accepted_bi", streamId]);
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
            case "send_stream_data": {
                const streamId = e.data[1];
                const data = e.data[2];
                if (writeStreams[streamId]) {
                    writeStreams[streamId].write(data).then(() => {
                    }, (e) => {
                        console.error("Error writing stream data: ", e);
                    });
                } else {
                    console.error("No write stream for id: ", streamId);
                }
                break;
            }
            case "poll_stream": {
                const streamId = e.data[1];
                if (readStreams[streamId]) {
                    readStreams[streamId].read().then(({ value, done }) => {
                        if (done) {
                            console.info("No more stream data");
                            self.postMessage(["received_stream_data", streamId, null]);
                        } else {
                            self.postMessage(["received_stream_data", streamId, value]);
                        }
                    }, (e) => {
                        console.error("Error reading stream data: ", e);
                    });
                } else {
                    console.error("No read stream for id: ", streamId);
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
