# flutter_piper_tts

TTS in Dart (or Flutter) using Piper TTS models + audio player in one package.

All onnx models are run via https://crates.io/crates/ort.

Built-in phonemizer is done via neural g2p ipa phonemizer model obtained from https://huggingface.co/OpenVoiceOS/g2p-mbyt5-12l-ipa-childes-espeak-onnx. This is done to avoid dealing with espeak GPL3 license and keep this package's MIT license. You can also provide your own ipa phonemes via `speakFromPhonemes` to bypass phonemes generation.

Piper TTS models can be downloaded from https://huggingface.co/rhasspy/piper-voices/tree/main.

Audio playback is done using https://crates.io/crates/tinyaudio.

## Usage

`Disclaimer: examples below are using the Flutter package`

- Download Piper TTS model of your choice, make sure to also prepare the `*.onnx.json` file
- Make them available to the device file system, eg. copying from your asset bundle to device application support directory
```dart
  final directory = await getApplicationSupportDirectory();
  final modelPath = join(directory.path, 'en_US-hfc_female-medium.onnx');
  final configPath = join(directory.path, 'en_US-hfc_female-medium.onnx.json');

  final exists = await File(modelPath).exists();

  if (!exists) {
    final modelData = await rootBundle.load(
      'assets/en_US-hfc_female-medium.onnx',
    );
    List<int> bytes = modelData.buffer.asUint8List(
      modelData.offsetInBytes,
      modelData.lengthInBytes,
    );

    await File(modelPath).writeAsBytes(bytes, flush: true);

    final configData = await rootBundle.load(
      'assets/en_US-hfc_female-medium.onnx.json',
    );
    bytes = configData.buffer.asUint8List(
      configData.offsetInBytes,
      configData.lengthInBytes,
    );
    await File(configPath).writeAsBytes(bytes, flush: true);
  }
```
- Initialize the package
```dart
  await PiperTTS.init();
  final tts = await PiperTTS.create(modelPath, configPath);
```
- Call `speak`
```dart
  await tts.speak("Hello world");
  tts.pause();
  tts.resume();
  await tts.speak("Bye", waitForCompletion: false)
  tts.stop();
```

## Supported platforms
- Android
- iOS
- MacOS
- Windows and Linux (not tested)

## TODO
- Better support for subsentence separator like comma
- Adjust speed (with change in pitch/not)