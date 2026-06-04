import "package:dart_piper_tts/dart_piper_tts.dart" as piper_dart;
import "package:flutter/foundation.dart";

export "package:dart_piper_tts/dart_piper_tts.dart" show PhonemizerStrategy;

class PiperTTS {
  final piper_dart.PiperTTS _tts;

  PiperTTS._(this._tts);

  static Future<void> init() async {
    piper_dart.PiperTTS.init((kDebugMode: kDebugMode));
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

  Future<void> speak(
    String text, {
    bool waitForCompletion = true,
    piper_dart.PhonemizerStrategy phonemizerStrategy =
        piper_dart.PhonemizerStrategy.neuralOnly,
  }) => _tts.speak(
    text,
    waitForCompletion: waitForCompletion,
    strategy: phonemizerStrategy,
  );

  Future<void> speakFromPhonemes({
    required String phonemes,
    bool waitForCompletion = true,
    piper_dart.PhonemizerStrategy phonemizerStrategy =
        piper_dart.PhonemizerStrategy.neuralOnly,
  }) => _tts.speakFromPhonemes(
    phonemes: phonemes,
    waitForCompletion: waitForCompletion,
    strategy: phonemizerStrategy,
  );

  void pause() => _tts.pause();

  void resume() => _tts.resume();

  void stop() => _tts.stop();

  void dispose() => _tts.dispose();
}
