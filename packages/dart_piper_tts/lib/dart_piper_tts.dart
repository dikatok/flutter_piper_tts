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
    _checkIfError(error_message);
  }

  static PiperTTS create({
    required String modelPath,
    required String configPath,
  }) {
    return using((arena) {
      final result = g.create_instance(
        modelPath.toNativeUtf8(allocator: arena).cast(),
        configPath.toNativeUtf8(allocator: arena).cast(),
      );
      final g.FFICreateInstanceResponse(:instance, :error_message) = result;
      _checkIfError(error_message);
      return PiperTTS._(instance);
    });
  }

  Future<void> speak(
    String text, {
    bool waitForCompletion = true,
    PhonemizerStrategy phonemizerStrategy = PhonemizerStrategy.neuralWord,
  }) async {
    return using((arena) {
      final id = _nextCompletionId++;
      final playbackComplete = _completionStreamController.stream
          .firstWhere((id_) => id_ == id)
          .then((_) {});
      final result = g.speak(
        _instancePtr,
        text.toNativeUtf8(allocator: arena).cast(),
        false,
        id,
        phonemizerStrategy.native.toNativeUtf8(allocator: arena).cast(),
      );
      final g.FFISpeakResponse(:error_message) = result;
      _checkIfError(error_message);
      if (waitForCompletion) return playbackComplete;
    });
  }

  Future<void> speakFromPhonemes({
    required String phonemes,
    bool waitForCompletion = true,
  }) async {
    return using((arena) {
      final id = _nextCompletionId++;
      final playbackComplete = _completionStreamController.stream
          .firstWhere((id_) => id_ == id)
          .then((_) {});
      final result = g.speak(
        _instancePtr,
        phonemes.toNativeUtf8(allocator: arena).cast(),
        true,
        id,
        PhonemizerStrategy.neuralWord.native
            .toNativeUtf8(allocator: arena)
            .cast(),
      );
      final g.FFISpeakResponse(:error_message) = result;
      _checkIfError(error_message);
      if (waitForCompletion) return playbackComplete;
    });
  }

  void pause() {
    final result = g.pause(_instancePtr);
    final g.FFIPauseResponse(:error_message) = result;
    _checkIfError(error_message);
  }

  void resume() {
    final result = g.resume(_instancePtr);
    final g.FFIResumeResponse(:error_message) = result;
    _checkIfError(error_message);
  }

  void stop() {
    final result = g.stop(_instancePtr);
    final g.FFIStopResponse(:error_message) = result;
    _checkIfError(error_message);
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
  neuralSentence("neural_sentence"),
  neuralWord("neural_word"),
  dictionaryWithNeuralFallback('dict_neural'),
  dictionaryWithOmitUnknown('dict_omit');

  final String native;

  const PhonemizerStrategy(this.native);
}

void _checkIfError(Pointer<Char> errorMessage) {
  if (errorMessage.isNotEmpty) {
    final message = errorMessage.toDartString();
    g.free_string(errorMessage);
    throw Exception(message);
  }
}
