import 'dart:async';
import 'dart:ffi';
import 'dart:isolate';

import 'package:dart_piper_tts/src/ffi.g.dart' as g;
import 'package:ffi/ffi.dart';

class PiperTTS implements Finalizable {
  static final StreamController<int> _completionStreamController =
      StreamController<int>.broadcast();

  static RawReceivePort? _receivePort;

  static NativeCallable<Void Function(Int64)>? _nativeCompletionCallback;

  static int _nextCompletionId = 0;

  static final NativeFinalizer _finalizer = NativeFinalizer(
    Native.addressOf<NativeFunction<Void Function(Pointer<Void>)>>(
      g.dispose,
    ).cast(),
  );

  final Pointer<Void> _instancePtr;

  PiperTTS._(this._instancePtr) {
    _finalizer.attach(this, _instancePtr.cast(), detach: this);
  }

  static void _onNativeComplete(int port) {
    _receivePort!.sendPort.send(port);
  }

  static void init({bool kDebugMode = true}) {
    _receivePort ??= RawReceivePort((dynamic port) {
      _completionStreamController.add(port);
    });

    _nativeCompletionCallback ??= NativeCallable<Void Function(Int64)>.listener(
      _onNativeComplete,
    );

    final result = g.init(
      _nativeCompletionCallback!.nativeFunction,
      kDebugMode,
    );
    final g.FFIInitResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }

  static PiperTTS create({
    required String modelPath,
    required String configPath,
  }) {
    final modelPathPointer = modelPath.toNativeUtf8();
    final configPathPointer = configPath.toNativeUtf8();

    try {
      final result = g.create_instance(
        modelPathPointer.cast<Char>(),
        configPathPointer.cast<Char>(),
      );
      final g.FFICreateInstanceResponse(:instance, :error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
      return PiperTTS._(instance);
    } finally {
      calloc.free(modelPathPointer);
      calloc.free(configPathPointer);
    }
  }

  Future<void> speak(
    String text, {
    bool waitForCompletion = true,
    PhonemizerStrategy phonemizerStrategy = PhonemizerStrategy.neuralOnly,
  }) async {
    final textPointer = text.toNativeUtf8();
    final phonemizerStrategyPointer = phonemizerStrategy.native.toNativeUtf8();
    final id = _nextCompletionId++;
    final playbackComplete = _completionStreamController.stream
        .firstWhere((id_) => id_ == id)
        .then((_) {});
    try {
      final result = g.speak(
        _instancePtr,
        textPointer.cast<Char>(),
        false,
        id,
        phonemizerStrategyPointer.cast<Char>(),
      );
      final g.FFISpeakResponse(:error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
    } finally {
      calloc.free(textPointer);
      calloc.free(phonemizerStrategyPointer);
    }
    if (waitForCompletion) return playbackComplete;
  }

  Future<void> speakFromPhonemes({
    required String phonemes,
    bool waitForCompletion = true,
    PhonemizerStrategy phonemizerStrategy = PhonemizerStrategy.neuralOnly,
  }) async {
    final textPointer = phonemes.toNativeUtf8();
    final phonemizerStrategyPointer = phonemizerStrategy.native.toNativeUtf8();
    final id = _nextCompletionId++;
    final playbackComplete = _completionStreamController.stream
        .firstWhere((id_) => id_ == id)
        .then((_) {});
    try {
      final result = g.speak(
        _instancePtr,
        textPointer.cast<Char>(),
        true,
        id,
        phonemizerStrategyPointer.cast<Char>(),
      );
      final g.FFISpeakResponse(:error_message) = result;
      if (error_message.isNotEmpty) {
        throw Exception(error_message.toDartString());
      }
    } finally {
      calloc.free(textPointer);
      calloc.free(phonemizerStrategyPointer);
    }
    if (waitForCompletion) return playbackComplete;
  }

  void pause() {
    final result = g.pause(_instancePtr);
    final g.FFIPauseResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }

  void resume() {
    final result = g.resume(_instancePtr);
    final g.FFIResumeResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }

  void stop() {
    final result = g.stop(_instancePtr);
    final g.FFIStopResponse(:error_message) = result;
    if (error_message.isNotEmpty) {
      throw Exception(error_message.toDartString());
    }
  }

  void dispose() {
    _finalizer.detach(this);
    g.dispose(_instancePtr);
  }
}

extension on Pointer<Char> {
  bool get isEmpty => this == nullptr || toDartString().isEmpty;

  bool get isNotEmpty => !isEmpty;

  String toDartString() => cast<Utf8>().toDartString();
}

enum PhonemizerStrategy {
  neuralOnly("neural"),
  dictionaryWithNeuralFallback('dict_neural'),
  dictionaryWithOmitUnknown('dict_omit');

  final String native;

  const PhonemizerStrategy(this.native);
}
