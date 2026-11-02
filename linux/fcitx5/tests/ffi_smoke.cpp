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

    // Test Avro Phonetic composition via C ABI
    borno_engine_set_mode(first, 3); // 3 = Phonetic

    // 1. Live composition of "ami" -> Preedit
    auto p1 = borno_engine_handle_key(first, 'a', 0, 0, 0);
    assert(p1.kind == BORNO_ACTION_PREEDIT);
    assert(take(p1) == "আ");

    auto p2 = borno_engine_handle_key(first, 'm', 0, 0, 0);
    assert(p2.kind == BORNO_ACTION_PREEDIT);
    assert(take(p2) == "আম");

    auto p3 = borno_engine_handle_key(first, 'i', 0, 0, 0);
    assert(p3.kind == BORNO_ACTION_PREEDIT);
    assert(take(p3) == "আমি");

    // 2. Backspace within preedit
    auto pBack = borno_engine_handle_key(first, '\b', 0, 0, 0);
    assert(pBack.kind == BORNO_ACTION_PREEDIT);
    assert(take(pBack) == "আম");

    // Restore 'i'
    auto pRestore = borno_engine_handle_key(first, 'i', 0, 0, 0);
    assert(pRestore.kind == BORNO_ACTION_PREEDIT);
    assert(take(pRestore) == "আমি");

    // 3. Space word boundary -> commits "আমি "
    auto pSpace = borno_engine_handle_key(first, ' ', 0, 0, 0);
    assert(pSpace.kind == BORNO_ACTION_COMMIT);
    assert(take(pSpace) == "আমি ");

    // 4. Escape cancellation -> resets preedit
    auto pCancel1 = borno_engine_handle_key(first, 'k', 0, 0, 0);
    assert(pCancel1.kind == BORNO_ACTION_PREEDIT);
    assert(take(pCancel1) == "ক");

    auto pCancel2 = borno_engine_handle_key(first, '\x1b', 0, 0, 0);
    assert(pCancel2.kind == BORNO_ACTION_RESET);

    borno_engine_free(first);
    borno_engine_free(second);
    return 0;
}
