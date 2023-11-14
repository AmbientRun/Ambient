var writer: WritableStreamDefaultWriter<any> | null = null;

self.onmessage = function (e) {
    if (e.data instanceof WritableStream) {
        console.warn("Got a stream");
        writer = e.data.getWriter();
        return;
    } else if (writer != null) {
        writer.write(e.data);
    } else {
        console.error("Got data but no stream is available");
    }
};

console.warn("Stream writer worker started");
