mod api;

use dioxus::prelude::*;
use api::{fetch_games, search_games, GameData};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut games = use_signal(|| Vec::<GameData>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);
    let search_query = use_signal(|| String::new());

    // Load games on mount
    use_effect(move || {
        spawn(async move {
            loading.set(true);
            error.set(None);
            match fetch_games().await {
                Ok(fetched_games) => {
                    games.set(fetched_games);
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e));
                    loading.set(false);
                }
            }
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        style { {include_str!("../assets/main.css")} }
        
        div {
            style: "min-height: 100vh; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 2rem 1rem;",
            
            div {
                style: "max-width: 1400px; margin: 0 auto;",
                
                // Header Section
                div {
                    style: "text-align: center; margin-bottom: 3rem;",
                    h1 {
                        style: "color: white; font-size: 3.5rem; margin: 0 0 0.5rem 0; font-weight: 800; text-shadow: 0 4px 12px rgba(0,0,0,0.3); letter-spacing: -0.02em;",
                        "🎮 PlayShelf"
                    }
                    p {
                        style: "color: rgba(255,255,255,0.9); font-size: 1.125rem; margin: 0; font-weight: 400;",
                        "Your personal game library"
                    }
                }
                
                SearchBar {
                    search_query: search_query,
                    on_search: move |query: String| {
                        spawn(async move {
                            loading.set(true);
                            error.set(None);
                            if query.is_empty() {
                                match fetch_games().await {
                                    Ok(fetched_games) => {
                                        games.set(fetched_games);
                                        loading.set(false);
                                    }
                                    Err(e) => {
                                        error.set(Some(e));
                                        loading.set(false);
                                    }
                                }
                            } else {
                                match search_games(query).await {
                                    Ok(fetched_games) => {
                                        games.set(fetched_games);
                                        loading.set(false);
                                    }
                                    Err(e) => {
                                        error.set(Some(e));
                                        loading.set(false);
                                    }
                                }
                            }
                        });
                    }
                }
                
                if loading() {
                    div {
                        class: "loading-container",
                        div { class: "loading-spinner" }
                        div { class: "loading-text", "Loading games..." }
                    }
                }
                
                if let Some(err) = error() {
                    div {
                        class: "error-message",
                        span { class: "error-icon", "⚠️" }
                        span { "Error: {err}" }
                    }
                }
                
                if !loading() && error().is_none() {
                    GameList { games: games }
                }
            }
        }
    }
}

#[component]
fn SearchBar(search_query: Signal<String>, on_search: EventHandler<String>) -> Element {
    rsx! {
        div {
            style: "margin-bottom: 3rem; display: flex; gap: 1rem; align-items: stretch;",
            
            div {
                class: "search-container",
                style: "flex: 1; position: relative;",
                
                div {
                    style: "position: absolute; left: 1rem; top: 50%; transform: translateY(-50%); color: #999; font-size: 1.25rem; z-index: 1;",
                    "🔍"
                }
                
                input {
                    r#type: "text",
                    placeholder: "Search for games by name, platform, or genre...",
                    value: "{search_query()}",
                    oninput: move |evt| search_query.set(evt.value()),
                    onkeydown: move |evt| {
                        if evt.key() == dioxus::prelude::Key::Enter {
                            on_search.call(search_query());
                        }
                    },
                    class: "search-input",
                }
            }
            
            button {
                onclick: move |_| on_search.call(search_query()),
                class: "search-button",
                "Search"
            }
        }
    }
}

#[component]
fn GameList(games: Signal<Vec<GameData>>) -> Element {
    let games_vec = games();
    
    if games_vec.is_empty() {
        return rsx! {
            div {
                class: "empty-state",
                div { class: "empty-state-icon", "🎮" }
                div { class: "empty-state-text", "No games found" }
            }
        };
    }
    
    rsx! {
        div {
            style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 2rem;",
            
            for game in games_vec.iter() {
                GameCard { game: game.clone() }
            }
        }
    }
}

#[component]
fn GameCard(game: GameData) -> Element {
    let release_date = if !game.first_release_date.is_empty() && game.first_release_date != "0" {
        match game.first_release_date.parse::<i64>() {
            Ok(timestamp) => {
                if let Some(dt) = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp, 0) {
                    Some(dt.format("%B %Y").to_string())
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };
    
    rsx! {
        div {
            class: "game-card",
            
            h2 {
                class: "game-title",
                "{game.name}"
            }
            
            if let Some(date) = release_date {
                div {
                    style: "margin-top: auto;",
                    span {
                        class: "badge badge-release",
                        "📅 {date}"
                    }
                }
            }
            
            if !game.platforms.is_empty() {
                div {
                    style: "margin-top: 0.5rem;",
                    div {
                        class: "platform-tags",
                        for platform in game.platforms.iter() {
                            span {
                                class: "platform-tag",
                                "{platform}"
                            }
                        }
                    }
                }
            }
            
            if !game.genres.is_empty() {
                div {
                    style: "margin-top: 0.75rem;",
                    div {
                        class: "genre-tags",
                        for genre in game.genres.iter() {
                            span {
                                class: "genre-tag",
                                "{genre}"
                            }
                        }
                    }
                }
            }
        }
    }
}
