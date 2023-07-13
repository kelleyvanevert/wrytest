use serde_json::{json, Value};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{canonicalize, read},
};
use tao::window::WindowId;
use wry::{
    application::{
        accelerator::Accelerator,
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        keyboard::{KeyCode, ModifiersState},
        menu::{MenuBar, MenuItemAttributes},
        window::{Window, WindowBuilder},
    },
    http::{header::CONTENT_TYPE, Method, Response},
    webview::{WebView, WebViewBuilder},
};

thread_local! {
    static WEBVIEWS: RefCell<HashMap<WindowId, WebView>> = RefCell::new(HashMap::new());
}

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();

    let mut menu = MenuBar::new();
    let mut file_menu = MenuBar::new();
    file_menu.add_native_item(tao::menu::MenuItem::Cut);
    file_menu.add_native_item(tao::menu::MenuItem::Copy);
    file_menu.add_native_item(tao::menu::MenuItem::Paste);
    file_menu.add_item(
        MenuItemAttributes::new("Quit").with_accelerators(&Accelerator::new(
            Some(ModifiersState::CONTROL | ModifiersState::SHIFT),
            KeyCode::KeyQ,
        )),
    );
    file_menu.add_item(
        MenuItemAttributes::new("Quit").with_accelerators(&Accelerator::new(None, KeyCode::KeyQ)),
    );
    file_menu.add_item(
        MenuItemAttributes::new("Quit").with_accelerators(&Accelerator::new(
            Some(ModifiersState::SHIFT),
            KeyCode::KeyQ,
        )),
    );
    menu.add_submenu("File", true, file_menu);

    let window = WindowBuilder::new()
        .with_title("Kelley's cool live-coding synth")
        .with_menu(menu)
        .build(&event_loop)?;

    let window_id = window.id();

    let webview = WebViewBuilder::new(window)?
        .with_ipc_handler(move |_: &Window, message: String| {
            let value: Value = serde_json::from_str(&message).unwrap();
            println!("Received ipc message: {:?}", value);

            WEBVIEWS.with(|webviews| {
                let webviews = webviews.borrow();
                if let Some(webview) = webviews.get(&window_id) {
                    let _ = webview.evaluate_script(&format!(
                        "_finishAsyncApiCall({}, 'resolve', {})",
                        value["id"],
                        serde_json::to_string(&json!({
                            "name": "Kelley van Evert",
                            "age": 32,
                            "args": value["args"],
                        }))
                        .unwrap()
                    ));
                }
            });
        })
        .with_devtools(true)
        .with_custom_protocol("kelley".into(), move |request| {
            if request.method() == Method::POST {
                let data = String::from_utf8_lossy(request.body());
                let value: Value = serde_json::from_str(&data)?;
                println!("Received message {}, {:?}", value["name"], value["args"]);
            }

            // remove leading slash
            let path = &request.uri().path()[1..];

            Response::builder()
                .header(CONTENT_TYPE, "text/html")
                .body(read(canonicalize(path)?)?.into())
                .map_err(Into::into)
        })
        .with_url("kelley://localhost/src/test.html")?
        // .with_url("https://tauri.studio")?
        .build()?;

    WEBVIEWS.with(|webviews| {
        webviews.borrow_mut().insert(window_id, webview);
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
