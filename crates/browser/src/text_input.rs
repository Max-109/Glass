use cef::{
    CefString, Domnode, Frame, ImplDomnode, ImplFrame, ImplListValue, ImplProcessMessage,
    ProcessId, ProcessMessage, process_message_create,
};
use gpui::Keystroke;

pub(crate) const TEXT_INPUT_STATE_MESSAGE_NAME: &str = "glass.text_input_state";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct BrowserTextInputState {
    pub(crate) editable: bool,
}

impl BrowserTextInputState {
    pub(crate) fn is_active(self, has_marked_text: bool) -> bool {
        self.editable || has_marked_text
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BrowserKeyDispatch {
    App,
    Browser,
    TextInput,
}

pub(crate) fn extract_text_input_state_from_message(
    message: &mut ProcessMessage,
) -> Option<BrowserTextInputState> {
    if CefString::from(&message.name()).to_string() != TEXT_INPUT_STATE_MESSAGE_NAME {
        return None;
    }

    let args = message.argument_list()?;
    Some(BrowserTextInputState {
        editable: args.bool(0) != 0,
    })
}

pub(crate) fn send_text_input_state(frame: &mut Frame, focused_node: Option<&Domnode>) -> bool {
    let Some(message) =
        process_message_create(Some(&CefString::from(TEXT_INPUT_STATE_MESSAGE_NAME)))
    else {
        return false;
    };

    let Some(args) = message.argument_list() else {
        return false;
    };

    args.set_bool(
        0,
        focused_node.is_some_and(|node| node.is_editable() != 0) as i32,
    );

    let mut message = message;
    frame.send_process_message(ProcessId::BROWSER, Some(&mut message));
    true
}

pub(crate) fn key_down_dispatch(
    keystroke: &Keystroke,
    text_input_active: bool,
) -> BrowserKeyDispatch {
    if keystroke.modifiers.platform || keystroke.modifiers.control {
        BrowserKeyDispatch::App
    } else if text_input_active && keystroke.key_char.is_some() {
        BrowserKeyDispatch::TextInput
    } else {
        BrowserKeyDispatch::Browser
    }
}

pub(crate) fn key_up_dispatch(
    keystroke: &Keystroke,
    text_input_active: bool,
) -> BrowserKeyDispatch {
    if keystroke.modifiers.platform || keystroke.modifiers.control {
        BrowserKeyDispatch::App
    } else if text_input_active && keystroke.key_char.is_some() {
        BrowserKeyDispatch::TextInput
    } else {
        BrowserKeyDispatch::Browser
    }
}

#[cfg(test)]
mod tests {
    use super::{BrowserKeyDispatch, key_down_dispatch, key_up_dispatch};
    use gpui::{Keystroke, Modifiers};

    fn keystroke(key: &str, key_char: Option<&str>, modifiers: Modifiers) -> Keystroke {
        Keystroke {
            key: key.into(),
            key_char: key_char.map(str::to_string),
            modifiers,
            native_key_code: None,
        }
    }

    #[test]
    fn printable_keys_use_browser_route_when_page_is_not_editable() {
        let keystroke = keystroke("e", Some("e"), Modifiers::default());

        assert_eq!(
            key_down_dispatch(&keystroke, false),
            BrowserKeyDispatch::Browser
        );
        assert_eq!(
            key_up_dispatch(&keystroke, false),
            BrowserKeyDispatch::Browser
        );
    }

    #[test]
    fn printable_keys_use_text_input_when_page_is_editable() {
        let keystroke = keystroke("e", Some("e"), Modifiers::default());

        assert_eq!(
            key_down_dispatch(&keystroke, true),
            BrowserKeyDispatch::TextInput
        );
        assert_eq!(
            key_up_dispatch(&keystroke, true),
            BrowserKeyDispatch::TextInput
        );
    }

    #[test]
    fn navigation_keys_still_reach_browser_when_page_is_editable() {
        let keystroke = keystroke("left", None, Modifiers::default());

        assert_eq!(
            key_down_dispatch(&keystroke, true),
            BrowserKeyDispatch::Browser
        );
        assert_eq!(
            key_up_dispatch(&keystroke, true),
            BrowserKeyDispatch::Browser
        );
    }

    #[test]
    fn command_shortcuts_stay_in_app_dispatch() {
        let keystroke = keystroke("c", Some("c"), Modifiers::command());

        assert_eq!(key_down_dispatch(&keystroke, true), BrowserKeyDispatch::App);
        assert_eq!(key_up_dispatch(&keystroke, true), BrowserKeyDispatch::App);
    }
}
