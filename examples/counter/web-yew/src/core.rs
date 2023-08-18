use futures_util::TryStreamExt;
use gloo_console::log;
use shared::{App, Capabilities, Effect, Event};
use std::rc::Rc;
use yew::{platform::spawn_local, Callback};

use crate::{http, sse};

pub type Core = Rc<shared::Core<Effect, App>>;

pub enum Message {
    Event(Event),
    Effect(Effect),
}

pub fn new() -> Core {
    Rc::new(shared::Core::new::<Capabilities>())
}

pub fn update(core: &Core, event: Event, render: &Callback<Message>) {
    log!(format!("event: {:?}", event));
    let effects = core.process_event(event);
    for effect in effects {
        process_effect(core, effect, render);
    }
}

pub fn process_effect(core: &Core, effect: Effect, render: &Callback<Message>) {
    log!(format!("effect: {:?}", effect));
    match effect {
        x @ Effect::Render(_) => render.emit(Message::Effect(x)),
        Effect::Http(mut request) => {
            spawn_local({
                let core = core.clone();
                let render = render.clone();

                async move {
                    let response = http::request(&request.operation).await.unwrap();

                    let effects = core.resolve(&mut request, response);
                    for effect in effects {
                        process_effect(&core, effect, &render);
                    }
                }
            });
        }
        Effect::ServerSentEvents(mut request) => {
            spawn_local({
                let core = core.clone();
                let render = render.clone();

                async move {
                    let mut stream = sse::request(&request.operation).await.unwrap();

                    while let Ok(Some(response)) = stream.try_next().await {
                        let effects = core.resolve(&mut request, response);
                        for effect in effects {
                            process_effect(&core, effect, &render);
                        }
                    }
                }
            });
        }
    }
}
