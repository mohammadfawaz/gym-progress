use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

const URL: &str = "https://zhlsfzjhlnxztjklhmpi.supabase.co";
const KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InpobHNmempobG54enRqa2xobXBpIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODU0MTE1NjAsImV4cCI6MjEwMDk4NzU2MH0.5E-algHiQRS8dD18r0blom86gU88nFShahk6cAMnpqI";
const LOCAL: &str = "lift-log-v1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Exercise {
    name: String,
    weight: Option<f64>,
    unit: String,
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
fn exercise_names(workouts: &[Workout]) -> Vec<String> {
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
            { for w.exercises.iter().map(|e| html! { <p class="exercise-summary">{format!("{} · {}{} · {}", e.name, e.weight.map(|v|v.to_string()).unwrap_or_else(||"Bodyweight".into()), if e.weight.is_some(){format!(" {}",e.unit)}else{String::new()}, e.reps)}</p> }) }
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
            <span class="pill">{format!("{} {}", e.weight.map(|v|v.to_string()).unwrap_or_else(||"Bodyweight".into()), e.unit)}</span>
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
                <h2>{"Train with your history."}</h2>
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
    exercise_pick: UseStateHandle<String>,
    name: UseStateHandle<String>,
    weight: UseStateHandle<String>,
    unit: UseStateHandle<String>,
    reps: UseStateHandle<String>,
    details: UseStateHandle<String>,
    draft: UseStateHandle<Vec<Exercise>>,
    workouts: UseStateHandle<Vec<Workout>>,
    editing_id: UseStateHandle<Option<String>>,
    status: UseStateHandle<String>,
    on_name: Callback<InputEvent>,
    on_pick: Callback<Event>,
    add: Callback<MouseEvent>,
    remove_draft: Callback<usize>,
    save: Callback<MouseEvent>,
    load_workout: Callback<Workout>,
    delete_selected: Callback<String>,
    exercise_options: Vec<String>,
}

fn workout_editor_view(props: WorkoutEditorProps) -> Html {
    let WorkoutEditorProps {
        date,
        note,
        exercise_pick,
        name,
        weight,
        unit,
        reps,
        details,
        draft,
        workouts,
        editing_id,
        status,
        on_name,
        on_pick,
        add,
        remove_draft,
        save,
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
                <span class="pill">{"SYNCED"}</span>
            </header>
            <section class="hero-card">
                <p class="eyebrow">{"NEW WORKOUT"}</p>
                <h2>{"Keep the streak moving."}</h2>
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
                <label class="field-label">
                    {"Pick from history"}
                    <select value={(*exercise_pick).clone()} onchange={on_pick}>
                        <option value="">{"Choose a previous exercise"}</option>
                        {for exercise_options.iter().map(|name| html!{ <option value={name.clone()}>{name.clone()}</option> })}
                    </select>
                </label>
                <div class="exercise-entry">
                    <label class="field-label">
                        {"Exercise"}
                        <input value={(*name).clone()} oninput={on_name} placeholder="Bench press" />
                    </label>
                    <div class="numbers-row">
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
                            {"Unit"}
                            <select
                                value={(*unit).clone()}
                                onchange={{
                                    let unit = unit.clone();
                                    Callback::from(move |e: Event| unit.set(e.target_unchecked_into::<HtmlSelectElement>().value()))
                                }}
                            >
                                <option value="lb">{"lb"}</option>
                                <option value="kg">{"kg"}</option>
                                <option value="bodyweight">{"bodyweight"}</option>
                            </select>
                        </label>
                    </div>
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
    let token = use_state(|| None::<String>);
    let uid = use_state(|| None::<String>);
    let workouts = use_state(Vec::<Workout>::new);
    let status = use_state(String::new);
    let email = use_state(String::new);
    let password = use_state(String::new);
    let signup = use_state(|| false);
    let date = use_state(|| "2026-07-30".to_string());
    let note = use_state(String::new);
    let exercise_pick = use_state(String::new);
    let name = use_state(String::new);
    let weight = use_state(String::new);
    let unit = use_state(|| "lb".to_string());
    let reps = use_state(String::new);
    let details = use_state(String::new);
    let draft = use_state(Vec::<Exercise>::new);
    let editing_id = use_state(|| None::<String>);

    let load_workout = {
        let date = date.clone();
        let note = note.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let name = name.clone();
        let weight = weight.clone();
        let unit = unit.clone();
        let reps = reps.clone();
        let details = details.clone();
        let exercise_pick = exercise_pick.clone();
        Callback::from(move |w: Workout| {
            date.set(w.date.clone());
            note.set(w.note.clone());
            draft.set(w.exercises.clone());
            editing_id.set(Some(w.id));
            name.set(String::new());
            weight.set(String::new());
            unit.set("lb".into());
            reps.set(String::new());
            details.set(String::new());
            exercise_pick.set(String::new());
        })
    };
    let submit_auth = {
        let email = email.clone();
        let password = password.clone();
        let signup = signup.clone();
        let token = token.clone();
        let uid = uid.clone();
        let workouts = workouts.clone();
        let status = status.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let email = (*email).clone();
            let password = (*password).clone();
            let is_signup = *signup;
            let token = token.clone();
            let uid = uid.clone();
            let workouts = workouts.clone();
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
                        let remote = get_workouts(&access_token, &user_id)
                            .await
                            .unwrap_or_default();
                        let local = LocalStorage::get::<Vec<Workout>>(LOCAL).unwrap_or_default();
                        if !local.is_empty() {
                            let merged = merge_workouts(remote, local);
                            for w in &merged {
                                let _ = put_workout(&access_token, &user_id, w).await;
                            }
                            workouts.set(merged);
                            status.set("Synced with Supabase.".into());
                        } else {
                            workouts.set(remote);
                            status.set("Synced with Supabase.".into());
                        }
                        token.set(Some(access_token));
                        uid.set(Some(user_id));
                    }
                    Err(e) => status.set(e),
                }
            });
        })
    };
    let add = {
        let name = name.clone();
        let weight = weight.clone();
        let unit = unit.clone();
        let reps = reps.clone();
        let details = details.clone();
        let draft = draft.clone();
        let status = status.clone();
        Callback::from(move |_| {
            if name.trim().is_empty() || reps.trim().is_empty() {
                status.set("Add an exercise name and reps first.".into());
                return;
            }
            let mut next = (*draft).clone();
            next.push(Exercise {
                name: (*name).trim().into(),
                weight: weight.parse().ok(),
                unit: (*unit).clone(),
                reps: (*reps).trim().into(),
                details: (*details).trim().into(),
            });
            draft.set(next);
            name.set(String::new());
            weight.set(String::new());
            unit.set("lb".into());
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
        let exercise_pick = exercise_pick.clone();
        let name = name.clone();
        let weight = weight.clone();
        let unit = unit.clone();
        let reps = reps.clone();
        let details = details.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        Callback::from(move |_: ()| {
            date.set(today_string());
            note.set(String::new());
            exercise_pick.set(String::new());
            name.set(String::new());
            weight.set(String::new());
            unit.set("lb".into());
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
        let weight = weight.clone();
        let reps = reps.clone();
        let details = details.clone();
        let exercise_pick = exercise_pick.clone();
        let workouts = workouts.clone();
        Callback::from(move |e: InputEvent| {
            let v = input_value(e);
            exercise_pick.set(String::new());
            name.set(v.clone());
            if let Some(p) = previous(&workouts, &v) {
                weight.set(p.weight.map(|x| x.to_string()).unwrap_or_default());
                reps.set(p.reps);
                details.set(p.details);
            }
        })
    };
    let on_pick = {
        let exercise_pick = exercise_pick.clone();
        let name = name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let details = details.clone();
        let unit = unit.clone();
        let workouts = workouts.clone();
        Callback::from(move |e: Event| {
            let choice = e.target_unchecked_into::<HtmlSelectElement>().value();
            if choice.is_empty() {
                return;
            }
            name.set(choice.clone());
            if let Some(p) = previous((*workouts).as_slice(), &choice) {
                weight.set(p.weight.map(|x| x.to_string()).unwrap_or_default());
                reps.set(p.reps);
                details.set(p.details);
            }
            unit.set("lb".into());
            exercise_pick.set(String::new());
        })
    };
    let exercise_options = exercise_names(&workouts);
    workout_editor_view(WorkoutEditorProps {
        date,
        note,
        exercise_pick,
        name,
        weight,
        unit,
        reps,
        details,
        draft,
        workouts,
        editing_id,
        status,
        on_name,
        on_pick,
        add,
        remove_draft,
        save,
        load_workout,
        delete_selected,
        exercise_options,
    })
}
fn main() {
    yew::Renderer::<App>::new().render();
}
