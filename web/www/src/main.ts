import "./style.css";

declare global {
  interface Window {
    audio_ctx: AudioContext | null;
    audio_params: AudioProcessorParams;
    audioStart: () => void;
    audioStop: () => void;
    setupAudio: () => void;
  }
}

async function URLFromFiles(files: string[]) {
  const promises = files.map((file) =>
    fetch(file).then((response) => response.text())
  );

  const texts = await Promise.all(promises);
  const text = texts.join("");
  const blob = new Blob([text], { type: "application/javascript" });
  return URL.createObjectURL(blob);
}

import audioProcessorUrl from "./audio_processor.ts?url";

async function setupAudio() {
  let audio_ctx = new AudioContext();
  audio_ctx.suspend();
  window.audio_ctx = audio_ctx;

  console.info(`Importing audio processor from ${audioProcessorUrl}`);
  let module = await URLFromFiles([audioProcessorUrl]);

  if (window.audio_ctx.audioWorklet === undefined) {
    console.error("No AudioWorklet.");
  } else {
    window.audio_ctx.audioWorklet.addModule(module).then(() => {
      let dataSAB = new SharedArrayBuffer(2048 * 4); // 4 is the byte lenth
      let pointerSAB = new SharedArrayBuffer(2 * 4);
      let writePtr = new Uint32Array(pointerSAB, 0, 1);
      let readPtr = new Uint32Array(pointerSAB, 4, 1);

      window.audio_params = {
        dataSAB,
        pointerSAB,
        writePtr,
        readPtr,
      };

      const n = new AudioWorkletNode(audio_ctx, "audio_processor", {
        processorOptions: {
          dataSAB: dataSAB,
          pointerSAB: pointerSAB,
          writePtr: writePtr,
          readPtr: readPtr,
        },
      });
      n.connect(audio_ctx.destination);
    });
  }
}

window.setupAudio = setupAudio;

window.audioStart = () => {
  if (window.audio_ctx) {
    window.audio_ctx.resume();
  }
};

window.audioStop = () => {
  if (window.audio_ctx) {
    window.audio_ctx.suspend();
  }
};

import("ambient_web")
  .catch((e) => console.error("Error importing `ambient`:", e))
  .then((ambient) => {
    if (!ambient) {
      console.error("Ambient is null");
      return;
    }

    ambient.init_ambient(true, true);

    let target = window.document.getElementById("instance-container");

    if (!target) {
      console.error("No target");
      return;
    }

    ambient.start(target, "https://api.ambient.run/servers/ensure-running?deployment_id=17zKiSVwEBHXiC0d8avbro");
    // setupAudio();
  });
