class Processor extends AudioWorkletProcessor {
  static get parameterDescriptors() {
    return [];
  }
  constructor(options) {
    super(options);
    const { dataSAB, pointerSAB, writePtr, readPtr } = options.processorOptions;
    // this.dataSAB = dataSAB;
    this.dataSAB = new Float32Array(dataSAB);
    // this.pointerSAB = new Uint32Array(pointerSAB);
    this.writePtr = writePtr;
    this.readPtr = readPtr;
  }

  process(_input, outputs, _parameters) {
    for (let i = 0; i < 128; i++) {
        const readIndex = Atomics.load(this.readPtr, 0);
        const writeIndex = Atomics.load(this.writePtr, 0);
        outputs[0][0][i] = this.dataSAB [readIndex];
        Atomics.store(this.readPtr, 0, (readIndex + 1) % 2048);
    }
    return true;
  }
}

registerProcessor("processor", Processor);