use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use gloo_utils::format::JsValueSerdeExt;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize)]
struct NotifyArgs {
    message: String,  // Changed back to message (since Tauri expects this key)
}

#[function_component(App)]
pub fn app() -> Html {
    let input_ref = use_node_ref();
    let status_msg = use_state(|| String::new());

    let receive = {
        let status_msg = status_msg.clone();

        Callback::from(move |e: SubmitEvent| {
            let status_msg = status_msg.clone();
            e.prevent_default();

            spawn_local(async move {
                // Make an HTTP GET request to the /getupdatedata endpoint
                let response = Request::get("/getupdatedata")
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        if resp.ok() {
                            let data = resp.text().await.unwrap_or_else(|_| "Failed to read response text".into());
                            status_msg.set(format!("Data received: {}", data));
                        } else {
                            status_msg.set(format!("Error: {}", resp.status_text()));
                        }
                    }
                    Err(err) => {
                        status_msg.set(format!("Error: {:?}", err));
                    }
                }
            });
        })
    };

    let on_submit = {
        let input_ref = input_ref.clone();
        let status_msg = status_msg.clone();
    
        Callback::from(move |e: SubmitEvent| {
            let input_ref = input_ref.clone();
            let status_msg = status_msg.clone();
            e.prevent_default();
    
            let input_element = input_ref
                .cast::<HtmlInputElement>()
                .expect("Failed to cast node to HtmlInputElement");
            let value = input_element.value();
    
            // Convert input to integer (i32)
            match value.trim().parse::<i32>() {
                Ok(number) => {
                    spawn_local(async move {
                        // Create a JSON object with the `message` key
                        let args = serde_json::json!({ "message": number });
                        let js_value = JsValue::from_serde(&args).expect("Failed to serialize args");
    
                        let result = invoke("notify_clients", js_value).await;
    
                        if let Err(err) = result.dyn_into::<JsValue>() {
                            status_msg.set(format!("Error: {:?}", err));
                        } else {
                            status_msg.set(format!("Notification sent with number: {}", number));
                        }
                    });
                }
                Err(_) => {
                    status_msg.set("Invalid input! Please enter a valid number.".into());
                }
            }
        })
    };

    html! {
        <main class="container">
            <h1>{ "Welcome to Admin Desktop" }</h1>
            <form class="row" onsubmit={on_submit}>
                <input ref={input_ref} type="number" placeholder="Enter a number" />
                <button type="submit" class="btn btn-primary">
                    { "Notify Clients" }
                </button>
            </form>
            <form class="row" onsubmit={receive}>
                <button type="submit" class="btn btn-primary">
                    { "Receive data from Clients" }
                </button>
            </form>
            <p>{ &*status_msg }</p>
        </main>
    }
}