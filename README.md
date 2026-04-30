# flutter_piper_tts

TTS in Dart using Piper TTS models + audio player in one package.

## Usage

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
  final tts = PiperTTS.create(modelPath, configPath);
```
- Make it speak, wrap it in `compute` if used in Flutter context
```dart
  compute(tts.speak, "Hello World!");
```

## Supported platforms
- Android
- iOS
- MacOS
- Windows and Linux (not tested)
- Web (will see in the future, whether to use opfs or load the model from uri/url)

## TODO
- Revisit phonemization
- Allow custom phoneme mapping
- Adjust speed (with change in pitch/not)