#include "borno_ffi.h"

#include <cctype>
#include <string>

#include <fcitx/addonfactory.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx-utils/keysym.h>

namespace {

class BornoState final : public fcitx::InputContextProperty {
public:
    BornoState() : engine_(borno_engine_new()) {}

    ~BornoState() override { borno_engine_free(engine_); }

    void reset() { borno_engine_reset(engine_); }

    BornoEngine *engine() const { return engine_; }

private:
    BornoEngine *engine_;
};

bool normalizeKey(const fcitx::KeyEvent &event, uint32_t *key) {
    const auto sym = event.key().sym();
    if (sym == FcitxKey_BackSpace) {
        *key = '\b';
        return true;
    }
    if (sym == FcitxKey_Return || sym == FcitxKey_KP_Enter) {
        *key = '\n';
        return true;
    }
    if (sym == FcitxKey_Tab) {
        *key = '\t';
        return true;
    }

    const auto text = fcitx::Key::keySymToUTF8(sym);
    if (text.size() != 1 || static_cast<unsigned char>(text[0]) > 0x7f) {
        return false;
    }
    *key = static_cast<uint32_t>(std::tolower(static_cast<unsigned char>(text[0])));
    return true;
}

void eraseBeforeCursor(fcitx::InputContext *inputContext, size_t count) {
    if (count == 0) {
        return;
    }

    if (inputContext->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText)) {
        inputContext->deleteSurroundingText(-static_cast<int>(count), count);
        return;
    }

    // Some terminal and legacy XIM clients do not expose surrounding text.
    // Forwarding real Backspace events still gives them the required replacement.
    for (size_t i = 0; i < count; ++i) {
        inputContext->forwardKey(fcitx::Key(FcitxKey_BackSpace));
    }
}

class BornoFcitxEngine final : public fcitx::InputMethodEngineV2 {
public:
    void activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override {
        event.inputContext()->propertyFor(&stateFactory_)->reset();
    }

    void deactivate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override {
        event.inputContext()->propertyFor(&stateFactory_)->reset();
    }

    void keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &event) override {
        if (event.isRelease() || event.key().isModifier()) {
            return;
        }

        uint32_t key = 0;
        if (!normalizeKey(event, &key)) {
            return;
        }

        const auto states = event.rawKey().states();
        const auto shift = states.test(fcitx::KeyState::Shift) ? 1 : 0;
        const auto ctrl = states.test(fcitx::KeyState::Ctrl) ? 1 : 0;
        const auto alt = states.test(fcitx::KeyState::Alt) ? 1 : 0;
        auto *inputContext = event.inputContext();
        auto *state = inputContext->propertyFor(&stateFactory_);
        const auto action = borno_engine_handle_key(state->engine(), key, shift, ctrl, alt);

        std::string text = action.text == nullptr ? "" : action.text;
        borno_action_free(action.text);

        if (action.kind == BORNO_ACTION_PASSTHROUGH ||
            action.kind == BORNO_ACTION_MODE_SWITCH) {
            return;
        }

        if (action.kind == BORNO_ACTION_REPLACE) {
            eraseBeforeCursor(inputContext, action.backspaces);
        }
        if (!text.empty()) {
            inputContext->commitString(text);
        }
        event.accept();
    }

    void reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override {
        event.inputContext()->propertyFor(&stateFactory_)->reset();
    }

private:
    fcitx::FactoryFor<BornoState> stateFactory_;
};

class BornoFcitxFactory final : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *) override {
        return new BornoFcitxEngine();
    }
};

} // namespace

FCITX_ADDON_FACTORY(BornoFcitxFactory)
