import "package:dart_piper_tts/dart_piper_tts.dart" as piper_dart;
import "package:path_provider/path_provider.dart";

class PiperTTS {
  final piper_dart.PiperTTS _tts;

  PiperTTS._(this._tts);

  static Future<void> init({String? dataDir}) async {
    piper_dart.PiperTTS.init((
      dataDir: dataDir ?? (await getApplicationDocumentsDirectory()).path,
    ));
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

  void pause() => _tts.pause();

  void resume() => _tts.resume();

  void stop() => _tts.stop();
}
