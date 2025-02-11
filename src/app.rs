use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use gloo_utils::format::JsValueSerdeExt;
use log::{info, error, debug};  // Add this line


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize)]
struct NotifyArgs {
    message: String,
}

#[derive(Deserialize, Debug)]
struct ClientDataResponse {
    data: i32,
}

#[function_component(App)]
pub fn app() -> Html {
    let input_ref = use_node_ref();
    let status_msg = use_state(|| String::new());
    let client_data = use_state(|| None);

// Modified fetch client data callback
let fetch_client_data = {
    let client_data = client_data.clone();
    let status_msg = status_msg.clone();

    Callback::from(move |e: SubmitEvent| {
        let client_data = client_data.clone();
        let status_msg = status_msg.clone();
        
        e.prevent_default();
        
        spawn_local(async move {
            info!("Starting client data fetch");  // Changed from log::info!
            
            match Request::get("http://localhost:8080/api/sse/getclientdata")
                .header("Accept", "application/json")
                .send()
                .await 
            {
                Ok(response) => {
                    debug!("Received response: {:?}", response);
                    
                    if response.ok() {
                        match response.json::<i32>().await {
                            Ok(data) => {
                                info!("Successfully parsed data: {}", data);
                                client_data.set(Some(data));
                                status_msg.set("Success! Data received".into());
                            }
                            Err(e) => {
                                error!("Parse error: {:?}", e);
                                status_msg.set(format!("Data format error: {}", e));
                            }
                        }
                    } else {
                        error!("HTTP error: {}", response.status());
                        status_msg.set(format!("Server error: {}", response.status()));
                    }
                }
                Err(e) => {
                    error!("Network error: {:?}", e);
                    status_msg.set("Connection failed".into());
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
    
            match value.trim().parse::<i32>() {
                Ok(number) => {
                    spawn_local(async move {
                        let args = serde_json::json!({ "message": number });
                        let js_value = JsValue::from_serde(&args).expect("Failed to serialize args");
    
                        match invoke("notify_clients", js_value).await.dyn_into::<JsValue>() {
                            Ok(_) => status_msg.set(format!("Notification sent: {}", number)),
                            Err(e) => status_msg.set(format!("Error: {:?}", e)),
                        }
                    });
                }
                Err(_) => {
                    status_msg.set("Invalid number input".into());
                }
            }
        })
    };

    html! {
        <main class="container">
            <h1>{ "Admin Dashboard" }</h1>
            
            // Notification Form
            <form class="row" onsubmit={on_submit}>
                <input ref={input_ref} type="number" placeholder="Enter notification number" />
                <button type="submit" class="btn btn-primary">
                    { "Notify Clients" }
                </button>
            </form>

            // Client Data Fetch Section
            <div class="data-section">
                <h2>{ "Client Data" }</h2>
                <form onsubmit={fetch_client_data}>
                    <button type="submit" class="btn btn-secondary">
                        { "Get Current Client Data" }
                    </button>
                </form>
                <div class="data-display">
                    {
                        match *client_data {
                            Some(data) => html! { <span class="data-value">{ data }</span> },
                            None => html! { <span class="data-placeholder">{ "No data available" }</span> },
                        }
                    }
                </div>
            </div>

            // Status Messages
            <p class="status-message">{ &*status_msg }</p>
        </main>
    }
}