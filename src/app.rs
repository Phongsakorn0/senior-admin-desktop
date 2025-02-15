use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, File};
use yew::prelude::*;
use gloo_utils::format::JsValueSerdeExt;
use log::{info, error, debug};
use gloo_file::{File as GlooFile, callbacks::read_as_bytes}; // Import read_as_bytes
use base64::engine::general_purpose::STANDARD; // Use the STANDARD engine
use base64::Engine as _; // Import the Engine trait

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
    let file_base64 = use_state(|| String::new());

    // Fetch client data callback
    let fetch_client_data = {
        let client_data = client_data.clone();
        let status_msg = status_msg.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let client_data = client_data.clone();
            let status_msg = status_msg.clone();
            
            spawn_local(async move {
                info!("Starting client data fetch");

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

    // Notification submit callback
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

    // File upload callback
    let file_upload_ref = use_node_ref();
    let on_file_upload = {
        let file_upload_ref = file_upload_ref.clone();
        let file_base64 = file_base64.clone();
        let status_msg = status_msg.clone();

        Callback::from(move |_e: Event| {
            let file_base64 = file_base64.clone();
            let status_msg = status_msg.clone();

            if let Some(input) = file_upload_ref.cast::<HtmlInputElement>() {
                if let Some(file_list) = input.files() {
                    if let Some(file) = file_list.get(0) {
                        let file_name = file.name();
                        let file = GlooFile::from(file);

                        info!("File selected: {}", file_name);

                        // Use read_as_bytes with a callback
                        let _reader = read_as_bytes(&file, move |result| {
                            match result {
                                Ok(bytes) => {
                                    info!("File read successfully: {} bytes", bytes.len());

                                    // Encode the bytes as base64 using the STANDARD engine
                                    let base64_string = STANDARD.encode(&bytes);
                                    info!("File encoded as base64: {} characters", base64_string.len());

                                    // Update the state
                                    file_base64.set(base64_string);
                                    status_msg.set(format!("File '{}' converted to Base64!", file_name));
                                }
                                Err(e) => {
                                    error!("File read error: {:?}", e);
                                    status_msg.set(format!("Failed to encode file: {}", e));
                                }
                            }
                        });
                    } else {
                        error!("No file selected");
                        status_msg.set("No file selected".into());
                    }
                } else {
                    error!("No files found in input");
                    status_msg.set("No files found in input".into());
                }
            } else {
                error!("Failed to cast input to HtmlInputElement");
                status_msg.set("Failed to cast input to HtmlInputElement".into());
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

            // File Upload Section
            <div class="file-upload">
                <h2>{ "Upload File for Base64 Encoding" }</h2>
                <input ref={file_upload_ref} type="file" accept=".pdf" onchange={on_file_upload.clone()} />
                <div class="file-output">
                    <span class="base64">{&*file_base64}</span>
                </div>
            </div>

            // Status Messages
            <p class="status-message :">{ &*status_msg }</p>
        </main>
    }
}