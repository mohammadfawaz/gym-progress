use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use std::collections::{BTreeSet, HashMap};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

mod models;
mod sets;
use models::{
    Auth, DbExerciseCatalog, DbUserSettings, DbWorkout, DbWorkoutTemplate, Exercise, Workout,
    WorkoutTemplate,
};
use sets::{defaults as default_set_reps, format as format_set_reps, parse as parse_set_reps};

const URL: &str = "https://zhlsfzjhlnxztjklhmpi.supabase.co";
const KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InpobHNmempobG54enRqa2xobXBpIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODU0MTE1NjAsImV4cCI6MjEwMDk4NzU2MH0.5E-algHiQRS8dD18r0blom86gU88nFShahk6cAMnpqI";
const WORKOUT_CACHE_PREFIX: &str = "lift-log-workouts-v2";
const AUTH_TOKEN_KEY: &str = "lift-log-auth-token";
const AUTH_UID_KEY: &str = "lift-log-auth-uid";
const AUTH_REFRESH_KEY: &str = "lift-log-auth-refresh";
const THEME_KEY: &str = "lift-log-theme-v2";
const ADD_EXERCISE_VALUE: &str = "__add_exercise__";

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
fn auth_refresh_token(auth: &Auth) -> Option<String> {
    auth.refresh_token.clone().or_else(|| {
        auth.session
            .as_ref()
            .and_then(|session| session.refresh_token.clone())
    })
}
fn auth_user_id(auth: &Auth) -> Option<String> {
    auth.user.as_ref().map(|user| user.id.clone()).or_else(|| {
        auth.session
            .as_ref()
            .and_then(|session| session.user.as_ref().map(|user| user.id.clone()))
    })
}
fn stored_theme() -> String {
    match LocalStorage::get::<String>(THEME_KEY).ok().as_deref() {
        Some("dark") => "dark".into(),
        Some("light") => "light".into(),
        Some("pink") => "pink".into(),
        Some(_) | None => "dark".into(),
    }
}
fn persist_session(access_token: &str, user_id: &str, refresh_token: &str) {
    let _ = LocalStorage::set(AUTH_TOKEN_KEY, access_token.to_string());
    let _ = LocalStorage::set(AUTH_UID_KEY, user_id.to_string());
    let _ = LocalStorage::set(AUTH_REFRESH_KEY, refresh_token.to_string());
}
async fn refresh_session(refresh_token: &str) -> Result<Auth, String> {
    let req = headers(
        Request::post(&format!("{URL}/auth/v1/token?grant_type=refresh_token")),
        None,
    )
    .header("Content-Type", "application/json");
    let res = req
        .body(serde_json::json!({ "refresh_token": refresh_token }).to_string())
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.ok() {
        return Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Could not refresh session".into()));
    }
    res.json().await.map_err(|e| e.to_string())
}
async fn current_access_token(
    token: Option<String>,
    refresh_token: Option<String>,
) -> Result<(String, Option<String>), String> {
    if let Some(rt) = refresh_token {
        let auth = refresh_session(&rt).await?;
        let next_token =
            auth_token(&auth).ok_or_else(|| "Could not refresh session".to_string())?;
        let next_refresh = auth_refresh_token(&auth).or(Some(rt));
        Ok((next_token, next_refresh))
    } else if let Some(token) = token {
        Ok((token, None))
    } else {
        Err("Sign in first.".into())
    }
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
async fn get_workout_templates(token: &str, uid: &str) -> Result<Vec<WorkoutTemplate>, String> {
    let url = format!(
        "{URL}/rest/v1/workout_templates?select=id,name,note,exercises&user_id=eq.{uid}&order=name.asc"
    );
    let res = headers(Request::get(&url), Some(token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.ok() {
        return Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Could not load workout templates".into()));
    }
    let rows: Vec<DbWorkoutTemplate> = res.json().await.map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|row| WorkoutTemplate {
            id: row.id,
            name: row.name,
            note: row.note,
            exercises: row.exercises,
        })
        .collect())
}
async fn put_workout_template(
    token: &str,
    uid: &str,
    template: &WorkoutTemplate,
) -> Result<(), String> {
    let req = headers(
        Request::post(&format!("{URL}/rest/v1/workout_templates")),
        Some(token),
    )
    .header("Content-Type", "application/json");
    let body = serde_json::json!({
        "id": template.id,
        "user_id": uid,
        "name": template.name,
        "note": template.note,
        "exercises": template.exercises,
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
            .unwrap_or_else(|_| "Could not save workout template".into()))
    }
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
async fn get_user_theme(token: &str, uid: &str) -> Result<Option<String>, String> {
    let url = format!("{URL}/rest/v1/user_settings?select=theme&user_id=eq.{uid}&limit=1");
    let res = headers(Request::get(&url), Some(token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.ok() {
        return Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Could not load settings".into()));
    }
    let rows: Vec<DbUserSettings> = res.json().await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().next().map(|row| row.theme))
}
async fn put_user_theme(token: &str, uid: &str, theme: &str) -> Result<(), String> {
    let req = headers(
        Request::post(&format!("{URL}/rest/v1/user_settings?on_conflict=user_id")),
        Some(token),
    )
    .header("Content-Type", "application/json")
    .header("Prefer", "resolution=merge-duplicates");
    let body = serde_json::json!({
        "user_id": uid,
        "theme": theme
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
            .unwrap_or_else(|_| "Could not save theme".into()))
    }
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
async fn put_exercise_catalog(token: &str, user_id: &str, name: &str) -> Result<(), String> {
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
        "created_by": user_id,
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
    // `toISOString` is UTC, which selects tomorrow for users west of UTC late at night.
    let date = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date()
    )
}
fn workout_cache_key(user_id: &str) -> String {
    format!("{WORKOUT_CACHE_PREFIX}-{user_id}")
}
fn new_workout_id() -> String {
    web_sys::window()
        .and_then(|window| window.crypto().ok())
        .map(|crypto| crypto.random_uuid())
        .unwrap_or_else(|| {
            format!(
                "workout-{}-{:x}",
                js_sys::Date::now() as u64,
                js_sys::Math::random().to_bits()
            )
        })
}
fn cached_workouts(user_id: Option<&str>) -> Vec<Workout> {
    user_id
        .and_then(|id| LocalStorage::get::<Vec<Workout>>(workout_cache_key(id)).ok())
        .unwrap_or_default()
}
fn cache_workouts(user_id: &str, workouts: &[Workout]) {
    let _ = LocalStorage::set(workout_cache_key(user_id), workouts);
}
fn stored_auth() -> (Option<String>, Option<String>, Option<String>) {
    (
        LocalStorage::get(AUTH_TOKEN_KEY).ok(),
        LocalStorage::get(AUTH_UID_KEY).ok(),
        LocalStorage::get(AUTH_REFRESH_KEY).ok(),
    )
}
fn clear_auth() {
    LocalStorage::delete(AUTH_TOKEN_KEY);
    LocalStorage::delete(AUTH_UID_KEY);
    LocalStorage::delete(AUTH_REFRESH_KEY);
}
fn input_value(e: InputEvent) -> String {
    e.target_unchecked_into::<HtmlInputElement>().value()
}
fn save_draft_exercise(
    draft: &UseStateHandle<Vec<Exercise>>,
    editing_draft_index: &UseStateHandle<Option<usize>>,
    exercise: Exercise,
) {
    let mut next = (**draft).clone();
    match **editing_draft_index {
        Some(index) if index < next.len() => next[index] = exercise,
        _ => next.push(exercise),
    }
    draft.set(next);
    editing_draft_index.set(None);
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
) -> Result<
    (
        Vec<Workout>,
        Vec<String>,
        HashMap<String, String>,
        String,
        Vec<WorkoutTemplate>,
    ),
    String,
> {
    let catalog_rows = get_exercise_catalog(access_token).await?;
    let (catalog_names, alias_map) = build_catalog(&catalog_rows);
    let theme = get_user_theme(access_token, user_id)
        .await?
        .unwrap_or_else(|| "dark".into());
    let remote = canonicalize_workouts(get_workouts(access_token, user_id).await?, &alias_map);
    // The database is authoritative. Local storage is only an account-scoped snapshot
    // for fast rendering, never a source that writes records back during sign-in.
    let remote = merge_workouts(remote, Vec::new());
    // Templates are an enhancement. A missing migration must not block workouts.
    let templates = get_workout_templates(access_token, user_id)
        .await
        .unwrap_or_default();
    cache_workouts(user_id, &remote);
    let _ = LocalStorage::set(THEME_KEY, theme.clone());
    Ok((remote, catalog_names, alias_map, theme, templates))
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
    merged.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.id.cmp(&b.id)));
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
fn draft_view(
    (i, e, on_edit, on_remove): (usize, &Exercise, Callback<MouseEvent>, Callback<MouseEvent>),
) -> Html {
    html! {
        <article class="exercise-entry">
            <div class="entry-head">
                <h3>{format!("{}. {}", i + 1, e.name)}</h3>
                <div class="entry-actions">
                    <button class="text-button" type="button" onclick={on_edit}>{"Edit"}</button>
                    <button class="remove-exercise" type="button" aria-label={format!("Remove {}", e.name)} onclick={on_remove}>{"×"}</button>
                </div>
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

fn draft_entries(
    draft: &[Exercise],
    edit_draft: Callback<usize>,
    remove_draft: Callback<usize>,
) -> Html {
    html! {
        <>
            {for draft.iter().enumerate().map(|(i, e)| {
                let on_remove = {
                    let remove_draft = remove_draft.clone();
                    Callback::from(move |_| remove_draft.emit(i))
                };
                let on_edit = {
                    let edit_draft = edit_draft.clone();
                    Callback::from(move |_| edit_draft.emit(i))
                };
                draft_view((i, e, on_edit, on_remove))
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
            {if workouts.is_empty() {
                html! { <p class="subtle empty-state">{"No workouts yet."}</p> }
            } else {
                html! {
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
                }
            }}
        </div>
    }
}

struct WorkoutEditorProps {
    date: UseStateHandle<String>,
    name: UseStateHandle<String>,
    theme: UseStateHandle<String>,
    theme_select_ref: NodeRef,
    token: UseStateHandle<Option<String>>,
    uid: UseStateHandle<Option<String>>,
    refresh_token: UseStateHandle<Option<String>>,
    weight: UseStateHandle<String>,
    reps: UseStateHandle<String>,
    set_reps: UseStateHandle<[u32; 3]>,
    details: UseStateHandle<String>,
    show_new_exercise: UseStateHandle<bool>,
    draft: UseStateHandle<Vec<Exercise>>,
    editing_draft_index: UseStateHandle<Option<usize>>,
    workouts: UseStateHandle<Vec<Workout>>,
    templates: UseStateHandle<Vec<WorkoutTemplate>>,
    status: UseStateHandle<String>,
    is_saving: UseStateHandle<bool>,
    new_exercise_name: UseStateHandle<String>,
    exercise_search: UseStateHandle<String>,
    template_name: UseStateHandle<String>,
    on_name: Callback<Event>,
    add: Callback<MouseEvent>,
    edit_draft: Callback<usize>,
    remove_draft: Callback<usize>,
    save: Callback<MouseEvent>,
    repeat_last: Callback<MouseEvent>,
    save_template: Callback<MouseEvent>,
    load_template: Callback<WorkoutTemplate>,
    logout: Callback<MouseEvent>,
    load_workout: Callback<Workout>,
    delete_selected: Callback<String>,
    exercise_options: Vec<String>,
    active_tab: UseStateHandle<String>,
}

fn workout_editor_view(props: WorkoutEditorProps) -> Html {
    let WorkoutEditorProps {
        date,
        name,
        theme,
        theme_select_ref,
        token,
        uid,
        refresh_token,
        weight,
        reps,
        set_reps,
        details,
        show_new_exercise,
        draft,
        editing_draft_index,
        workouts,
        templates,
        status,
        is_saving,
        new_exercise_name,
        exercise_search,
        template_name,
        on_name,
        add,
        edit_draft,
        remove_draft,
        save,
        repeat_last,
        save_template,
        load_template,
        logout,
        load_workout,
        delete_selected,
        exercise_options,
        active_tab,
        ..
    } = props;

    let visible_exercise_options: Vec<&String> = exercise_options
        .iter()
        .filter(|exercise| {
            exercise_search.is_empty()
                || exercise
                    .to_ascii_lowercase()
                    .contains(&exercise_search.to_ascii_lowercase())
                || **name == ***exercise
        })
        .collect();

    html! {
        <main class={classes!("app-shell", format!("theme-{}", *theme))}>
            <header class="topbar">
                <div>
                    <p class="eyebrow">{"PERSONAL TRAINING LOG"}</p>
                    <h1>{"Lift Log"}</h1>
                </div>
                <div class="topbar-actions">
                    <label class="theme-picker">
                        <span class="sr-only">{"Theme"}</span>
                        <select
                            ref={theme_select_ref.clone()}
                            data-testid="theme-select"
                            onchange={{
                                let theme = theme.clone();
                                let token = token.clone();
                                let uid = uid.clone();
                                let refresh_token = refresh_token.clone();
                                let status = status.clone();
                                Callback::from(move |e: Event| {
                                    let value = e.target_unchecked_into::<HtmlSelectElement>().value();
                                    let selected = value.clone();
                                    theme.set(selected.clone());
                                    let token = token.clone();
                                    let uid = uid.clone();
                                    let refresh_token = refresh_token.clone();
                                    let status = status.clone();
                                    spawn_local(async move {
                                        let token_value = (*token).clone();
                                        let refresh_value = (*refresh_token).clone();
                                        let uid_value = (*uid).clone();
                                        match current_access_token(token_value, refresh_value).await {
                                            Ok((fresh_token, next_refresh)) => {
                                                token.set(Some(fresh_token.clone()));
                                                if let Some(rt) = next_refresh {
                                                    if let Some(u) = uid_value.as_deref() {
                                                        persist_session(&fresh_token, u, &rt);
                                                    }
                                                    refresh_token.set(Some(rt));
                                                }
                                                if let Some(u) = uid_value {
                                                    if let Err(e) = put_user_theme(&fresh_token, &u, &selected).await {
                                                        status.set(e);
                                                        return;
                                                    }
                                                }
                                                let _ = LocalStorage::set(THEME_KEY, selected);
                                            }
                                            Err(e) => status.set(e),
                                        }
                                    });
                                })
                            }}
                        >
                            <option value="dark" selected={*theme == "dark"}>{"Dark"}</option>
                            <option value="light" selected={*theme == "light"}>{"Light"}</option>
                            <option value="pink" selected={*theme == "pink"}>{"Pink"}</option>
                        </select>
                    </label>
                    <button class="text-button" type="button" onclick={logout}>{"Logout"}</button>
                </div>
            </header>
            {if !status.is_empty() {
                html! { <p class="status" role="status">{(*status).clone()}</p> }
            } else { html! {} }}
            <div class="tab-bar" role="tablist" aria-label="Lift Log sections">
                <button
                    class={classes!("tab-button", (*active_tab == "workout").then_some("active"))}
                    type="button"
                    onclick={{
                        let active_tab = active_tab.clone();
                        Callback::from(move |_| active_tab.set("workout".into()))
                    }}
                >
                    {"Workout"}
                </button>
                <button
                    class={classes!("tab-button", (*active_tab == "history").then_some("active"))}
                    type="button"
                    onclick={{
                        let active_tab = active_tab.clone();
                        Callback::from(move |_| active_tab.set("history".into()))
                    }}
                >
                    {"History"}
                </button>
            </div>
            <section class={classes!("view", (*active_tab == "workout").then_some("active"))}>
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
                    {if workouts.is_empty() {
                        html! {}
                    } else {
                        html! { <button class="text-button repeat-button" type="button" onclick={repeat_last}>{"Repeat last workout"}</button> }
                    }}
                    <div class="template-controls">
                        <label class="field-label">
                            {"Workout template"}
                            <select data-testid="template-select" onchange={{
                                let templates = templates.clone();
                                let load_template = load_template.clone();
                                Callback::from(move |e: Event| {
                                    let id = e.target_unchecked_into::<HtmlSelectElement>().value();
                                    if let Some(template) = templates.iter().find(|template| template.id == id) {
                                        load_template.emit(template.clone());
                                    }
                                })
                            }}>
                                <option value="">{"Choose a template"}</option>
                                {for templates.iter().map(|template| html! {
                                    <option value={template.id.clone()}>{template.name.clone()}</option>
                                })}
                            </select>
                        </label>
                        <span class="template-save-row">
                            <input
                                data-testid="template-name"
                                value={(*template_name).clone()}
                                oninput={{
                                    let template_name = template_name.clone();
                                    Callback::from(move |e: InputEvent| template_name.set(input_value(e)))
                                }}
                                placeholder="Template name"
                            />
                            <button class="text-button" type="button" onclick={save_template}>{"Save template"}</button>
                        </span>
                    </div>
                    <div class="exercise-card">
                        <div class="exercise-stack">
                        <label class="field-label">
                            {"Search exercises"}
                            <input
                                data-testid="exercise-search"
                                value={(*exercise_search).clone()}
                                oninput={{
                                    let exercise_search = exercise_search.clone();
                                    Callback::from(move |e: InputEvent| exercise_search.set(input_value(e)))
                                }}
                                placeholder="Search your exercises"
                            />
                        </label>
                        <label class="field-label">
                            {"Exercise"}
                            <select data-testid="exercise-select" onchange={on_name}>
                                {for visible_exercise_options.iter().map(|exercise| html!{
                                    <option value={(**exercise).clone()} selected={*name == **exercise}>{(**exercise).clone()}</option>
                                })}
                                <option value={ADD_EXERCISE_VALUE} selected={*name == ADD_EXERCISE_VALUE}>{"New Exercise"}</option>
                            </select>
                        </label>
                        {if *show_new_exercise {
                            html! {
                                <label class="field-label">
                                    {"New exercise name"}
                                    <input
                                        data-testid="new-exercise-name"
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
                            <span class="weight-row">
                                <input
                                    data-testid="weight-input"
                                    value={(*weight).clone()}
                                    oninput={{
                                        let weight = weight.clone();
                                        Callback::from(move |e: InputEvent| weight.set(input_value(e)))
                                    }}
                                />
                                <span class="weight-unit">{"lbs"}</span>
                            </span>
                        </label>
                        <label class="field-label">
                            {"Sets"}
                            <div class="sets-grid">
                                {for (0..3).map(|index| {
                                    let set_reps = set_reps.clone();
                                    let reps = reps.clone();
                                    let value = (*set_reps)[index];
                                    html! {
                                        <button
                                            class={classes!("set-chip", (value < 10).then_some("set-chip-dim"))}
                                            type="button"
                                            data-testid={format!("set-rep-{}", index + 1)}
                                            onclick={Callback::from(move |_| {
                                                let mut next = *set_reps;
                                                next[index] = if next[index] == 0 { 10 } else { next[index] - 1 };
                                                set_reps.set(next);
                                                reps.set(format_set_reps(&next));
                                            })}
                                        >
                                            {value}
                                        </button>
                                    }
                                })}
                            </div>
                            <p class="subtle sets-hint">{"Tap a set to count down from 10."}</p>
                        </label>
                        <label class="field-label">
                            {"Details"}
                            <input
                                data-testid="details-input"
                                value={(*details).clone()}
                                oninput={{
                                    let details = details.clone();
                                    Callback::from(move |e: InputEvent| details.set(input_value(e)))
                                }}
                            />
                        </label>
                        <button class="add-button" data-testid="add-exercise-button" type="button" onclick={add}>
                            {if editing_draft_index.is_some() { "Update Exercise" } else { "+ Add Exercise" }}
                        </button>
                        </div>
                    </div>
                    {draft_entries(&draft, edit_draft, remove_draft)}
                    <button class="primary-button save-button" type="button" onclick={save} disabled={*is_saving} aria-busy={is_saving.to_string()}>
                        {if *is_saving { "Saving…" } else { "Log Workout" }}
                    </button>
                </section>
            </section>
            <section class={classes!("view", (*active_tab == "history").then_some("active"))}>
                <div class="section-heading">
                    <div>
                        <p class="eyebrow">{"YOUR LOG"}</p>
                        <h2>{"Workout history"}</h2>
                    </div>
                </div>
                {history_view(&workouts, load_workout, delete_selected)}
            </section>
        </main>
    }
}

#[function_component(App)]
fn app() -> Html {
    let (stored_token, stored_uid, stored_refresh) = stored_auth();
    let token = use_state(|| stored_token);
    let uid = use_state(|| stored_uid.clone());
    let refresh_token = use_state(|| stored_refresh);
    let workouts = use_state(|| cached_workouts(stored_uid.as_deref()));
    let templates = use_state(Vec::<WorkoutTemplate>::new);
    let exercise_catalog = use_state(Vec::<String>::new);
    let exercise_aliases = use_state(HashMap::<String, String>::new);
    let status = use_state(String::new);
    let is_saving = use_state(|| false);
    let email = use_state(String::new);
    let password = use_state(String::new);
    let signup = use_state(|| false);
    let active_tab = use_state(|| "workout".to_string());
    let theme = use_state(stored_theme);
    let theme_select_ref = use_node_ref();
    let date = use_state(today_string);
    let note = use_state(String::new);
    let name = use_state(String::new);
    let new_exercise_name = use_state(String::new);
    let weight = use_state(String::new);
    let set_reps = use_state(default_set_reps);
    let reps = use_state(|| format_set_reps(&default_set_reps()));
    let details = use_state(String::new);
    let show_new_exercise = use_state(|| false);
    let draft = use_state(Vec::<Exercise>::new);
    let editing_id = use_state(|| None::<String>);
    let editing_draft_index = use_state(|| None::<usize>);
    let exercise_search = use_state(String::new);
    let template_name = use_state(String::new);

    {
        let token = token.clone();
        let uid = uid.clone();
        let refresh_token = refresh_token.clone();
        let workouts = workouts.clone();
        let templates = templates.clone();
        let exercise_catalog = exercise_catalog.clone();
        let exercise_aliases = exercise_aliases.clone();
        let theme = theme.clone();
        let status = status.clone();
        use_effect_with((), move |_| {
            if let (Some(access_token), Some(user_id)) = ((*token).clone(), (*uid).clone()) {
                let token = token.clone();
                let uid = uid.clone();
                let refresh_token = refresh_token.clone();
                let workouts = workouts.clone();
                let templates = templates.clone();
                let exercise_catalog = exercise_catalog.clone();
                let exercise_aliases = exercise_aliases.clone();
                let theme = theme.clone();
                let status = status.clone();
                spawn_local(async move {
                    let mut access_token = access_token;
                    let mut refresh_token_value = (*refresh_token).clone();
                    if let Some(rt) = refresh_token_value.clone() {
                        match refresh_session(&rt).await {
                            Ok(a) => {
                                let Some(next_token) = auth_token(&a) else {
                                    clear_auth();
                                    token.set(None);
                                    uid.set(None);
                                    refresh_token.set(None);
                                    status.set(
                                        "Supabase did not return a refreshed session token.".into(),
                                    );
                                    return;
                                };
                                let Some(next_uid) = auth_user_id(&a) else {
                                    clear_auth();
                                    token.set(None);
                                    uid.set(None);
                                    refresh_token.set(None);
                                    status
                                        .set("Supabase did not return a refreshed user id.".into());
                                    return;
                                };
                                refresh_token_value =
                                    auth_refresh_token(&a).or(Some(rt)).or(refresh_token_value);
                                if let Some(current_refresh) = refresh_token_value.as_deref() {
                                    persist_session(&next_token, &next_uid, current_refresh);
                                }
                                access_token = next_token;
                                token.set(Some(access_token.clone()));
                                uid.set(Some(next_uid.clone()));
                                refresh_token.set(refresh_token_value.clone());
                            }
                            Err(e) => {
                                clear_auth();
                                token.set(None);
                                uid.set(None);
                                refresh_token.set(None);
                                status.set(e);
                                return;
                            }
                        }
                    }
                    match sync_user_data(&access_token, &user_id).await {
                        Ok((merged, catalog_names, alias_map, saved_theme, saved_templates)) => {
                            workouts.set(merged);
                            templates.set(saved_templates);
                            exercise_catalog.set(catalog_names);
                            exercise_aliases.set(alias_map);
                            theme.set(saved_theme.clone());
                            let _ = LocalStorage::set(THEME_KEY, saved_theme);
                            status.set(String::new());
                        }
                        Err(e) => {
                            clear_auth();
                            token.set(None);
                            uid.set(None);
                            refresh_token.set(None);
                            status.set(e);
                        }
                    }
                });
            }
            || ()
        });
    }
    {
        let theme = theme.clone();
        let theme_select_ref = theme_select_ref.clone();
        use_effect_with((*theme).clone(), move |current_theme| {
            if let Some(select) = theme_select_ref.cast::<HtmlSelectElement>() {
                select.set_value(current_theme);
            }
            || ()
        });
    }

    let load_workout = {
        let date = date.clone();
        let note = note.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let editing_draft_index = editing_draft_index.clone();
        let name = name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let set_reps = set_reps.clone();
        let show_new_exercise = show_new_exercise.clone();
        let details = details.clone();
        let new_exercise_name = new_exercise_name.clone();
        let active_tab = active_tab.clone();
        let exercise_search = exercise_search.clone();
        let editing_draft_index = editing_draft_index.clone();
        Callback::from(move |w: Workout| {
            date.set(w.date.clone());
            note.set(w.note.clone());
            draft.set(w.exercises.clone());
            editing_id.set(Some(w.id));
            name.set(String::new());
            new_exercise_name.set(String::new());
            weight.set(String::new());
            set_reps.set(default_set_reps());
            reps.set(format_set_reps(&default_set_reps()));
            details.set(String::new());
            show_new_exercise.set(false);
            exercise_search.set(String::new());
            editing_draft_index.set(None);
            active_tab.set("workout".into());
        })
    };
    let submit_auth = {
        let email = email.clone();
        let password = password.clone();
        let signup = signup.clone();
        let token = token.clone();
        let uid = uid.clone();
        let refresh_token = refresh_token.clone();
        let workouts = workouts.clone();
        let templates = templates.clone();
        let exercise_catalog = exercise_catalog.clone();
        let exercise_aliases = exercise_aliases.clone();
        let theme = theme.clone();
        let status = status.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let email = (*email).clone();
            let password = (*password).clone();
            let is_signup = *signup;
            let token = token.clone();
            let uid = uid.clone();
            let refresh_token = refresh_token.clone();
            let workouts = workouts.clone();
            let templates = templates.clone();
            let exercise_catalog = exercise_catalog.clone();
            let exercise_aliases = exercise_aliases.clone();
            let theme = theme.clone();
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
                        let refresh = auth_refresh_token(&a).unwrap_or_default();
                        if refresh.is_empty() {
                            status.set("Supabase did not return a refresh token.".into());
                            return;
                        }
                        persist_session(&access_token, &user_id, &refresh);
                        token.set(Some(access_token.clone()));
                        uid.set(Some(user_id.clone()));
                        refresh_token.set(Some(refresh.clone()));
                        let workouts = workouts.clone();
                        let templates = templates.clone();
                        let exercise_catalog = exercise_catalog.clone();
                        let exercise_aliases = exercise_aliases.clone();
                        let theme = theme.clone();
                        let status = status.clone();
                        spawn_local(async move {
                            match sync_user_data(&access_token, &user_id).await {
                                Ok((
                                    merged,
                                    catalog_names,
                                    alias_map,
                                    saved_theme,
                                    saved_templates,
                                )) => {
                                    workouts.set(merged);
                                    templates.set(saved_templates);
                                    exercise_catalog.set(catalog_names);
                                    exercise_aliases.set(alias_map);
                                    theme.set(saved_theme.clone());
                                    let _ = LocalStorage::set(THEME_KEY, saved_theme);
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
        let uid = uid.clone();
        let refresh_token = refresh_token.clone();
        let token_state = token.clone();
        let uid_state = uid.clone();
        let refresh_state = refresh_token.clone();
        let name = name.clone();
        let new_exercise_name = new_exercise_name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let set_reps = set_reps.clone();
        let details = details.clone();
        let show_new_exercise = show_new_exercise.clone();
        let exercise_search = exercise_search.clone();
        let draft = draft.clone();
        let editing_draft_index = editing_draft_index.clone();
        let status = status.clone();
        let exercise_catalog = exercise_catalog.clone();
        let exercise_aliases = exercise_aliases.clone();
        Callback::from(move |_| {
            let selected = (*name).clone();
            let weight_value = weight.parse().ok();
            let reps_value = (*reps).trim().to_string();
            let details_value = (*details).trim().to_string();

            if *show_new_exercise || selected == ADD_EXERCISE_VALUE {
                let canonical = title_case_name(new_exercise_name.trim());
                if canonical.is_empty() {
                    status.set("Enter a new exercise name first.".into());
                    return;
                }

                let already_exists = exercise_catalog
                    .iter()
                    .any(|item| item.eq_ignore_ascii_case(&canonical));

                let token = (*token).clone();
                let uid = (*uid).clone();
                let refresh_token = (*refresh_token).clone();
                let token_state = token_state.clone();
                let uid_state = uid_state.clone();
                let refresh_state = refresh_state.clone();
                let draft = draft.clone();
                let editing_draft_index = editing_draft_index.clone();
                let status = status.clone();
                let exercise_catalog = exercise_catalog.clone();
                let exercise_aliases = exercise_aliases.clone();
                let name = name.clone();
                let new_exercise_name = new_exercise_name.clone();
                let weight_state = weight.clone();
                let reps_state = reps.clone();
                let set_reps_state = set_reps.clone();
                let details_state = details.clone();
                let show_new_exercise_state = show_new_exercise.clone();
                spawn_local(async move {
                    if let (Some(t), Some(current_user_id)) = (token, uid) {
                        let (fresh_token, next_refresh) =
                            match current_access_token(Some(t), refresh_token).await {
                                Ok(session) => session,
                                Err(e) => {
                                    status.set(e);
                                    return;
                                }
                            };
                        token_state.set(Some(fresh_token.clone()));
                        uid_state.set(Some(current_user_id.clone()));
                        if let Some(next_refresh) = next_refresh {
                            persist_session(&fresh_token, &current_user_id, &next_refresh);
                            refresh_state.set(Some(next_refresh));
                        }
                        if !already_exists {
                            if let Err(e) =
                                put_exercise_catalog(&fresh_token, &current_user_id, &canonical)
                                    .await
                            {
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
                        save_draft_exercise(
                            &draft,
                            &editing_draft_index,
                            Exercise {
                                name: canonical,
                                weight: weight_value,
                                reps: reps_value,
                                details: details_value,
                            },
                        );
                        name.set(String::new());
                        new_exercise_name.set(String::new());
                        weight_state.set(String::new());
                        set_reps_state.set(default_set_reps());
                        reps_state.set(format_set_reps(&default_set_reps()));
                        details_state.set(String::new());
                        show_new_exercise_state.set(false);
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
            save_draft_exercise(
                &draft,
                &editing_draft_index,
                Exercise {
                    name: canonical_name,
                    weight: weight_value,
                    reps: reps_value,
                    details: details_value,
                },
            );
            name.set(selected);
            new_exercise_name.set(String::new());
            weight.set(String::new());
            set_reps.set(default_set_reps());
            reps.set(format_set_reps(&default_set_reps()));
            details.set(String::new());
            show_new_exercise.set(false);
            exercise_search.set(String::new());
        })
    };
    let logout = {
        let token = token.clone();
        let uid = uid.clone();
        let workouts = workouts.clone();
        let templates = templates.clone();
        let exercise_catalog = exercise_catalog.clone();
        let exercise_aliases = exercise_aliases.clone();
        let status = status.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let editing_draft_index = editing_draft_index.clone();
        let name = name.clone();
        let new_exercise_name = new_exercise_name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let set_reps = set_reps.clone();
        let details = details.clone();
        let show_new_exercise = show_new_exercise.clone();
        let exercise_search = exercise_search.clone();
        let template_name = template_name.clone();
        Callback::from(move |_| {
            clear_auth();
            token.set(None);
            uid.set(None);
            workouts.set(Vec::new());
            templates.set(Vec::new());
            exercise_catalog.set(Vec::new());
            exercise_aliases.set(HashMap::new());
            status.set(String::new());
            draft.set(Vec::new());
            editing_id.set(None);
            editing_draft_index.set(None);
            name.set(String::new());
            new_exercise_name.set(String::new());
            weight.set(String::new());
            set_reps.set(default_set_reps());
            reps.set(format_set_reps(&default_set_reps()));
            details.set(String::new());
            show_new_exercise.set(false);
            exercise_search.set(String::new());
            template_name.set(String::new());
        })
    };
    let remove_draft = {
        let draft = draft.clone();
        let editing_draft_index = editing_draft_index.clone();
        Callback::from(move |index: usize| {
            let mut next = (*draft).clone();
            if index < next.len() {
                next.remove(index);
                draft.set(next);
                match *editing_draft_index {
                    Some(current) if current == index => editing_draft_index.set(None),
                    Some(current) if current > index => editing_draft_index.set(Some(current - 1)),
                    _ => {}
                }
            }
        })
    };
    let edit_draft = {
        let draft = draft.clone();
        let name = name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let set_reps = set_reps.clone();
        let details = details.clone();
        let show_new_exercise = show_new_exercise.clone();
        let status = status.clone();
        let editing_draft_index = editing_draft_index.clone();
        Callback::from(move |index: usize| {
            let Some(exercise) = (*draft).get(index).cloned() else {
                return;
            };
            name.set(exercise.name);
            weight.set(
                exercise
                    .weight
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            );
            let parsed = parse_set_reps(&exercise.reps);
            set_reps.set(parsed);
            reps.set(format_set_reps(&parsed));
            details.set(exercise.details);
            show_new_exercise.set(false);
            editing_draft_index.set(Some(index));
            status.set("Update the exercise, then press Update Exercise to apply it.".into());
        })
    };
    let reset_editor: Callback<()> = {
        let date = date.clone();
        let note = note.clone();
        let name = name.clone();
        let new_exercise_name = new_exercise_name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let set_reps = set_reps.clone();
        let details = details.clone();
        let show_new_exercise = show_new_exercise.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let editing_draft_index = editing_draft_index.clone();
        let active_tab = active_tab.clone();
        let exercise_search = exercise_search.clone();
        Callback::from(move |_: ()| {
            date.set(today_string());
            note.set(String::new());
            name.set(String::new());
            new_exercise_name.set(String::new());
            weight.set(String::new());
            set_reps.set(default_set_reps());
            reps.set(format_set_reps(&default_set_reps()));
            details.set(String::new());
            show_new_exercise.set(false);
            draft.set(Vec::new());
            editing_id.set(None);
            editing_draft_index.set(None);
            exercise_search.set(String::new());
            active_tab.set("workout".into());
        })
    };
    let repeat_last = {
        let workouts = workouts.clone();
        let date = date.clone();
        let note = note.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let editing_draft_index = editing_draft_index.clone();
        let name = name.clone();
        let new_exercise_name = new_exercise_name.clone();
        let weight = weight.clone();
        let reps = reps.clone();
        let set_reps = set_reps.clone();
        let details = details.clone();
        let show_new_exercise = show_new_exercise.clone();
        let exercise_search = exercise_search.clone();
        let active_tab = active_tab.clone();
        let status = status.clone();
        Callback::from(move |_| {
            let Some(last) = (*workouts).first().cloned() else {
                status.set("No previous workout to repeat yet.".into());
                return;
            };
            date.set(today_string());
            note.set(last.note);
            draft.set(last.exercises);
            editing_id.set(None);
            editing_draft_index.set(None);
            name.set(String::new());
            new_exercise_name.set(String::new());
            weight.set(String::new());
            set_reps.set(default_set_reps());
            reps.set(format_set_reps(&default_set_reps()));
            details.set(String::new());
            show_new_exercise.set(false);
            exercise_search.set(String::new());
            active_tab.set("workout".into());
            status.set("Last workout copied. Update anything you need, then log it.".into());
        })
    };
    let load_template = {
        let date = date.clone();
        let note = note.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let editing_draft_index = editing_draft_index.clone();
        let status = status.clone();
        Callback::from(move |template: WorkoutTemplate| {
            date.set(today_string());
            note.set(template.note);
            draft.set(template.exercises);
            editing_id.set(None);
            editing_draft_index.set(None);
            status.set(format!("Loaded template: {}", template.name));
        })
    };
    let save_template = {
        let token_handle = token.clone();
        let uid_handle = uid.clone();
        let refresh_handle = refresh_token.clone();
        let templates = templates.clone();
        let template_name = template_name.clone();
        let draft = draft.clone();
        let note = note.clone();
        let status = status.clone();
        Callback::from(move |_| {
            let name = title_case_name(template_name.trim());
            if name.is_empty() {
                status.set("Name the template first.".into());
                return;
            }
            if draft.is_empty() {
                status.set("Add at least one exercise before saving a template.".into());
                return;
            }
            let template = WorkoutTemplate {
                id: new_workout_id(),
                name,
                note: (*note).clone(),
                exercises: (*draft).clone(),
            };
            let token = (*token_handle).clone();
            let uid = (*uid_handle).clone();
            let refresh_token = (*refresh_handle).clone();
            let templates = templates.clone();
            let template_name = template_name.clone();
            let status = status.clone();
            let token_handle = token_handle.clone();
            let refresh_handle = refresh_handle.clone();
            spawn_local(async move {
                let (Some(token), Some(uid)) = (token, uid) else {
                    status.set("Sign in first.".into());
                    return;
                };
                match current_access_token(Some(token), refresh_token).await {
                    Ok((fresh_token, next_refresh)) => {
                        match put_workout_template(&fresh_token, &uid, &template).await {
                            Ok(()) => {
                                let mut next = (*templates).clone();
                                next.push(template);
                                next.sort_by(|a, b| a.name.cmp(&b.name));
                                templates.set(next);
                                template_name.set(String::new());
                                token_handle.set(Some(fresh_token.clone()));
                                if let Some(refresh) = next_refresh {
                                    persist_session(&fresh_token, &uid, &refresh);
                                    refresh_handle.set(Some(refresh));
                                }
                                status.set("Workout template saved.".into());
                            }
                            Err(error) => status.set(error),
                        }
                    }
                    Err(error) => status.set(error),
                }
            });
        })
    };
    let save = {
        let token_handle = token.clone();
        let uid_handle = uid.clone();
        let refresh_handle = refresh_token.clone();
        let workouts = workouts.clone();
        let date = date.clone();
        let note = note.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let status = status.clone();
        let reset_editor = reset_editor.clone();
        let is_saving = is_saving.clone();
        Callback::from(move |_| {
            if draft.is_empty() {
                status.set("Add at least one exercise.".into());
                return;
            }
            if *is_saving {
                return;
            }
            is_saving.set(true);
            let previous_id = (*editing_id).clone();
            let w = Workout {
                id: previous_id.as_ref().cloned().unwrap_or_else(new_workout_id),
                date: (*date).clone(),
                note: (*note).clone(),
                exercises: (*draft).clone(),
            };
            let token = (*token_handle).clone();
            let uid = (*uid_handle).clone();
            let refresh_token = (*refresh_handle).clone();
            let workouts = workouts.clone();
            let token_state = token_handle.clone();
            let uid_state = uid_handle.clone();
            let refresh_state = refresh_handle.clone();
            let status = status.clone();
            let reset_editor = reset_editor.clone();
            let is_saving = is_saving.clone();
            spawn_local(async move {
                if let (Some(u), Some(t)) = (uid, token) {
                    match current_access_token(Some(t), refresh_token).await {
                        Ok((fresh_token, next_refresh)) => {
                            match put_workout(&fresh_token, &u, &w).await {
                                Ok(()) => {
                                    let mut all = (*workouts).clone();
                                    if let Some(id) = previous_id.as_ref() {
                                        all.retain(|existing| existing.id != *id);
                                    }
                                    all.push(w);
                                    all.sort_by(|a, b| {
                                        b.date.cmp(&a.date).then_with(|| a.id.cmp(&b.id))
                                    });
                                    cache_workouts(&u, &all);
                                    workouts.set(all);
                                    token_state.set(Some(fresh_token.clone()));
                                    if let Some(rt) = next_refresh {
                                        persist_session(&fresh_token, &u, &rt);
                                        refresh_state.set(Some(rt));
                                    }
                                    uid_state.set(Some(u.clone()));
                                    reset_editor.emit(());
                                    status.set("Workout saved to Supabase.".into());
                                    is_saving.set(false);
                                }
                                Err(e) => {
                                    status.set(e);
                                    is_saving.set(false);
                                }
                            }
                        }
                        Err(e) => {
                            status.set(e);
                            is_saving.set(false);
                        }
                    }
                } else {
                    status.set("Sign in first.".into());
                    is_saving.set(false);
                }
            });
        })
    };
    let delete_selected = {
        let token_handle = token.clone();
        let uid_handle = uid.clone();
        let refresh_handle = refresh_token.clone();
        let workouts = workouts.clone();
        let status = status.clone();
        Callback::from(move |id: String| {
            let confirmed = web_sys::window()
                .and_then(|window| {
                    window
                        .confirm_with_message("Delete this workout? This cannot be undone.")
                        .ok()
                })
                .unwrap_or(false);
            if !confirmed {
                return;
            }
            let token = (*token_handle).clone();
            let uid = (*uid_handle).clone();
            let refresh_token = (*refresh_handle).clone();
            let workouts = workouts.clone();
            let status = status.clone();
            let token_state = token_handle.clone();
            let uid_state = uid_handle.clone();
            let refresh_state = refresh_handle.clone();
            spawn_local(async move {
                if let (Some(u), Some(t)) = (uid, token) {
                    match current_access_token(Some(t), refresh_token).await {
                        Ok((fresh_token, next_refresh)) => {
                            match delete_workout(&fresh_token, &u, &id).await {
                                Ok(()) => {
                                    let mut next = (*workouts).clone();
                                    next.retain(|w| w.id != id);
                                    cache_workouts(&u, &next);
                                    workouts.set(next);
                                    token_state.set(Some(fresh_token.clone()));
                                    if let Some(rt) = next_refresh {
                                        persist_session(&fresh_token, &u, &rt);
                                        refresh_state.set(Some(rt));
                                    }
                                    uid_state.set(Some(u.clone()));
                                    status.set("Workout deleted.".into());
                                }
                                Err(e) => status.set(e),
                            }
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
        let set_reps = set_reps.clone();
        let show_new_exercise = show_new_exercise.clone();
        let details = details.clone();
        let workouts = workouts.clone();
        Callback::from(move |e: Event| {
            let v = e.target_unchecked_into::<HtmlSelectElement>().value();
            name.set(v.clone());
            if v == ADD_EXERCISE_VALUE {
                show_new_exercise.set(true);
                new_exercise_name.set(String::new());
                weight.set(String::new());
                set_reps.set(default_set_reps());
                reps.set(format_set_reps(&default_set_reps()));
                details.set(String::new());
            } else if let Some(p) = previous(&workouts, &v) {
                show_new_exercise.set(false);
                weight.set(p.weight.map(|x| x.to_string()).unwrap_or_default());
                let parsed = parse_set_reps(&p.reps);
                set_reps.set(parsed);
                reps.set(format_set_reps(&parsed));
                details.set(p.details);
            } else {
                show_new_exercise.set(false);
                weight.set(String::new());
                set_reps.set(default_set_reps());
                reps.set(format_set_reps(&default_set_reps()));
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
    {
        let name = name.clone();
        let draft = draft.clone();
        let editing_id = editing_id.clone();
        let exercise_options = exercise_options.clone();
        use_effect_with(
            (exercise_options.len(), draft.len(), editing_id.is_some()),
            move |_| {
                if editing_id.is_none() && draft.is_empty() && name.is_empty() {
                    if let Some(first) = exercise_options.first() {
                        name.set(first.clone());
                    } else {
                        name.set(String::new());
                    }
                }
                || ()
            },
        );
    }
    html! {
        <>
            {workout_editor_view(WorkoutEditorProps {
                date,
                name,
                theme,
                theme_select_ref,
                token,
                uid,
                refresh_token,
                weight,
                reps,
                set_reps,
                details,
                show_new_exercise,
                draft,
                editing_draft_index,
                workouts,
                templates,
                status,
                is_saving,
                new_exercise_name,
                exercise_search,
                template_name,
                on_name,
                add,
                edit_draft,
                remove_draft,
                save,
                repeat_last,
                save_template,
                load_template,
                logout,
                load_workout,
                delete_selected,
                exercise_options,
                active_tab,
            })}
        </>
    }
}
fn main() {
    yew::Renderer::<App>::new().render();
}
