#ifndef BORNO_FREE_FFI_H
#define BORNO_FREE_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum BornoActionKind {
    BORNO_ACTION_PASSTHROUGH = 0,
    BORNO_ACTION_EMIT = 1,
    BORNO_ACTION_REPLACE = 2,
    BORNO_ACTION_MODE_SWITCH = 3,
};

typedef struct BornoEngine BornoEngine;

typedef struct BornoAction {
    uint32_t kind;
    size_t backspaces;
    char *text;
} BornoAction;

BornoEngine *borno_engine_new(void);
void borno_engine_reset(BornoEngine *engine);
void borno_engine_set_mode(BornoEngine *engine, uint32_t mode);
void borno_engine_free(BornoEngine *engine);
BornoAction borno_engine_handle_key(BornoEngine *engine, uint32_t key,
                                    uint8_t shift, uint8_t ctrl, uint8_t alt);
void borno_action_free(char *text);

#ifdef __cplusplus
}
#endif

#endif
