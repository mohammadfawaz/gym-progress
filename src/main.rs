use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

const URL: &str = "https://zhlsfzjhlnxztjklhmpi.supabase.co";
const KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InpobHNmempobG54enRqa2xobXBpIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODU0MTE1NjAsImV4cCI6MjEwMDk4NzU2MH0.5E-algHiQRS8dD18r0blom86gU88nFShahk6cAMnpqI";
const LOCAL: &str = "lift-log-v1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Exercise { name: String, weight: Option<f64>, unit: String, reps: String, details: String }
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Workout { id: String, date: String, note: String, exercises: Vec<Exercise> }
#[derive(Deserialize)] struct Auth { access_token: String, user: User }
#[derive(Deserialize)] struct User { id: String }
#[derive(Deserialize)] struct DbWorkout { id: String, workout_date: String, note: String, exercises: Vec<Exercise> }

fn headers(req: gloo_net::http::RequestBuilder, token: Option<&str>) -> gloo_net::http::RequestBuilder {
    let req = req.header("apikey", KEY);
    match token { Some(t) => req.header("Authorization", &format!("Bearer {t}")), None => req }
}
async fn auth(email: &str, password: &str, signup: bool) -> Result<Auth, String> {
    let path = if signup { "signup" } else { "token?grant_type=password" };
    let req = headers(Request::post(&format!("{URL}/auth/v1/{path}")), None).header("Content-Type", "application/json");
    let res = req.body(serde_json::json!({"email":email,"password":password}).to_string()).map_err(|e| e.to_string())?.send().await.map_err(|e| e.to_string())?;
    if !res.ok() { return Err(res.text().await.unwrap_or_else(|_| "Authentication failed".into())); }
    res.json().await.map_err(|e| e.to_string())
}
async fn get_workouts(token: &str, uid: &str) -> Result<Vec<Workout>, String> {
    let url = format!("{URL}/rest/v1/workouts?select=id,workout_date,note,exercises&user_id=eq.{uid}&order=workout_date.desc");
    let res = headers(Request::get(&url), Some(token)).send().await.map_err(|e| e.to_string())?;
    if !res.ok() { return Err(res.text().await.unwrap_or_else(|_| "Could not load workouts".into())); }
    let rows: Vec<DbWorkout> = res.json().await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| Workout { id:r.id, date:r.workout_date, note:r.note, exercises:r.exercises }).collect())
}
async fn put_workout(token: &str, uid: &str, w: &Workout) -> Result<(), String> {
    let req = headers(Request::post(&format!("{URL}/rest/v1/workouts?on_conflict=user_id,id")), Some(token)).header("Content-Type", "application/json").header("Prefer", "resolution=merge-duplicates");
    let body = serde_json::json!({"id":w.id,"user_id":uid,"workout_date":w.date,"note":w.note,"exercises":w.exercises});
    let res = req.body(body.to_string()).map_err(|e| e.to_string())?.send().await.map_err(|e| e.to_string())?;
    if res.ok() { Ok(()) } else { Err(res.text().await.unwrap_or_else(|_| "Could not save workout".into())) }
}
fn input_value(e: InputEvent) -> String { e.target_unchecked_into::<HtmlInputElement>().value() }
fn previous(workouts: &[Workout], name: &str) -> Option<Exercise> { workouts.iter().flat_map(|w| w.exercises.iter()).find(|e| e.name.eq_ignore_ascii_case(name)).cloned() }
fn workout_view(w: &Workout) -> Html { html! { <article class="workout-card"><div class="workout-card-top"><div><time>{w.date.clone()}</time><h3>{if w.note.is_empty(){format!("{} exercises",w.exercises.len())}else{w.note.clone()}}</h3></div><span class="pill">{format!("{} lifts",w.exercises.len())}</span></div>{ for w.exercises.iter().map(|e| html! { <p class="exercise-summary">{format!("{} · {}{} · {}", e.name, e.weight.map(|v|v.to_string()).unwrap_or_else(||"Bodyweight".into()), if e.weight.is_some(){format!(" {}",e.unit)}else{String::new()}, e.reps)}</p> }) }</article> } }
fn draft_view((i,e): (usize, &Exercise)) -> Html { html! { <article class="exercise-entry"><div class="entry-head"><h3>{format!("{}. {}",i+1,e.name)}</h3><span class="pill">{format!("{} {}",e.weight.map(|v|v.to_string()).unwrap_or_else(||"Bodyweight".into()),e.unit)}</span></div><p class="exercise-summary">{format!("{} reps{}",e.reps,if e.details.is_empty(){String::new()}else{format!(" · {}",e.details)})}</p></article> } }

#[function_component(App)]
fn app() -> Html {
    let token = use_state(|| None::<String>); let uid = use_state(|| None::<String>); let workouts = use_state(Vec::<Workout>::new); let status = use_state(String::new);
    let email = use_state(String::new); let password = use_state(String::new); let signup = use_state(|| false);
    let date = use_state(|| "2026-07-30".to_string()); let note = use_state(String::new); let name = use_state(String::new); let weight = use_state(String::new); let unit = use_state(|| "lb".to_string()); let reps = use_state(String::new); let details = use_state(String::new); let draft = use_state(Vec::<Exercise>::new);
    let submit_auth = { let email=email.clone(); let password=password.clone(); let signup=signup.clone(); let token=token.clone(); let uid=uid.clone(); let workouts=workouts.clone(); let status=status.clone(); Callback::from(move |e:SubmitEvent| { e.prevent_default(); let email=(*email).clone(); let password=(*password).clone(); let is_signup=*signup; let token=token.clone(); let uid=uid.clone(); let workouts=workouts.clone(); let status=status.clone(); spawn_local(async move { status.set("Connecting…".into()); match auth(&email,&password,is_signup).await { Ok(a)=>{ let remote=get_workouts(&a.access_token,&a.user.id).await.unwrap_or_default(); let local=LocalStorage::get::<Vec<Workout>>(LOCAL).unwrap_or_default(); if remote.is_empty() && !local.is_empty() { for w in &local { let _=put_workout(&a.access_token,&a.user.id,w).await; } workouts.set(local); status.set("Existing workouts migrated and synced.".into()); } else { workouts.set(remote); status.set("Synced with Supabase.".into()); } token.set(Some(a.access_token)); uid.set(Some(a.user.id)); } Err(e)=>status.set(e), } }); }) };
    let add = { let name=name.clone(); let weight=weight.clone(); let unit=unit.clone(); let reps=reps.clone(); let details=details.clone(); let draft=draft.clone(); let status=status.clone(); Callback::from(move |_| { if name.trim().is_empty()||reps.trim().is_empty(){status.set("Add an exercise name and reps first.".into());return} let mut next=(*draft).clone(); next.push(Exercise{name:(*name).trim().into(),weight:weight.parse().ok(),unit:(*unit).clone(),reps:(*reps).trim().into(),details:(*details).trim().into()}); draft.set(next); name.set(String::new());weight.set(String::new());reps.set(String::new());details.set(String::new()); }) };
    let save = { let token=token.clone(); let uid=uid.clone(); let workouts=workouts.clone(); let date=date.clone(); let note=note.clone(); let draft=draft.clone(); let status=status.clone(); Callback::from(move |_| { if draft.is_empty(){status.set("Add at least one exercise.".into());return} let w=Workout{id:format!("local-{}",js_sys::Date::now() as u64),date:(*date).clone(),note:(*note).clone(),exercises:(*draft).clone()}; let token=(*token).clone();let uid=(*uid).clone();let workouts=workouts.clone();let status=status.clone();spawn_local(async move {if let(Some(t),Some(u))=(token,uid){match put_workout(&t,&u,&w).await{Ok(())=>{let mut all=(*workouts).clone();all.insert(0,w);workouts.set(all);status.set("Workout saved to Supabase.".into())},Err(e)=>status.set(e)}}}); }) };
    if token.is_none() { return html! { <main class="app-shell"><div class="hero-card"><p class="eyebrow">{"PERSONAL TRAINING LOG"}</p><h1>{"Lift Log"}</h1><h2>{"Train with your history."}</h2><p>{"Sign in to sync across devices."}</p></div><form class="auth-card" onsubmit={submit_auth}><label class="field-label">{"Email"}<input type="email" value={(*email).clone()} oninput={let email=email.clone();Callback::from(move|e:InputEvent|email.set(input_value(e)))} required=true /></label><label class="field-label">{"Password"}<input type="password" value={(*password).clone()} oninput={let password=password.clone();Callback::from(move|e:InputEvent|password.set(input_value(e)))} required=true minlength="6" /></label><button class="primary-button" type="submit">{if *signup{"Create account"}else{"Sign in"}}<span>{"→"}</span></button></form><button class="text-button auth-toggle" onclick={let signup=signup.clone();Callback::from(move |_|signup.set(!*signup))}>{if *signup{"Already have an account? Sign in"}else{"Need an account? Create one"}}</button><p class="subtle">{(*status).clone()}</p></main> }; }
    let on_name = { let name=name.clone();let weight=weight.clone();let reps=reps.clone();let workouts=workouts.clone();Callback::from(move|e:InputEvent|{let v=input_value(e);name.set(v.clone());if let Some(p)=previous(&workouts,&v){weight.set(p.weight.map(|x|x.to_string()).unwrap_or_default());reps.set(p.reps)}}) };
    html! { <main class="app-shell"><header class="topbar"><div><p class="eyebrow">{"PERSONAL TRAINING LOG"}</p><h1>{"Lift Log"}</h1></div><span class="pill">{"SYNCED"}</span></header><section class="hero-card"><p class="eyebrow">{"NEW WORKOUT"}</p><h2>{"Keep the streak moving."}</h2><p>{"Your previous numbers prefill as targets."}</p></section><section class="workout-form"><label class="field-label">{"Date"}<input type="date" value={(*date).clone()} oninput={let date=date.clone();Callback::from(move|e:InputEvent|date.set(input_value(e)))}/></label><label class="field-label">{"Session note"}<input value={(*note).clone()} oninput={let note=note.clone();Callback::from(move|e:InputEvent|note.set(input_value(e)))}/></label><div class="exercise-entry"><label class="field-label">{"Exercise"}<input value={(*name).clone()} oninput={on_name} placeholder="Bench press" /></label><div class="numbers-row"><label class="field-label">{"Weight"}<input value={(*weight).clone()} oninput={let weight=weight.clone();Callback::from(move|e:InputEvent|weight.set(input_value(e)))}/></label><label class="field-label">{"Unit"}<select value={(*unit).clone()} onchange={let unit=unit.clone();Callback::from(move|e:Event|unit.set(e.target_unchecked_into::<HtmlSelectElement>().value()))}><option value="lb">{"lb"}</option><option value="kg">{"kg"}</option><option value="bodyweight">{"bodyweight"}</option></select></label></div><label class="field-label">{"Reps per set"}<input value={(*reps).clone()} oninput={let reps=reps.clone();Callback::from(move|e:InputEvent|reps.set(input_value(e)))} placeholder="8, 8, 6" /></label><label class="field-label">{"Details"}<input value={(*details).clone()} oninput={let details=details.clone();Callback::from(move|e:InputEvent|details.set(input_value(e)))} /></label><button class="add-button" type="button" onclick={add}>{"+ Add exercise"}</button></div>{for draft.iter().enumerate().map(draft_view)}<button class="primary-button save-button" type="button" onclick={save}>{"Save workout"}<span>{"✓"}</span></button><p class="subtle">{(*status).clone()}</p></section><div class="section-heading"><div><p class="eyebrow">{"YOUR LOG"}</p><h2>{"Workout history"}</h2></div></div><div class="workout-list">{for workouts.iter().map(workout_view)}</div></main> }
}
fn main() { yew::Renderer::<App>::new().render(); }
