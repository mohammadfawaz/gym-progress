use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Exercise {
    pub(crate) name: String,
    pub(crate) weight: Option<f64>,
    pub(crate) reps: String,
    pub(crate) details: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Workout {
    pub(crate) id: String,
    pub(crate) date: String,
    pub(crate) note: String,
    pub(crate) exercises: Vec<Exercise>,
}

#[derive(Deserialize)]
pub(crate) struct Auth {
    #[serde(default)]
    pub(crate) access_token: Option<String>,
    #[serde(default)]
    pub(crate) refresh_token: Option<String>,
    #[serde(default)]
    pub(crate) session: Option<AuthSession>,
    #[serde(default)]
    pub(crate) user: Option<User>,
}

#[derive(Deserialize)]
pub(crate) struct AuthSession {
    #[serde(default)]
    pub(crate) access_token: Option<String>,
    #[serde(default)]
    pub(crate) refresh_token: Option<String>,
    #[serde(default)]
    pub(crate) user: Option<User>,
}

#[derive(Deserialize)]
pub(crate) struct User {
    pub(crate) id: String,
}

#[derive(Deserialize)]
pub(crate) struct DbWorkout {
    pub(crate) id: String,
    pub(crate) workout_date: String,
    pub(crate) note: String,
    pub(crate) exercises: Vec<Exercise>,
}

#[derive(Deserialize)]
pub(crate) struct DbExerciseCatalog {
    pub(crate) canonical_name: String,
    #[serde(default)]
    pub(crate) aliases: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct DbUserSettings {
    pub(crate) theme: String,
}
