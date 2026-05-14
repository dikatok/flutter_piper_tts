import 'dart:io';

import 'package:dart_piper_tts/dart_piper_tts.dart';

void main(List<String> arguments) {
  PiperTTS.init((
    phonemizerModelPath:
        "../../flutter_piper_tts/assets/phonemizer/g2p-mbyt5-12l-ipa-childes-espeak-onnx-quantized.onnx",
  ));
  final tts = PiperTTS.create((
    configPath: "./en_US-hfc_female-medium.onnx.json",
    modelPath: "./en_US-hfc_female-medium.onnx",
  ));
  tts.speak("Hello world!");
  sleep(Duration(seconds: 2));
}
