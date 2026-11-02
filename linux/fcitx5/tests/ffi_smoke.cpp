#include "borno_ffi.h"

#include <cassert>
#include <cstring>
#include <string>

namespace {

std::string take(BornoAction action) {
    std::string text = action.text == nullptr ? "" : action.text;
    borno_action_free(action.text);
    return text;
}

} // namespace

int main() {
    auto *first = borno_engine_new();
    auto *second = borno_engine_new();
    assert(first != nullptr && second != nullptr);

    // A pending pre-kar in one input context must not affect another.
    auto pending = borno_engine_handle_key(first, 'd', 0, 0, 0);
    assert(pending.kind == BORNO_ACTION_REPLACE);
    assert(take(pending).empty());

    auto firstCommit = borno_engine_handle_key(first, 'j', 0, 0, 0);
    assert(firstCommit.kind == BORNO_ACTION_EMIT);
    assert(take(firstCommit) == "কি");

    auto secondCommit = borno_engine_handle_key(second, 'j', 0, 0, 0);
    assert(secondCommit.kind == BORNO_ACTION_EMIT);
    assert(take(secondCommit) == "ক");

    auto hasanta = borno_engine_handle_key(second, 'g', 0, 0, 0);
    assert(take(hasanta) == "্");
    auto replacement = borno_engine_handle_key(second, 'f', 0, 0, 0);
    assert(replacement.kind == BORNO_ACTION_REPLACE);
    assert(replacement.backspaces == 1);
    assert(take(replacement) == "আ");

    auto modifier = borno_engine_handle_key(second, 'v', 0, 1, 1);
    assert(modifier.kind == BORNO_ACTION_PASSTHROUGH);

    borno_engine_free(first);
    borno_engine_free(second);
    return 0;
}
