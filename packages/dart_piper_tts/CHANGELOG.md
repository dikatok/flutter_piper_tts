## 0.2.1
- Missing phonemizer model

## 0.2.0
- use opaque pointers, so rust object will be released as well when the dart container class is disposed
- use streamController for managing completion, preventing dangling completers
- re-factor phonemizer, now with more options (neural and dict based, and their offspring)
- include phonemizer model within the rust asset, so dart package no longer need to provide the model and flutter package no longer include it in flutter asset
- support common punctuations

## 0.1.0
- Disable building native assets for intel mac target
- Migrate phonemizer from espeak derivative to neural based to avoid dealing with GPL3 license
- Expose `speakFromPhonemes`

## 0.0.1
- Initial release (play, pause, resume, stop)
