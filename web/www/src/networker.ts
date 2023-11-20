var webtransport: WebTransport | null = null;
var datagramsReader: ReadableStreamDefaultReader<Uint8Array> | null = null;
var datagramsWriter: WritableStreamDefaultWriter<Uint8Array> | null = null;
var incomingUnidirectionalStreams: ReadableStreamDefaultReader<ReadableStream> | null = null;
var incomingBidirectionalStreams: ReadableStreamDefaultReader<ReadableStream> | null = null;
var writeStreams: { [id: number]: WritableStreamDefaultWriter<Uint8Array> } = {};
var readStreams: { [id: number]: ReadableStreamDefaultReader<Uint8Array> } = {};
var nextStreamId = 0;

var pollingDatagrams = false;
var pollingStreams: { [id: number]: boolean } = {};

self.onmessage = function (e) {
    if (e.data instanceof Array) {
        switch (e.data[0]) {
            case "ping": {
                console.log("App->Wrkr delay: ", Date.now() - e.data[1]);
                self.postMessage(["pong", e.data[1]]);
                break;
            }
            case "pong": {
                console.log("Wrkr-App ping latency: ", Date.now() - e.data[1]);
                break;
            }
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

                    const pinger = () => {
                        self.postMessage(["ping", Date.now()]);
                        setTimeout(pinger, 1000);
                    };
                    pinger();
                }, (e) => { 
                    self.postMessage(["connect_error", e.message]);
                });
                break;
            }
            case "poll_datagrams": {
                if (pollingDatagrams) {
                    return;
                }
                pollingDatagrams = true;
                if (datagramsReader) {
                    const reader = datagramsReader;
                    const poll = () => {
                        reader.read().then(({ value, done }) => {
                            if (done) {
                                console.info("Datagrams reader done");
                                self.postMessage(["datagram", null]);
                            } else {
                                self.postMessage(["datagram", value, Date.now()]);
                                poll();
                            }
                        }, (e) => {
                            console.error("Error reading datagrams: ", e);
                        });
                    };
                    poll();
                } else {
                    console.error("No datagrams reader");
                }
                break;
            }
            case "send_datagram": {
                if (datagramsWriter) {
                    datagramsWriter.write(e.data[1]).then(() => {
                        // if (e.data[2]) {
                        //     console.log("Sent datagram delay: ", Date.now() - e.data[2]);
                        // }
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
                if (pollingStreams[streamId]) {
                    return;
                }
                pollingStreams[streamId] = true;
                if (readStreams[streamId]) {
                    const reader = readStreams[streamId];
                    const poll = () => {
                        reader.read().then(({ value, done }) => {
                            if (done) {
                                console.info("No more stream data");
                                self.postMessage(["received_stream_data", streamId, null]);
                            } else {
                                self.postMessage(["received_stream_data", streamId, value]);
                                poll();
                            }
                        }, (e) => {
                            console.error("Error reading stream data: ", e);
                        });
                    };
                    poll();
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
