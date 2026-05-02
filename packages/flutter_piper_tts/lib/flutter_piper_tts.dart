import "package:dart_piper_tts/dart_piper_tts.dart" as piper_dart;
import "package:flutter/foundation.dart";
import "package:path_provider/path_provider.dart";

class PiperTTS {
  final piper_dart.PiperTTS _tts;

  PiperTTS._(this._tts);

  static Future<void> init({String? dataDir}) async => compute(
    piper_dart.PiperTTS.init,
    (dataDir: dataDir ?? (await getApplicationSupportDirectory()).path),
  );

  static Future<PiperTTS> create({
    required String modelPath,
    required String configPath,
  }) async {
    return PiperTTS._(
      await compute(piper_dart.PiperTTS.create, (
        modelPath: modelPath,
        configPath: configPath,
      )),
    );
  }

  void speak(String text) => compute(_tts.speak, text);

  void pause() => compute(_tts.pause, null);

  void resume() => compute(_tts.resume, null);

  void stop() => compute(_tts.stop, null);
}
