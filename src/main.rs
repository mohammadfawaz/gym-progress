use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

const URL: &str = "https://zhlsfzjhlnxztjklhmpi.supabase.co";
const KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InpobHNmempobG54enRqa2xobXBpIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODU0MTE1NjAsImV4cCI6MjEwMDk4NzU2MH0.5E-algHiQRS8dD18r0blom86gU88nFShahk6cAMnpqI";
const LOCAL: &str = "lift-log-v1";
const AUTH_TOKEN_KEY: &str = "lift-log-auth-token";
const AUTH_UID_KEY: &str = "lift-log-auth-uid";
const ADD_EXERCISE_VALUE: &str = "__add_exercise__";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Exercise {
    name: String,
    weight: Option<f64>,
    reps: String,
    details: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Workout {
    id: String,
    date: String,
    note: String,
    exercises: Vec<Exercise>,
}
#[derive(Deserialize)]
struct Auth {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    session: Option<AuthSession>,
    #[serde(default)]
    user: Option<User>,
}
#[derive(Deserialize)]
struct AuthSession {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    user: Option<User>,
}
#[derive(Deserialize)]
struct User {
    id: String,
}
#[derive(Deserialize)]
struct DbWorkout {
    id: String,
    workout_date: String,
    note: String,
    exercises: Vec<Exercise>,
}
#[derive(Deserialize)]
struct DbExerciseCatalog {
    canonical_name: String,
    #[serde(default)]
    aliases: Vec<String>,
}

fn headers(
    req: gloo_net::http::RequestBuilder,
    token: Option<&str>,
) -> gloo_net::http::RequestBuilder {
    let req = req.header("apikey", KEY);
    match token {
        Some(t) => req.header("Authorization", &format!("Bearer {t}")),
        None => req,
    }
}
async fn auth(email: &str, password: &str, signup: bool) -> Result<Auth, String> {
    let path = if signup {
        "signup"
    } else {
        "token?grant_type=password"
    };
    let req = headers(Request::post(&format!("{URL}/auth/v1/{path}")), None)
        .header("Content-Type", "application/json");
    let res = req
        .body(serde_json::json!({"email":email,"password":password}).to_string())
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.ok() {
        return Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Authentication failed".into()));
    }
    res.json().await.map_err(|e| e.to_string())
}
fn auth_token(auth: &Auth) -> Option<String> {
    auth.access_token.clone().or_else(|| {
        auth.session
            .as_ref()
            .and_then(|session| session.access_token.clone())
    })
}
fn auth_user_id(auth: &Auth) -> Option<String> {
    auth.user.as_ref().map(|user| user.id.clone()).or_else(|| {
        auth.session
            .as_ref()
            .and_then(|session| session.user.as_ref().map(|user| user.id.clone()))
    })
}
async fn get_workouts(token: &str, uid: &str) -> Result<Vec<Workout>, String> {
    let url = format!("{URL}/rest/v1/workouts?select=id,workout_date,note,exercises&user_id=eq.{uid}&order=workout_date.desc");
    let res = headers(Request::get(&url), Some(token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.ok() {
        return Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Could not load workouts".into()));
    }
    let rows: Vec<DbWorkout> = res.json().await.map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|r| Workout {
            id: r.id,
            date: r.workout_date,
            note: r.note,
            exercises: r.exercises,
        })
        .collect())
}
async fn get_exercise_catalog(token: &str) -> Result<Vec<DbExerciseCatalog>, String> {
    let url = format!(
        "{URL}/rest/v1/exercise_catalog?select=canonical_name,aliases&order=sort_order.asc,canonical_name.asc"
    );
    let res = headers(Request::get(&url), Some(token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.ok() {
        return Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Could not load exercise catalog".into()));
    }
    res.json().await.map_err(|e| e.to_string())
}
async fn put_workout(token: &str, uid: &str, w: &Workout) -> Result<(), String> {
    let req = headers(
        Request::post(&format!("{URL}/rest/v1/workouts?on_conflict=user_id,id")),
        Some(token),
    )
    .header("Content-Type", "application/json")
    .header("Prefer", "resolution=merge-duplicates");
    let body = serde_json::json!({"id":w.id,"user_id":uid,"workout_date":w.date,"note":w.note,"exercises":w.exercises});
    let res = req
        .body(body.to_string())
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if res.ok() {
        Ok(())
    } else {
        Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Could not save workout".into()))
    }
}
async fn put_exercise_catalog(token: &str, name: &str) -> Result<(), String> {
    let req = headers(
        Request::post(&format!(
            "{URL}/rest/v1/exercise_catalog?on_conflict=canonical_name"
        )),
        Some(token),
    )
    .header("Content-Type", "application/json")
    .header("Prefer", "resolution=merge-duplicates");
    let body = serde_json::json!({
        "canonical_name": name,
        "aliases": [],
        "sort_order": 9999
    });
    let res = req
        .body(body.to_string())
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if res.ok() {
        Ok(())
    } else {
        Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Could not add exercise".into()))
    }
}
async fn delete_workout(token: &str, uid: &str, id: &str) -> Result<(), String> {
    let url = format!("{URL}/rest/v1/workouts?id=eq.{id}&user_id=eq.{uid}");
    let res = headers(Request::delete(&url), Some(token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if res.ok() {
        Ok(())
    } else {
        Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Could not delete workout".into()))
    }
}
fn today_string() -> String {
    let iso: String = js_sys::Date::new_0().to_iso_string().into();
    iso[..10].to_string()
}
fn stored_auth() -> (Option<String>, Option<String>) {
    (
        LocalStorage::get(AUTH_TOKEN_KEY).ok(),
        LocalStorage::get(AUTH_UID_KEY).ok(),
    )
}
fn clear_auth() {
    let _ = LocalStorage::delete(AUTH_TOKEN_KEY);
    let _ = LocalStorage::delete(AUTH_UID_KEY);
}
fn input_value(e: InputEvent) -> String {
    e.target_unchecked_into::<HtmlInputElement>().value()
}
fn previous(workouts: &[Workout], name: &str) -> Option<Exercise> {
    workouts
        .iter()
        .flat_map(|w| w.exercises.iter())
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .cloned()
}
fn canonicalize_name(name: &str, aliases: &HashMap<String, String>) -> String {
    aliases
        .get(&name.trim().to_ascii_lowercase())
        .cloned()
        .unwrap_or_else(|| name.trim().to_string())
}
fn title_case_name(name: &str) -> String {
    name.split_whitespace()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn build_catalog(rows: &[DbExerciseCatalog]) -> (Vec<String>, HashMap<String, String>) {
    let mut options = Vec::new();
    let mut aliases = HashMap::new();
    for row in rows {
        options.push(row.canonical_name.clone());
        aliases.insert(
            row.canonical_name.to_ascii_lowercase(),
            row.canonical_name.clone(),
        );
        for alias in &row.aliases {
            aliases.insert(alias.to_ascii_lowercase(), row.canonical_name.clone());
        }
    }
    (options, aliases)
}
async fn sync_user_data(
    access_token: &str,
    user_id: &str,
) -> Result<(Vec<Workout>, Vec<String>, HashMap<String, String>), String> {
    let catalog_rows = get_exercise_catalog(access_token).await.unwrap_or_default();
    let (catalog_names, alias_map) = build_catalog(&catalog_rows);
    let remote = canonicalize_workouts(
        get_workouts(access_token, user_id)
            .await
            .unwrap_or_default(),
        &alias_map,
    );
    let local = canonicalize_workouts(
        LocalStorage::get::<Vec<Workout>>(LOCAL).unwrap_or_default(),
        &alias_map,
    );
    let merged = merge_workouts(remote, local);
    for w in &merged {
        let _ = put_workout(access_token, user_id, w).await;
    }
    Ok((merged, catalog_names, alias_map))
}
fn canonicalize_workout(w: &Workout, aliases: &HashMap<String, String>) -> Workout {
    let mut next = w.clone();
    next.exercises = next
        .exercises
        .into_iter()
        .map(|mut e| {
            e.name = canonicalize_name(&e.name, aliases);
            e
        })
        .collect();
    next
}
fn canonicalize_workouts(
    workouts: Vec<Workout>,
    aliases: &HashMap<String, String>,
) -> Vec<Workout> {
    workouts
        .into_iter()
        .map(|w| canonicalize_workout(&w, aliases))
        .collect()
}
fn exercise_names_from_workouts(workouts: &[Workout]) -> Vec<String> {
    let mut names = BTreeSet::new();
    for workout in workouts {
        for exercise in &workout.exercises {
            names.insert(exercise.name.clone());
        }
    }
    names.into_iter().collect()
}
fn workout_key(w: &Workout) -> String {
    serde_json::to_string(&serde_json::json!({
        "date": w.date,
        "note": w.note,
        "exercises": w.exercises,
    }))
    .unwrap_or_default()
}
fn merge_workouts(mut primary: Vec<Workout>, secondary: Vec<Workout>) -> Vec<Workout> {
    let mut seen = BTreeSet::new();
    let mut merged = Vec::new();
    for workout in primary.drain(..).chain(secondary.into_iter()) {
        let key = workout_key(&workout);
        if seen.insert(key) {
            merged.push(workout);
        }
    }
    merged.sort_by(|a, b| b.date.cmp(&a.date));
    merged
}
fn workout_view(
    w: &Workout,
    on_edit: Callback<MouseEvent>,
    on_delete: Callback<MouseEvent>,
) -> Html {
    html! {
        <article class="workout-card">
            <div class="workout-card-top">
                <div>
                    <time>{w.date.clone()}</time>
                    <h3>{if w.note.is_empty(){format!("{} exercises",w.exercises.len())}else{w.note.clone()}}</h3>
                </div>
                <span class="pill">{format!("{} lifts",w.exercises.len())}</span>
            </div>
            <div class="workout-actions">
                <button class="text-button" type="button" onclick={on_edit}>{"Edit"}</button>
                <button class="text-button" type="button" onclick={on_delete}>{"Delete"}</button>
            </div>
            { for w.exercises.iter().map(|e| html! { <p class="exercise-summary">{format!("{} · {} · {}", e.name, e.weight.map(|v| format!("{} lbs", v)).unwrap_or_else(||"BW".into()), e.reps)}</p> }) }
        </article>
    }
}
fn draft_view((i, e, on_remove): (usize, &Exercise, Callback<MouseEvent>)) -> Html {
    html! {
        <article class="exercise-entry">
            <div class="entry-head">
                <h3>{format!("{}. {}", i + 1, e.name)}</h3>
                <button class="remove-exercise" type="button" onclick={on_remove}>{"×"}</button>
            </div>
            <span class="pill">{e.weight.map(|v| format!("{} lbs", v)).unwrap_or_else(||"BW".into())}</span>
            <p class="exercise-summary">{format!("{} reps{}", e.reps, if e.details.is_empty(){String::new()}else{format!(" · {}", e.details)})}</p>
        </article>
    }
}

fn auth_view(
    email: UseStateHandle<String>,
    password: UseStateHandle<String>,
    signup: UseStateHandle<bool>,
    status: UseStateHandle<String>,
    submit_auth: Callback<SubmitEvent>,
) -> Html {
    let on_email = {
        let email = email.clone();
        Callback::from(move |e: InputEvent| email.set(input_value(e)))
    };
    let on_password = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| password.set(input_value(e)))
    };
    let toggle_signup = {
        let signup = signup.clone();
        Callback::from(move |_| signup.set(!*signup))
    };

    html! {
        <main class="app-shell">
            <div class="hero-card">
                <p class="eyebrow">{"PERSONAL TRAINING LOG"}</p>
                <h1>{"Lift Log"}</h1>
                <p>{"Sign in to sync across devices."}</p>
            </div>
            <form class="auth-card" onsubmit={submit_auth}>
                <label class="field-label">
                    {"Email"}
                    <input type="email" value={(*email).clone()} oninput={on_email} required=true />
                </label>
                <label class="field-label">
                    {"Password"}
                    <input type="password" value={(*password).clone()} oninput={on_password} required=true minlength="6" />
                </label>
                <button class="primary-button" type="submit">
                    {if *signup { "Create account" } else { "Sign in" }}
                    <span>{"→"}</span>
                </button>
            </form>
            <button class="text-button auth-toggle" onclick={toggle_signup}>
                {if *signup {
                    "Already have an account? Sign in"
                } else {
                    "Need an account? Create one"
                }}
            </button>
            <p class="subtle">{(*status).clone()}</p>
        </main>
    }
}

fn draft_entries(draft: &[Exercise], remove_draft: Callback<usize>) -> Html {
    html! {
        <>
            {for draft.iter().enumerate().map(|(i, e)| {
                let on_remove = {
                    let remove_draft = remove_draft.clone();
                    Callback::from(move |_| remove_draft.emit(i))
                };
                draft_view((i, e, on_remove))
            })}
        </>
    }
}

fn history_view(
    workouts: &[Workout],
    load_workout: Callback<Workout>,
    delete_selected: Callback<String>,
) -> Html {
    html! {
        <div class="workout-list">
            {for workouts.iter().map(|w| {
                let on_edit = {
                    let load_workout = load_workout.clone();
                    let w = w.clone();
                    Callback::from(move |_| load_workout.emit(w.clone()))
                };
                let on_delete = {
                    let delete_selected = delete_selected.clone();
                    let id = w.id.clone();
                    Callback::from(move |_| delete_selected.emit(id.clone()))
                };
                workout_view(w, on_edit, on_delete)
            })}
        </div>
    }
}

struct WorkoutEditorProps {
    date: UseStateHandle<String>,
    note: UseStateHandle<String>,
    name: UseStateHandle<String>,
    weight: UseStateHandle<String>,
    reps: UseStateHandle<String>,
    details: UseStateHandle<String>,
    draft: UseStateHandle<Vec<Exercise>>,
    workouts: UseStateHandle<Vec<Workout>>,
    editing_id: UseStateHandle<Option<String>>,
    status: UseStateHandle<String>,
    new_exercise_name: UseStateHandle<String>,
    on_name: Callback<Event>,
    add: Callback<MouseEvent>,
    remove_draft: Callback<usize>,
    save: Callback<MouseEvent>,
    logout: Callback<MouseEvent>,
    load_workout: Callback<Workout>,
    delete_selected: Callback<String>,
    exercise_options: Vec<String>,
}

fn workout_editor_view(props: WorkoutEditorProps) -> Html {
    let WorkoutEditorProps {
        date,
        note,
        name,
        weight,
        reps,
        details,
        draft,
        workouts,
        editing_id,
        status,
        new_exercise_name,
        on_name,
        add,
        remove_draft,
        save,
        logout,
        load_workout,
        delete_selected,
        exercise_options,
    } = props;

    html! {
        <main class="app-shell">
            <header class="topbar">
                <div>
                    <p class="eyebrow">{"PERSONAL TRAINING LOG"}</p>
                    <h1>{"Lift Log"}</h1>
                </div>
                <button class="text-button" type="button" onclick={logout}>{"Logout"}</button>
            </header>
            <section class="hero-card">
                <p class="eyebrow">{"NEW WORKOUT"}</p>
                <p>{"Your previous numbers prefill as targets."}</p>
            </section>
            <section class="workout-form">
                <label class="field-label">
                    {"Date"}
                    <input
                        type="date"
                        value={(*date).clone()}
                        oninput={{
                            let date = date.clone();
                            Callback::from(move |e: InputEvent| date.set(input_value(e)))
                        }}
                    />
                </label>
                <label class="field-label">
                    {"Session note"}
                    <input
                        value={(*note).clone()}
                        oninput={{
                            let note = note.clone();
                            Callback::from(move |e: InputEvent| note.set(input_value(e)))
                        }}
                    />
                </label>
                <div class="exercise-stack">
                    <label class="field-label">
                        {"Exercise"}
                        <select value={(*name).clone()} onchange={on_name}>
                            {for exercise_options.iter().map(|exercise| html!{ <option value={exercise.clone()}>{exercise.clone()}</option> })}
                            <option value={ADD_EXERCISE_VALUE}>{"New Exercise"}</option>
                        </select>
                    </label>
                    {if *name == ADD_EXERCISE_VALUE {
                        html! {
                            <label class="field-label">
                                {"New exercise name"}
                                <input
                                    value={(*new_exercise_name).clone()}
                                    oninput={{
                                        let new_exercise_name = new_exercise_name.clone();
                                        Callback::from(move |e: InputEvent| new_exercise_name.set(input_value(e)))
                                    }}
                                    placeholder="Romanian deadlift"
                                />
                            </label>
                        }
                    } else {
                        html! {}
                    }}
                    <label class="field-label">
                        {"Weight"}
                        <input
                            value={(*weight).clone()}
                            oninput={{
                                let weight = weight.clone();
                                Callback::from(move |e: InputEvent| weight.set(input_value(e)))
                            }}
                        />
                    </label>
                    <label class="field-label">
                        {"Reps per set"}
                        <input
                            value={(*reps).clone()}
                            oninput={{
                                let reps = reps.clone();
                                Callback::from(move |e: InputEvent| reps.set(input_value(e)))
                            }}
                            placeholder="8, 8, 6"
                        />
                    </label>
                    <label class="field-label">
                        {"Details"}
                        <input
                            value={(*details).clone()}
                            oninput={{
                                let details = details.clone();
                                Callback::from(move |e: InputEvent| details.set(input_value(e)))
                            }}
                        />
                    </label>
                    <button class="add-button" type="button" onclick={add}>{"+ Add exercise"}</button>
                </div>
                {draft_entries(&draft, remove_draft)}
                <button class="primary-button save-button" type="button" onclick={save}>
                    {if editing_id.is_some() { "Update workout" } else { "Save workout" }}
                    <span>{"✓"}</span>
                </button>
                <p class="subtle">{(*status).clone()}</p>
            </section>
            <div class="section-heading">
                <div>
                    <p class="eyebrow">{"YOUR LOG"}</p>
                    <h2>{"Workout history"}</h2>
                </div>
            </div>
            {history_view(&workouts, load_workout, delete_selected)}
        </main>
    }
}

#[function_component(App)]
fn app() -> Html {
    let (stored_token, stored_uid) = stored_auth();
    let token = use_state(|| stored_token);
    let uid = use_state(|| stored_uid);
    let workouts = use_state(Vec::<Workout>::new);
    let exercise_catalog = use_state(Vec::<String>::new);
    let exercise_aliases = use_state(HashMap::<String, String>::new);
    let status = use_state(String::new);
    let email = use_state(String::new);
    let password = use_state(String::new);
    let signup = use_state(|| false);
    let date = use_state(|| "2026-07-30".to_string());
    let note = use_state(String::new);
    let name = use_state(String::new);
    let new_exercise_name = use_state(String::new);
    let weight = use_state(String::new);
    let reps = use_state(String::new);
    let details = use_state(String::new);
    let draft = use_state(Vec::<Exercise>::new);
    let editing_id = use_state(|| None::<String>);

    {
        let token = token.clone();
        let uid = uid.clone();
        let workouts = workouts.clone();
        let exercise_catalog = exercise_catalog.clone();
        let exercise_aliases = exercise_aliases.clone();
        let status = status.clone();
        use_effect_with((), move |_| {
            if let (Some(access_token), Some(user_id)) = ((*token).clone(), (*uid).clone()) {
                let token = token.clone();
                let uid = uid.clone();
                let workouts = workouts.clone();
                let exercise_catalog = exercise_catalog.clone();
                let exercise_aliases = exercise_aliases.clone();
                let status = status.clone();
                spawn_local(async move {
                    match sync_user_data(&access_token, &user_id).await {
                        Ok((merged, catalog_names, alias_map)) => {
                            workouts.set(merged);
                            exercise_catalog.set(catalog_names);
                            exercise_aliases.set(alias_map);
                            status.set(String::new());
                        }
                        Err(e) => {
                            clear_auth();
                            token.set(None);
                            uid.set(None);
                            status.set(e);
                        }
                    }
                });
            }
            || ()
        });
    }

    let load_workout = {
        let date = date.clone();
        let note = note.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let name = name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let details = details.clone();
        let new_exercise_name = new_exercise_name.clone();
        Callback::from(move |w: Workout| {
            date.set(w.date.clone());
            note.set(w.note.clone());
            draft.set(w.exercises.clone());
            editing_id.set(Some(w.id));
            name.set(String::new());
            new_exercise_name.set(String::new());
            weight.set(String::new());
            reps.set(String::new());
            details.set(String::new());
        })
    };
    let submit_auth = {
        let email = email.clone();
        let password = password.clone();
        let signup = signup.clone();
        let token = token.clone();
        let uid = uid.clone();
        let workouts = workouts.clone();
        let exercise_catalog = exercise_catalog.clone();
        let exercise_aliases = exercise_aliases.clone();
        let status = status.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let email = (*email).clone();
            let password = (*password).clone();
            let is_signup = *signup;
            let token = token.clone();
            let uid = uid.clone();
            let workouts = workouts.clone();
            let exercise_catalog = exercise_catalog.clone();
            let exercise_aliases = exercise_aliases.clone();
            let status = status.clone();
            spawn_local(async move {
                status.set("Connecting…".into());
                match auth(&email, &password, is_signup).await {
                    Ok(a) => {
                        let Some(access_token) = auth_token(&a) else {
                            if is_signup {
                                status.set(
                                    "Account created. Check your email to confirm it, then sign in."
                                        .into(),
                                );
                            } else {
                                status.set(
                                    "Sign-in succeeded but Supabase did not return a session token."
                                        .into(),
                                );
                            }
                            return;
                        };
                        let Some(user_id) = auth_user_id(&a) else {
                            status.set("Signed in, but Supabase did not return a user id.".into());
                            return;
                        };
                        let _ = LocalStorage::set(AUTH_TOKEN_KEY, access_token.clone());
                        let _ = LocalStorage::set(AUTH_UID_KEY, user_id.clone());
                        token.set(Some(access_token.clone()));
                        uid.set(Some(user_id.clone()));
                        let workouts = workouts.clone();
                        let exercise_catalog = exercise_catalog.clone();
                        let exercise_aliases = exercise_aliases.clone();
                        let status = status.clone();
                        spawn_local(async move {
                            match sync_user_data(&access_token, &user_id).await {
                                Ok((merged, catalog_names, alias_map)) => {
                                    workouts.set(merged);
                                    exercise_catalog.set(catalog_names);
                                    exercise_aliases.set(alias_map);
                                    status.set(String::new());
                                }
                                Err(e) => status.set(e),
                            }
                        });
                    }
                    Err(e) => status.set(e),
                }
            });
        })
    };
    let add = {
        let token = token.clone();
        let name = name.clone();
        let new_exercise_name = new_exercise_name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let details = details.clone();
        let draft = draft.clone();
        let status = status.clone();
        let exercise_catalog = exercise_catalog.clone();
        let exercise_aliases = exercise_aliases.clone();
        Callback::from(move |_| {
            if reps.trim().is_empty() {
                status.set("Enter reps first.".into());
                return;
            }

            let selected = (*name).clone();
            let weight_value = weight.parse().ok();
            let reps_value = (*reps).trim().to_string();
            let details_value = (*details).trim().to_string();

            if selected == ADD_EXERCISE_VALUE {
                let canonical = title_case_name(new_exercise_name.trim());
                if canonical.is_empty() {
                    status.set("Enter a new exercise name first.".into());
                    return;
                }

                let already_exists = exercise_catalog
                    .iter()
                    .any(|item| item.eq_ignore_ascii_case(&canonical));

                let token = (*token).clone();
                let draft = draft.clone();
                let status = status.clone();
                let exercise_catalog = exercise_catalog.clone();
                let exercise_aliases = exercise_aliases.clone();
                let name = name.clone();
                let new_exercise_name = new_exercise_name.clone();
                let weight_state = weight.clone();
                let reps_state = reps.clone();
                let details_state = details.clone();
                spawn_local(async move {
                    if let Some(t) = token {
                        if !already_exists {
                            if let Err(e) = put_exercise_catalog(&t, &canonical).await {
                                status.set(e);
                                return;
                            }
                            let mut next = (*exercise_catalog).clone();
                            next.push(canonical.clone());
                            next.sort();
                            next.dedup();
                            exercise_catalog.set(next);
                            let mut aliases = (*exercise_aliases).clone();
                            aliases.insert(canonical.to_ascii_lowercase(), canonical.clone());
                            exercise_aliases.set(aliases);
                        }
                        let mut next = (*draft).clone();
                        next.push(Exercise {
                            name: canonical,
                            weight: weight_value,
                            reps: reps_value,
                            details: details_value,
                        });
                        draft.set(next);
                        name.set(String::new());
                        new_exercise_name.set(String::new());
                        weight_state.set(String::new());
                        reps_state.set(String::new());
                        details_state.set(String::new());
                        status.set(String::new());
                    } else {
                        status.set("Sign in first.".into());
                    }
                });
                return;
            }

            if selected.trim().is_empty() {
                status.set("Choose an exercise first.".into());
                return;
            }

            let canonical_name = canonicalize_name(&selected, &exercise_aliases);
            if !exercise_catalog
                .iter()
                .any(|item| item.eq_ignore_ascii_case(&canonical_name))
            {
                status.set("Choose an exercise from the database list.".into());
                return;
            }
            let mut next = (*draft).clone();
            next.push(Exercise {
                name: canonical_name,
                weight: weight_value,
                reps: reps_value,
                details: details_value,
            });
            draft.set(next);
            name.set(String::new());
            new_exercise_name.set(String::new());
            weight.set(String::new());
            reps.set(String::new());
            details.set(String::new());
        })
    };
    let logout = {
        let token = token.clone();
        let uid = uid.clone();
        let workouts = workouts.clone();
        let exercise_catalog = exercise_catalog.clone();
        let exercise_aliases = exercise_aliases.clone();
        let status = status.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let name = name.clone();
        let new_exercise_name = new_exercise_name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let details = details.clone();
        Callback::from(move |_| {
            clear_auth();
            token.set(None);
            uid.set(None);
            workouts.set(Vec::new());
            exercise_catalog.set(Vec::new());
            exercise_aliases.set(HashMap::new());
            status.set(String::new());
            draft.set(Vec::new());
            editing_id.set(None);
            name.set(String::new());
            new_exercise_name.set(String::new());
            weight.set(String::new());
            reps.set(String::new());
            details.set(String::new());
        })
    };
    let remove_draft = {
        let draft = draft.clone();
        Callback::from(move |index: usize| {
            let mut next = (*draft).clone();
            if index < next.len() {
                next.remove(index);
                draft.set(next);
            }
        })
    };
    let reset_editor: Callback<()> = {
        let date = date.clone();
        let note = note.clone();
        let name = name.clone();
        let new_exercise_name = new_exercise_name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let details = details.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        Callback::from(move |_: ()| {
            date.set(today_string());
            note.set(String::new());
            name.set(String::new());
            new_exercise_name.set(String::new());
            weight.set(String::new());
            reps.set(String::new());
            details.set(String::new());
            draft.set(Vec::new());
            editing_id.set(None);
        })
    };
    let save = {
        let token = token.clone();
        let uid = uid.clone();
        let workouts = workouts.clone();
        let date = date.clone();
        let note = note.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let status = status.clone();
        let reset_editor = reset_editor.clone();
        Callback::from(move |_| {
            if draft.is_empty() {
                status.set("Add at least one exercise.".into());
                return;
            }
            let previous_id = (*editing_id).clone();
            let w = Workout {
                id: previous_id
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| format!("local-{}", js_sys::Date::now() as u64)),
                date: (*date).clone(),
                note: (*note).clone(),
                exercises: (*draft).clone(),
            };
            let token = (*token).clone();
            let uid = (*uid).clone();
            let workouts = workouts.clone();
            let status = status.clone();
            let reset_editor = reset_editor.clone();
            spawn_local(async move {
                if let (Some(t), Some(u)) = (token, uid) {
                    match put_workout(&t, &u, &w).await {
                        Ok(()) => {
                            let mut all = (*workouts).clone();
                            if let Some(id) = previous_id.as_ref() {
                                all.retain(|existing| existing.id != *id);
                            }
                            all.insert(0, w);
                            workouts.set(all);
                            reset_editor.emit(());
                            status.set("Workout saved to Supabase.".into())
                        }
                        Err(e) => status.set(e),
                    }
                }
            });
        })
    };
    let delete_selected = {
        let token = token.clone();
        let uid = uid.clone();
        let workouts = workouts.clone();
        let status = status.clone();
        Callback::from(move |id: String| {
            let token = (*token).clone();
            let uid = (*uid).clone();
            let workouts = workouts.clone();
            let status = status.clone();
            spawn_local(async move {
                if let (Some(t), Some(u)) = (token, uid) {
                    match delete_workout(&t, &u, &id).await {
                        Ok(()) => {
                            let mut next = (*workouts).clone();
                            next.retain(|w| w.id != id);
                            workouts.set(next);
                            status.set("Workout deleted.".into());
                        }
                        Err(e) => status.set(e),
                    }
                }
            });
        })
    };
    if token.is_none() {
        return auth_view(email, password, signup, status, submit_auth);
    }
    let on_name = {
        let name = name.clone();
        let new_exercise_name = new_exercise_name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let details = details.clone();
        let workouts = workouts.clone();
        Callback::from(move |e: Event| {
            let v = e.target_unchecked_into::<HtmlSelectElement>().value();
            name.set(v.clone());
            if v == ADD_EXERCISE_VALUE {
                new_exercise_name.set(String::new());
                weight.set(String::new());
                reps.set(String::new());
                details.set(String::new());
            } else if let Some(p) = previous(&workouts, &v) {
                weight.set(p.weight.map(|x| x.to_string()).unwrap_or_default());
                reps.set(p.reps);
                details.set(p.details);
            } else {
                weight.set(String::new());
                reps.set(String::new());
                details.set(String::new());
            }
        })
    };
    let exercise_options = {
        let mut options = (*exercise_catalog).clone();
        for name in exercise_names_from_workouts(&workouts) {
            if !options
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&name))
            {
                options.push(name);
            }
        }
        options.sort();
        options
    };
    workout_editor_view(WorkoutEditorProps {
        date,
        note,
        name,
        weight,
        reps,
        details,
        draft,
        workouts,
        editing_id,
        status,
        new_exercise_name,
        on_name,
        add,
        remove_draft,
        save,
        logout,
        load_workout,
        delete_selected,
        exercise_options,
    })
}
fn main() {
    yew::Renderer::<App>::new().render();
}
