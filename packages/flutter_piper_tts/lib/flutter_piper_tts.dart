import 'dart:async';
import 'dart:isolate';

import 'package:dart_piper_tts/dart_piper_tts.dart' as native_piper;
import 'package:flutter/foundation.dart';

export 'package:dart_piper_tts/dart_piper_tts.dart' show PhonemizerStrategy;

// --- 1. INTERNAL MESSAGE CLASSES ---
// These safely cross the isolate boundary.
sealed class _TtsCommand {}

class _SpeakCmd extends _TtsCommand {
  final SendPort replyPort;
  final String text;
  final bool waitForCompletion;
  final native_piper.PhonemizerStrategy strategy;

  _SpeakCmd(this.replyPort, this.text, this.waitForCompletion, this.strategy);
}

class _SpeakPhonemesCmd extends _TtsCommand {
  final SendPort replyPort;
  final String phonemes;
  final bool waitForCompletion;

  _SpeakPhonemesCmd(this.replyPort, this.phonemes, this.waitForCompletion);
}

enum _SimpleCmdAction { pause, resume, stop, dispose }

class _SimpleCmd extends _TtsCommand {
  final SendPort replyPort;
  final _SimpleCmdAction action;

  _SimpleCmd(this.replyPort, this.action);
}

class _IsolateInitData {
  final SendPort replyPort;
  final String modelPath;
  final String configPath;

  _IsolateInitData(this.replyPort, this.modelPath, this.configPath);
}

// --- 2. PUBLIC API WRAPPER ---

/// An asynchronous, non-blocking wrapper for PiperTTS.
/// Offloads all FFI calls to a background isolate to prevent UI jank.
class PiperTTS {
  final SendPort _commandPort;
  bool _isDisposed = false;

  PiperTTS._(this._commandPort);

  /// Initializes the background isolate and loads the Piper model.
  static Future<PiperTTS> create({
    required String modelPath,
    required String configPath,
  }) async {
    final initPort = ReceivePort();

    await Isolate.spawn(
      _ttsWorkerIsolate,
      _IsolateInitData(initPort.sendPort, modelPath, configPath),
    );

    final response = await initPort.first;

    if (response is Exception) {
      throw response;
    }

    return PiperTTS._(response as SendPort);
  }

  /// Sends a command to the isolate and waits for the result.
  Future<void> _sendCommand(_TtsCommand command, ReceivePort port) async {
    if (_isDisposed) throw Exception('PiperTTS is disposed');
    _commandPort.send(command);

    final result = await port.first;
    if (result is Exception) throw result;
  }

  Future<void> speak(
    String text, {
    bool waitForCompletion = true,
    native_piper.PhonemizerStrategy phonemizerStrategy =
        native_piper.PhonemizerStrategy.neuralWord,
  }) async {
    final port = ReceivePort();
    await _sendCommand(
      _SpeakCmd(port.sendPort, text, waitForCompletion, phonemizerStrategy),
      port,
    );
  }

  Future<void> speakFromPhonemes({
    required String phonemes,
    bool waitForCompletion = true,
  }) async {
    final port = ReceivePort();
    await _sendCommand(
      _SpeakPhonemesCmd(port.sendPort, phonemes, waitForCompletion),
      port,
    );
  }

  Future<void> pause() async {
    final port = ReceivePort();
    await _sendCommand(_SimpleCmd(port.sendPort, .pause), port);
  }

  Future<void> resume() async {
    final port = ReceivePort();
    await _sendCommand(_SimpleCmd(port.sendPort, .resume), port);
  }

  Future<void> stop() async {
    final port = ReceivePort();
    await _sendCommand(_SimpleCmd(port.sendPort, .stop), port);
  }

  Future<void> dispose() async {
    if (_isDisposed) return;
    _isDisposed = true;
    final port = ReceivePort();
    await _sendCommand(_SimpleCmd(port.sendPort, .dispose), port);
  }
}

// --- 3. BACKGROUND WORKER ISOLATE ---

/// The entry point for the background isolate.
/// All heavy native FFI work happens safely here.
void _ttsWorkerIsolate(_IsolateInitData data) {
  final commandPort = ReceivePort();
  late native_piper.PiperTTS tts;

  try {
    // Initialize the native layer inside this specific isolate
    native_piper.PiperTTS.init(kDebugMode: kDebugMode);

    // Load the model (This is the heaviest operation)
    tts = native_piper.PiperTTS.create(
      modelPath: data.modelPath,
      configPath: data.configPath,
    );

    // Send the command port back to the main thread
    data.replyPort.send(commandPort.sendPort);
  } catch (e) {
    data.replyPort.send(Exception('Failed to initialize PiperTTS: $e'));
    return;
  }

  // Listen for instructions from the main thread
  commandPort.listen((message) async {
    if (message is _SpeakCmd) {
      try {
        await tts.speak(
          message.text,
          waitForCompletion: message.waitForCompletion,
          phonemizerStrategy: message.strategy,
        );
        message.replyPort.send(true); // Success
      } catch (e) {
        message.replyPort.send(Exception(e.toString()));
      }
    } else if (message is _SpeakPhonemesCmd) {
      try {
        await tts.speakFromPhonemes(
          phonemes: message.phonemes,
          waitForCompletion: message.waitForCompletion,
        );
        message.replyPort.send(true);
      } catch (e) {
        message.replyPort.send(Exception(e.toString()));
      }
    } else if (message is _SimpleCmd) {
      try {
        switch (message.action) {
          case .pause:
            tts.pause();
            break;
          case .resume:
            tts.resume();
            break;
          case .stop:
            tts.stop();
            break;
          case .dispose:
            tts.dispose();
            commandPort.close(); // Kills the isolate safely
            break;
        }
        message.replyPort.send(true);
      } catch (e) {
        message.replyPort.send(Exception(e.toString()));
      }
    }
  });
}
