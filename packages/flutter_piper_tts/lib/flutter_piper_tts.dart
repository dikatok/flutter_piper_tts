import "dart:io";

import "package:dart_piper_tts/dart_piper_tts.dart" as piper_dart;
import "package:flutter/services.dart";
import "package:path/path.dart";
import "package:path_provider/path_provider.dart";

class PiperTTS {
  final piper_dart.PiperTTS _tts;

  PiperTTS._(this._tts);

  static Future<void> init() async {
    final directory = await getApplicationSupportDirectory();
    final phonemizerPath = join(directory.path, 'phonemizer.onnx');
    final exists = await File(phonemizerPath).exists();
    if (!exists) {
      final data = await rootBundle.load(
        'packages/flutter_piper_tts/assets/phonemizer/model.onnx',
      );
      List<int> bytes = data.buffer.asUint8List(
        data.offsetInBytes,
        data.lengthInBytes,
      );
      await File(phonemizerPath).writeAsBytes(bytes, flush: true);
    }
    piper_dart.PiperTTS.init((phonemizerModelPath: phonemizerPath));
  }

  static Future<PiperTTS> create({
    required String modelPath,
    required String configPath,
  }) async {
    return PiperTTS._(
      piper_dart.PiperTTS.create((
        configPath: configPath,
        modelPath: modelPath,
      )),
    );
  }

  Future<void> speak(String text, {bool waitForCompletion = true}) =>
      _tts.speak(text, waitForCompletion: waitForCompletion);

  Future<void> speakFromPhonemes({
    required String phonemes,
    bool waitForCompletion = true,
  }) => _tts.speakFromPhonemes(
    phonemes: phonemes,
    waitForCompletion: waitForCompletion,
  );

  void pause() => _tts.pause();

  void resume() => _tts.resume();

  void stop() => _tts.stop();

  void dispose() => _tts.dispose();
}
