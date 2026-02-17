use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::KeyboardEvent;

pub struct Input {
    keys: Rc<RefCell<HashSet<String>>>,
}

impl Input {
    pub fn new() -> Self {
        let keys: Rc<RefCell<HashSet<String>>> = Rc::new(RefCell::new(HashSet::new()));
        let window = web_sys::window().expect("no global window");

        // keydown
        {
            let keys = Rc::clone(&keys);
            let closure = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                let event: KeyboardEvent = event.unchecked_into();
                if event.repeat() {
                    return;
                }
                keys.borrow_mut().insert(event.code());
            });
            window
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .expect("failed to add keydown listener");
            closure.forget();
        }

        // keyup
        {
            let keys = Rc::clone(&keys);
            let closure = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                let event: KeyboardEvent = event.unchecked_into();
                keys.borrow_mut().remove(&event.code());
            });
            window
                .add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())
                .expect("failed to add keyup listener");
            closure.forget();
        }

        Self { keys }
    }

    pub fn is_key_down(&self, code: &str) -> bool {
        self.keys.borrow().contains(code)
    }
}
