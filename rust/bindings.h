#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define BOS '^'

#define EOS '$'

#define PAD '_'

typedef struct FFIInitResponse {
  char *error_message;
} FFIInitResponse;

typedef struct FFICreateInstanceResponse {
  int32_t fd;
  char *error_message;
} FFICreateInstanceResponse;

typedef struct FFISpeakResponse {
  char *error_message;
} FFISpeakResponse;

struct FFIInitResponse init(void);

struct FFICreateInstanceResponse create_instance(const char *model_path, const char *config_path);

struct FFISpeakResponse speak(int32_t fd, const char *text);
