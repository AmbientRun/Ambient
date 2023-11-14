var writer: WritableStreamDefaultWriter<any> | null = null;

self.onmessage = function (e) {
    if (e.data instanceof WritableStream) {
        console.warn("Got a stream");
        writer = e.data.getWriter();
        return;
    } else if (writer != null) {
        let now = new Date();
        writer.write(e.data).then(() => {
            let elapsed = new Date().getTime() - now.getTime();
            console.warn("Write took " + elapsed + "ms");
        });
    } else {
        console.error("Got data but no stream is available");
    }
};

console.warn("Stream writer worker started");
