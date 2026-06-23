import 'dart:async';
import 'dart:isolate';

import 'package:dart_piper_tts/dart_piper_tts.dart' as native_piper;
import 'package:flutter/foundation.dart';

export 'package:dart_piper_tts/dart_piper_tts.dart' show PhonemizerStrategy;

// --- 1. INTERNAL SERVICE PROTOCOL ---

sealed class _TtsCommand {
  final int? instanceId;
  final SendPort replyPort;
  _TtsCommand(this.instanceId, this.replyPort);
}

class _CreateInstanceCmd extends _TtsCommand {
  final String modelPath;
  final String configPath;

  _CreateInstanceCmd(SendPort replyPort, this.modelPath, this.configPath)
    : super(null, replyPort);
}

class _SpeakCmd extends _TtsCommand {
  final String text;
  final bool waitForCompletion;
  final native_piper.PhonemizerStrategy strategy;
  final int phonemeChunkSize;

  _SpeakCmd(
    super.instanceId,
    super.replyPort,
    this.text,
    this.waitForCompletion,
    this.strategy,
    this.phonemeChunkSize,
  );
}

class _SpeakPhonemesCmd extends _TtsCommand {
  final String phonemes;
  final bool waitForCompletion;
  int phonemeChunkSize = 80;

  _SpeakPhonemesCmd(
    super.instanceId,
    super.replyPort,
    this.phonemes,
    this.waitForCompletion,
    this.phonemeChunkSize,
  );
}

enum _SimpleAction { pause, resume, stop, dispose }

class _SimpleCmd extends _TtsCommand {
  final _SimpleAction action;
  _SimpleCmd(super.instanceId, super.replyPort, this.action);
}

// --- 2. PUBLIC CLIENT API WRAPPER ---

/// An asynchronous, non-blocking interface for the Piper Text-to-Speech (TTS) engine.
///
/// This class acts as a client handle that communicates with a centralized, shared
/// background isolate. Offloading heavy native FFI allocations and processing
/// ensures your Flutter UI thread runs completely free of jank.
class PiperTTS {
  static SendPort? _sharedWorkerPort;
  static final List<Completer<SendPort>> _workerSpawnQueue = [];

  final int _instanceId;
  bool _isDisposed = false;

  PiperTTS._(this._instanceId);

  /// Standardizes worker initialization ensuring only ONE background isolate ever spawns.
  static Future<SendPort> _getWorkerPort() async {
    if (_sharedWorkerPort != null) return _sharedWorkerPort!;

    if (_workerSpawnQueue.isNotEmpty) {
      final completer = Completer<SendPort>();
      _workerSpawnQueue.add(completer);
      return completer.future;
    }

    final primaryCompleter = Completer<SendPort>();
    _workerSpawnQueue.add(primaryCompleter);

    final initPort = ReceivePort();
    await Isolate.spawn(_ttsWorkerIsolate, initPort.sendPort);
    _sharedWorkerPort = await initPort.first as SendPort;

    for (final completer in _workerSpawnQueue) {
      if (!completer.isCompleted) completer.complete(_sharedWorkerPort);
    }
    _workerSpawnQueue.clear();

    return _sharedWorkerPort!;
  }

  /// Allocates and initializes a new native voice model instance inside the shared
  /// background worker isolate.
  ///
  /// * [modelPath]: The absolute local file path to the Piper `.onnx` voice model.
  /// * [configPath]: The absolute local file path to the accompanying `.json` configuration file.
  ///
  /// Throws an [Exception] if the background worker fails to spin up or if the native
  /// voice initialization fails (e.g., file not found, bad model format).
  static Future<PiperTTS> create({
    required String modelPath,
    required String configPath,
  }) async {
    final worker = await _getWorkerPort();
    final replyPort = ReceivePort();

    worker.send(_CreateInstanceCmd(replyPort.sendPort, modelPath, configPath));
    final response = await replyPort.first;

    if (response is Exception) throw response;
    return PiperTTS._(response as int);
  }

  /// Sends an internal command payload to the background isolate thread and awaits its status response.
  Future<void> _sendCommand(_TtsCommand command, ReceivePort port) async {
    if (_isDisposed) throw Exception('PiperTTS instance is disposed');
    final worker = await _getWorkerPort();
    worker.send(command);

    final result = await port.first;
    if (result is Exception) throw result;
  }

  /// Synthesizes the given [text] into speech.
  ///
  /// * [text]: The plain text string to be converted to speech.
  /// * [waitForCompletion]: When set to `true` (default), the returned [Future] completes
  ///   only after the audio synthesis and playback have finished completely. When `false`,
  ///   the method returns as soon as the playback job has been queued up natively.
  /// * [phonemizerStrategy]: The level of optimization used by the text phonemizer. Defaults
  ///   to [PhonemizerStrategy.neuralWord].
  /// * [phonemeChunkSize] controls the split size of phonemes. Defaults to 80.
  /// Maximum value is 255 (max of u8). Setting to 0 will disable chunking.
  ///
  /// Throws an [Exception] if this instance has been disposed, or if a native rendering
  /// error occurs.
  Future<void> speak(
    String text, {
    bool waitForCompletion = true,
    native_piper.PhonemizerStrategy phonemizerStrategy =
        native_piper.PhonemizerStrategy.neuralWord,
    int phonemeChunkSize = 80,
  }) async {
    final port = ReceivePort();
    await _sendCommand(
      _SpeakCmd(
        _instanceId,
        port.sendPort,
        text,
        waitForCompletion,
        phonemizerStrategy,
        phonemeChunkSize,
      ),
      port,
    );
  }

  /// Synthesizes speech directly using pre-computed native [phonemes].
  ///
  /// This method skips standard textual phonemization pipelines, lowering latency when
  /// reusing cached phoneme sequences.
  ///
  /// * [phonemes]: The raw phoneme tokens to render.
  /// * [waitForCompletion]: When set to `true` (default), the returned [Future] completes
  ///   only after the audio synthesis and playback have finished completely.
  ///
  /// * [phonemeChunkSize] controls the split size of phonemes. Defaults to 80.
  /// Maximum value is 255 (max of u8). Setting to 0 will disable chunking.
  ///
  /// Throws an [Exception] if this instance has been disposed, or if a native rendering
  /// error occurs.
  Future<void> speakFromPhonemes({
    required String phonemes,
    bool waitForCompletion = true,
    int phonemeChunkSize = 80,
  }) async {
    final port = ReceivePort();
    await _sendCommand(
      _SpeakPhonemesCmd(
        _instanceId,
        port.sendPort,
        phonemes,
        waitForCompletion,
        phonemeChunkSize,
      ),
      port,
    );
  }

  /// Pauses the current audio playback stream for this instance.
  ///
  /// Active synthesis jobs remain loaded in memory and can be resumed with [resume].
  Future<void> pause() async {
    final port = ReceivePort();
    await _sendCommand(
      _SimpleCmd(_instanceId, port.sendPort, _SimpleAction.pause),
      port,
    );
  }

  /// Resumes an audio playback stream that was previously suspended by a call to [pause].
  Future<void> resume() async {
    final port = ReceivePort();
    await _sendCommand(
      _SimpleCmd(_instanceId, port.sendPort, _SimpleAction.resume),
      port,
    );
  }

  /// Immediately halts any audio playback and synthesis processing running on this instance.
  Future<void> stop() async {
    final port = ReceivePort();
    await _sendCommand(
      _SimpleCmd(_instanceId, port.sendPort, _SimpleAction.stop),
      port,
    );
  }

  /// Cleanly closes this handle and deallocates its underlying native model resources.
  ///
  /// Once a [PiperTTS] instance is disposed, calling any execution methods on it will
  /// throw an error. Subsequent requests should allocate a new object via [create].
  Future<void> dispose() async {
    if (_isDisposed) return;
    _isDisposed = true;
    final port = ReceivePort();
    await _sendCommand(
      _SimpleCmd(_instanceId, port.sendPort, _SimpleAction.dispose),
      port,
    );
  }
}

// --- 3. BACKGROUND WORKER ISOLATE ---

void _ttsWorkerIsolate(SendPort mainIsolatePort) {
  final commandPort = ReceivePort();
  mainIsolatePort.send(commandPort.sendPort);

  native_piper.PiperTTS.init(kDebugMode: kDebugMode);

  final Map<int, native_piper.PiperTTS> activeInstances = {};
  int nextInstanceId = 0;

  commandPort.listen((message) async {
    if (message is _CreateInstanceCmd) {
      try {
        final tts = native_piper.PiperTTS.create(
          modelPath: message.modelPath,
          configPath: message.configPath,
        );
        final id = nextInstanceId++;
        activeInstances[id] = tts;
        message.replyPort.send(id);
      } catch (e) {
        message.replyPort.send(
          Exception('Failed to initialize PiperTTS Instance: $e'),
        );
      }
      return;
    }

    final tts = activeInstances[message.instanceId];
    if (tts == null) {
      message.replyPort.send(
        Exception('Native instance already dropped or invalid.'),
      );
      return;
    }

    if (message is _SpeakCmd) {
      try {
        await tts.speak(
          message.text,
          waitForCompletion: message.waitForCompletion,
          phonemizerStrategy: message.strategy,
          phonemeChunkSize: message.phonemeChunkSize,
        );
        message.replyPort.send(true);
      } catch (e) {
        message.replyPort.send(Exception(e.toString()));
      }
    } else if (message is _SpeakPhonemesCmd) {
      try {
        await tts.speakFromPhonemes(
          phonemes: message.phonemes,
          waitForCompletion: message.waitForCompletion,
          phonemeChunkSize: message.phonemeChunkSize,
        );
        message.replyPort.send(true);
      } catch (e) {
        message.replyPort.send(Exception(e.toString()));
      }
    } else if (message is _SimpleCmd) {
      try {
        switch (message.action) {
          case _SimpleAction.pause:
            tts.pause();
            break;
          case _SimpleAction.resume:
            tts.resume();
            break;
          case _SimpleAction.stop:
            tts.stop();
            break;
          case _SimpleAction.dispose:
            tts.dispose();
            activeInstances.remove(message.instanceId);
            break;
        }
        message.replyPort.send(true);
      } catch (e) {
        message.replyPort.send(Exception(e.toString()));
      }
    }
  });
}
