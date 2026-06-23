## 0.5.2
- Improve control latency
- Expose phoneme chunk size, long sentence = long phoneme = longer wait for low-end hardwares, by chunking it can lower time to first bytes (audio) but can disregard punctuations
- Fix dispose

## 0.5.1
- Fix issue when creating multiple instances, older instances are not able to wait for completion

## 0.5.0
- Use isolate to prevent ui janks
- Bump dart_piper_tts to 0.2.3

## 0.4.4
- Bump dart_piper_tts to 0.2.2

## 0.4.2
- Re-add dart_piper_tts version since i am not using melos auto generation

## 0.4.2
- Missing phonemizer model

## 0.4.1
- Re-add `android_libcpp_shared`

## 0.4.0
- For now, flutter package only re-export dart package, to prevent breaking package users
- In the future there might be improvements made specifically in the flutter realm

## 0.3.0
- Disable building native assets for intel mac target
- Migrate phonemizer from espeak derivative to neural based to avoid dealing with GPL3 license
- Expose `speakFromPhonemes`
- Fix hot restart

## 0.2.0
- separate `dart_piper_tts` from `flutter_piper_tts`
- add `waitForCompletion` flag for `speak`

## 0.1.0
- Fix android compatibility
- Implement play, pause, resume and stop

## 0.0.1
- Initial release
