# Ambient Audio

## Terminology

- Stream: handles the backend audio playing device
- Mixer: contained in a stream. Takes multiple playing sources, and adds them
  together into the single device audio output stream.

- Source: Represents a node in the audio graph. Can either be an input node,
  like a `SineWave` or `Track`, a transform node such as `Gain` or `Repeat`, or
  an output node like the mixer.

  As sources can have a different sample rate than the native audio stream
  sample rate, the `SampleConversion` source is needed between the input source
  and the output source/mixer.

- Sound: An instance of a playing graph of sources, can be used to control the
  playback or modify the graph.

- Sample: A floating point PCM value of the average waveform amplitude at a given slice
  of (1/sample_rate) time. Channels are interleaved in source

- Frame: A _planar_ collection of samples from the same point in time of all
  channels in the source.

  Note that there may be up 8 channels of audio for any given source.
