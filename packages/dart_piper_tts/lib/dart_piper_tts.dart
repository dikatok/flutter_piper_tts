import 'dart:async';
import 'dart:ffi';
import 'dart:isolate';

import 'package:dart_piper_tts/src/ffi.g.dart' as g;
import 'package:ffi/ffi.dart';

class PiperTTS {
  static final Map<int, Completer<void>> _completers = {};

  static RawReceivePort? _receivePort;

  static NativeCallable<Void Function(Int64)>? _nativeCompletionCallback;

  static int _nextId = 0;

  final int _fd;

  PiperTTS._(this._fd);

  static void _onNativeComplete(int port) {
    _receivePort!.sendPort.send(port);
  }

  static void init(({String phonemizerModelPath, bool kDebugMode}) args) {
    final phonomizerModelPointer = args.phonemizerModelPath.toNativeUtf8();

    _receivePort ??= RawReceivePort((dynamic port) {
      _completers.remove(port as int)?.complete();
    });

    _nativeCompletionCallback ??= NativeCallable<Void Function(Int64)>.listener(
      _onNativeComplete,
    );

    try {
      final result = g.init(
        phonomizerModelPointer.cast<Char>(),
        _nativeCompletionCallback!.nativeFunction,
        args.kDebugMode,
      );
      final g.FFIInitResponse(:error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
    } finally {
      calloc.free(phonomizerModelPointer);
    }
  }

  static PiperTTS create(({String modelPath, String configPath}) args) {
    final modelPathPointer = args.modelPath.toNativeUtf8();
    final configPathPointer = args.configPath.toNativeUtf8();

    try {
      final result = g.create_instance(
        modelPathPointer.cast<Char>(),
        configPathPointer.cast<Char>(),
      );
      final g.FFICreateInstanceResponse(:fd, :error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
      return PiperTTS._(fd);
    } finally {
      calloc.free(modelPathPointer);
      calloc.free(configPathPointer);
    }
  }

  Future<void> speak(String text, {bool waitForCompletion = true}) async {
    final textPointer = text.toNativeUtf8();
    final id = _nextId++;
    final completer = Completer<void>();
    _completers[id] = completer;
    try {
      final result = g.speak(_fd, textPointer.cast<Char>(), false, id);
      final g.FFISpeakResponse(:error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
    } finally {
      calloc.free(textPointer);
    }
    if (waitForCompletion) return completer.future;
  }

  Future<void> speakFromPhonemes({
    required String phonemes,
    bool waitForCompletion = true,
  }) async {
    final textPointer = phonemes.toNativeUtf8();
    final id = _nextId++;
    final completer = Completer<void>();
    _completers[id] = completer;
    try {
      final result = g.speak(_fd, textPointer.cast<Char>(), true, id);
      final g.FFISpeakResponse(:error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
    } finally {
      calloc.free(textPointer);
    }
    if (waitForCompletion) return completer.future;
  }

  void pause([dynamic _]) {
    final result = g.pause(_fd);
    final g.FFIPauseResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }

  void resume([dynamic _]) {
    final result = g.resume(_fd);
    final g.FFIResumeResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }

  void stop([dynamic _]) {
    final result = g.stop(_fd);
    final g.FFIStopResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }

  void dispose() {
    final result = g.dispose(_fd);
    final g.FFIDisposeResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }
}

extension on Pointer<Char> {
  bool get isEmpty => this == nullptr || cast<Utf8>().toDartString().isEmpty;

  bool get isNotEmpty => !isEmpty;

  String toDartString() => cast<Utf8>().toDartString();
}
