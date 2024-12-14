use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use twust::tw;

use leptos::web_sys::console;
use wasm_bindgen::JsValue;
use web3::contract::{Contract, Options};
use web3::transports::http::Http;
use web3::types::{Address, U256};
use web3::Web3;

use crate::env;

#[derive(Clone, Debug, PartialEq)]
pub struct Campaign {
    id: u128,
    creator: String, // Convert the `Address` to a readable string
    title: String,
    description: String,
    goal: f64,
    funds_raised: f64,
    completed: bool,
}

#[derive(Clone)]
pub struct AppState {
    web3: Web3<Http>,
}

impl AppState {
    pub fn new(rpc_url: &str) -> Self {
        let transport = Http::new(rpc_url).expect("Failed to create HTTP transport");
        let web3 = Web3::new(transport);
        AppState { web3 }
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let rpc_url = env::APP_RPC_URL;

    let state = AppState::new(&rpc_url);
    provide_context(state.clone());

    view! {
        <script src="https://cdn.tailwindcss.com"></script>
        <Title text="Afunding"/>
        <Router>
            <Header/>
            <main class=tw!("p-4")>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("create") view=CreateCampaignPage/>
                    <Route path=StaticSegment("campaigns") view=CampaignListPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not found");
    let (block_number, set_block_number) = signal(None);

    // Fetch the block number when the component is mounted
    create_effect(move |_| {
        let web3 = state.web3.clone(); // Clone web3 to avoid moving it
        spawn_local(async move {
            match web3.eth().block_number().await {
                Ok(number) => set_block_number.set(Some(number.as_u64())),
                Err(err) => {
                    eprintln!("Error fetching block number: {:?}", err);
                    set_block_number.set(None);
                }
            }
        });
    });

    view! {
        <h1 class=tw!("text-2xl font-bold")>"Welcome to Afunding"</h1>
        <p class=tw!("text-gray-600")>
            "Current Block Number: "
            {move || block_number.get().map_or("Loading...".to_string(), |n| n.to_string())}
        </p>
    }
}

#[component]
fn CreateCampaignPage() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not found");
    let title = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let goal = RwSignal::new(String::new());
    let message = RwSignal::new(None::<String>);

    let on_submit = move |_| {
        let web3 = state.web3.clone();
        let contract_address: Address = env::APP_CONTRACT_ADDRESS
            .trim_start_matches("0x")
            .parse()
            .unwrap();

        let contract_abi = include_bytes!("../abis.json");
        let contract = Contract::from_json(web3.eth(), contract_address, contract_abi).unwrap();

        let params = (
            title.get().clone(),
            description.get().clone(),
            U256::from_dec_str(&goal.get()).unwrap_or(U256::zero()),
        );

        log(env::APP_SENDER_ADDRESS);

        let sender_address: Address = env::APP_SENDER_ADDRESS.parse().unwrap();

        spawn_local(async move {
            let options = Options {
                ..Default::default()
            };

            match contract
                .call::<(std::string::String, std::string::String, U256)>(
                    "createCampaign",
                    params,
                    sender_address,
                    options,
                )
                .await
            {
                Ok(_) => message.set(Some("Campaign created successfully!".into())),
                Err(err) => message.set(Some(format!("Error: {:?}", err))),
            }
        });
    };

    view! {
        <h1 class=tw!("text-3xl font-bold mb-4")>"Create a New Campaign"</h1>
        <form class=tw!("space-y-4")>
            <div>
                <label for="title" class=tw!("block text-gray-700")>"Title"</label>
                <input id="title" type="text" bind:value=title class=tw!("w-full border rounded p-2")/>
            </div>
            <div>
                <label for="description" class=tw!("block text-gray-700")>"Description"</label>
                <textarea id="description" bind:value=description class=tw!("w-full border rounded p-2")></textarea>
            </div>
            <div>
                <label for="goal" class=tw!("block text-gray-700")>"Funding Goal (ETH)"</label>
                <input id="goal" type="number" bind:value=goal class=tw!("w-full border rounded p-2")/>
            </div>
            <button type="button" on:click=on_submit class=tw!("bg-blue-500 text-white p-2 rounded")>
                "Create Campaign"
            </button>
        </form>
        {move || if let Some(msg) = message.get() {
            view! { <p class=tw!("text-green-500 mt-4")>{msg}</p> }
        } else {
            view! { <p class=tw!("text-green-500 mt-4")>{String::new()}</p> }
        }}
    }
}

#[component]
fn CampaignListPage() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not found");
    let campaigns = RwSignal::new(vec![]);

    create_effect(move |_| {
        let web3 = state.web3.clone();
        let campaigns_signal = campaigns.clone();

        spawn_local(async move {
            log("Starting to fetch campaigns...");

            let contract_address: Address = env::APP_CONTRACT_ADDRESS.parse().unwrap();

            let contract_abi = include_bytes!("../abis.json");

            let contract = match Contract::from_json(web3.eth(), contract_address, contract_abi) {
                Ok(contract) => contract,
                Err(err) => {
                    log(format!("Failed to load contract: {:?}", err));
                    return;
                }
            };

            // Fetch campaign count
            let campaign_count: U256 = match contract
                .query("campaignCount", (), None, Options::default(), None)
                .await
            {
                Ok(count) => {
                    log(format!("Campaign count: {}", count));
                    count
                }
                Err(err) => {
                    log(format!("Error fetching campaign count: {:?}", err));
                    return;
                }
            };

            let mut fetched_campaigns = Vec::new();

            // Fetch each campaign
            for i in 0..campaign_count.as_u64() {
                log(format!("Fetching campaign at index: {}", i));
                let campaign: (Address, String, String, U256, U256, bool) = match contract
                    .query("campaigns", (i,), None, Options::default(), None)
                    .await
                {
                    Ok(data) => {
                        log(format!("Fetched campaign: {:?}", data));
                        data
                    }
                    Err(err) => {
                        log(format!("Error fetching campaign at index {}: {:?}", i, err));
                        continue;
                    }
                };

                fetched_campaigns.push(Campaign {
                    id: i as u128,
                    creator: format!("{:?}", campaign.0), // Convert Address to String
                    title: campaign.1,
                    description: campaign.2,
                    goal: campaign.3.as_u64() as f64,
                    funds_raised: campaign.4.as_u64() as f64,
                    completed: campaign.5,
                });
            }

            // Update the signal
            log(format!("Fetched campaigns: {:?}", fetched_campaigns));
            campaigns_signal.set(fetched_campaigns);
        });
    });

    view! {
        <h1 class=tw!("text-3xl font-bold mb-4")>"Active Campaigns"</h1>
        <ul class=tw!("space-y-4")>
            {move || campaigns.get().iter().map(|campaign| {
                view! {
                    <li class=tw!("bg-gray-100 p-4 rounded shadow-lg")>
                        <h2 class=tw!("text-xl font-bold")>{campaign.title.clone()}</h2>
                        <p class=tw!("text-gray-700")>{campaign.description.clone()}</p>
                        <p class=tw!("text-gray-500")>
                            "Goal: "
                            {campaign.goal}
                            " ETH"
                        </p>
                        <p class=tw!("text-gray-500")>
                            "Funds Raised: "
                            {campaign.funds_raised}
                            " ETH"
                        </p>
                        <p class=tw!("text-gray-500")>
                            "Creator: "
                            {campaign.creator.clone()}
                        </p>
                        <p class=tw!("text-gray-500")>
                            "Completed: "
                            {if campaign.completed { "Yes" } else { "No" }}
                        </p>
                    </li>
                }
            }).collect::<Vec<_>>()}
        </ul>
    }
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <header class=tw!("bg-gray-800 text-white p-4")>
            <nav class=tw!("flex justify-between items-center")>
                <h1 class=tw!("text-2xl font-bold")>"Afunding"</h1>
                <ul class=tw!("flex space-x-4")>
                    <li><a href="/" class=tw!("hover:text-blue-400")>"Home"</a></li>
                    <li><a href="/create" class=tw!("hover:text-blue-400")>"Create Campaign"</a></li>
                    <li><a href="/campaigns" class=tw!("hover:text-blue-400")>"View Campaigns"</a></li>
                </ul>
            </nav>
        </header>
    }
}

pub fn log(value: impl Into<JsValue>) {
    console::log_1(&value.into());
}
