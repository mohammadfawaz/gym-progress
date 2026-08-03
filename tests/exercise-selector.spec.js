const { test, expect } = require("@playwright/test");

const STORAGE_KEY = "lift-log-v1";
const AUTH_TOKEN_KEY = "lift-log-auth-token";
const AUTH_UID_KEY = "lift-log-auth-uid";
const THEME_KEY = "lift-log-theme-v2";

const exerciseCatalog = [
  { canonical_name: "Bench Press", aliases: ["Bench press"], sort_order: 1 },
  {
    canonical_name: "Barbell Squats",
    aliases: ["Squats", "Barbell squats"],
    sort_order: 2,
  },
  {
    canonical_name: "Dumbbell Overhead Press",
    aliases: ["Seated dumbbell overhead press"],
    sort_order: 3,
  },
];

function jsonResponse(payload, status = 200) {
  return {
    status,
    contentType: "application/json",
    body: JSON.stringify(payload),
  };
}

async function mockSupabase(page, options = {}) {
  const catalogRequests = [];
  const workoutRequests = [];
  const deleteRequests = [];
  const themeRequests = [];
  const templateRequests = [];
  let workouts = [...(options.workouts ?? [])];
  let templates = [...(options.templates ?? [])];

  await page.route("**/auth/v1/**", async (route) => {
    const url = new URL(route.request().url());
    if (
      route.request().method() === "POST" &&
      url.pathname.endsWith("/token") &&
      url.searchParams.get("grant_type") === "refresh_token"
    ) {
      await route.fulfill(
        jsonResponse(
          options.refreshAuth ?? {
            access_token: "refreshed-test-token",
            refresh_token: "refreshed-test-refresh",
            user: { id: "00000000-0000-0000-0000-000000000001" },
          },
        ),
      );
      return;
    }
    await route.fulfill(
      jsonResponse(
        {
          error: `Unhandled auth route: ${route.request().method()} ${url.pathname}`,
        },
        500,
      ),
    );
  });

  await page.route("**/rest/v1/**", async (route) => {
    const url = new URL(route.request().url());
    const { pathname } = url;
    const method = route.request().method();

    if (pathname.endsWith("/exercise_catalog")) {
      if (method === "GET") {
        await route.fulfill(jsonResponse(options.catalog ?? exerciseCatalog));
        return;
      }
      if (method === "POST") {
        catalogRequests.push(JSON.parse(route.request().postData() || "{}"));
        await route.fulfill(jsonResponse([], 201));
        return;
      }
    }

    if (pathname.endsWith("/user_settings")) {
      if (method === "GET") {
        await route.fulfill(
          jsonResponse(options.userSettings ?? [{ theme: "dark" }]),
        );
        return;
      }
      if (method === "POST") {
        themeRequests.push(JSON.parse(route.request().postData() || "{}"));
        await route.fulfill(jsonResponse([], 201));
        return;
      }
    }

    if (pathname.endsWith("/workout_templates")) {
      if (method === "GET") {
        await route.fulfill(jsonResponse(templates));
        return;
      }
      if (method === "POST") {
        const template = JSON.parse(route.request().postData() || "{}");
        templateRequests.push(template);
        templates.push(template);
        await route.fulfill(jsonResponse([], 201));
        return;
      }
    }

    if (pathname.endsWith("/workouts")) {
      if (method === "GET") {
        await route.fulfill(jsonResponse(workouts));
        return;
      }
      if (method === "POST") {
        const workout = JSON.parse(route.request().postData() || "{}");
        workoutRequests.push(workout);
        const index = workouts.findIndex(
          (existing) => existing.id === workout.id,
        );
        if (index >= 0) workouts[index] = workout;
        else workouts.push(workout);
        await route.fulfill(jsonResponse([], 201));
        return;
      }
      if (method === "DELETE") {
        deleteRequests.push({ pathname, search: url.searchParams.toString() });
        const id = url.searchParams.get("id")?.replace(/^eq\./, "");
        workouts = workouts.filter((workout) => workout.id !== id);
        await route.fulfill(jsonResponse([], 200));
        return;
      }
    }

    await route.fulfill(
      jsonResponse({ error: `Unhandled route: ${method} ${pathname}` }, 500),
    );
  });

  await page.context().addInitScript(
    ({
      storageKey,
      tokenKey,
      uidKey,
      refreshKey,
      themeKey,
      tokenValue,
      uidValue,
      refreshValue,
      themeValue,
    }) => {
      if (tokenValue !== null && localStorage.getItem(tokenKey) === null) {
        localStorage.setItem(tokenKey, JSON.stringify(tokenValue));
      }
      if (uidValue !== null && localStorage.getItem(uidKey) === null) {
        localStorage.setItem(uidKey, JSON.stringify(uidValue));
      }
      if (refreshValue !== null && localStorage.getItem(refreshKey) === null) {
        localStorage.setItem(refreshKey, JSON.stringify(refreshValue));
      }
      if (themeValue === null) {
        localStorage.removeItem(themeKey);
      } else if (localStorage.getItem(themeKey) === null) {
        localStorage.setItem(themeKey, JSON.stringify(themeValue));
      }
    },
    {
      storageKey: STORAGE_KEY,
      tokenKey: AUTH_TOKEN_KEY,
      uidKey: AUTH_UID_KEY,
      refreshKey: "lift-log-auth-refresh",
      themeKey: THEME_KEY,
      tokenValue: options.token ?? "test-token",
      uidValue: options.uid ?? "00000000-0000-0000-0000-000000000001",
      refreshValue: options.refreshToken ?? null,
      themeValue: Object.prototype.hasOwnProperty.call(options, "initialTheme")
        ? options.initialTheme
        : null,
    },
  );

  return {
    catalogRequests,
    workoutRequests,
    deleteRequests,
    themeRequests,
    templateRequests,
  };
}

async function openWorkoutTab(page) {
  await page.getByRole("button", { name: "Workout", exact: true }).click();
  await expect(page.getByTestId("exercise-select")).toBeVisible();
}

async function openHistoryTab(page) {
  await page.getByRole("button", { name: "History" }).click();
  await expect(page.getByText("Workout history")).toBeVisible();
}

async function selectExistingExercise(page, name) {
  await page.getByTestId("exercise-select").selectOption(name);
}

async function addExercise(page, { name, weight, clicks = 0, details = "" }) {
  const select = page.getByTestId("exercise-select");
  const weightInput = page.getByTestId("weight-input");
  const detailsInput = page.getByTestId("details-input");
  await select.selectOption(name);
  if (weight !== undefined) {
    await weightInput.fill(String(weight));
  }
  if (details !== undefined) {
    await detailsInput.fill(details);
  }
  const setButtons = [1, 2, 3].map((n) => page.getByTestId(`set-rep-${n}`));
  for (let i = 0; i < clicks; i += 1) {
    await setButtons[0].click();
  }
  await page.getByTestId("add-exercise-button").click();
}

test("loads the exercise catalog and defaults to a real exercise", async ({
  page,
}) => {
  await mockSupabase(page);

  await page.goto("/");
  const select = page.getByTestId("exercise-select");
  await expect(select).toBeVisible();
  await expect(select).toHaveValue("Barbell Squats");
  await expect(page.getByTestId("theme-select")).toHaveValue("dark");

  const options = await select.locator("option").allTextContents();
  expect(options).toEqual([
    "Barbell Squats",
    "Bench Press",
    "Dumbbell Overhead Press",
    "New Exercise",
  ]);
  expect(options).not.toContain("Choose an exercise");
  await expect(page.getByRole("button", { name: "History" })).toBeVisible();
  await expect(page.getByText("Workout history")).not.toBeVisible();
  await expect(page.getByTestId("theme-select").locator("option")).toHaveText([
    "Dark",
    "Light",
    "Pink",
  ]);
});

test("restores a saved session and server theme on refresh", async ({
  page,
}) => {
  const { workoutRequests } = await mockSupabase(page, {
    refreshToken: "refresh-token-1",
    userSettings: [{ theme: "pink" }],
  });

  await page.goto("/");
  await expect(page.getByTestId("theme-select")).toHaveValue("pink");

  await openWorkoutTab(page);
  await page.getByTestId("exercise-select").selectOption("Bench Press");
  await page.getByTestId("weight-input").fill("185");
  await page.getByTestId("set-rep-1").click();
  await page.getByTestId("set-rep-1").click();
  await page.getByTestId("add-exercise-button").click();
  await page.getByRole("button", { name: "Log Workout" }).click();

  await expect(page.getByText("Workout saved to Supabase.")).toBeVisible();
  expect(workoutRequests).toContainEqual(
    expect.objectContaining({
      workout_date: "2026-08-02",
    }),
  );

  await page.reload();

  await expect(page.getByTestId("theme-select")).toHaveValue("pink");
  await openHistoryTab(page);
  await expect(page.locator(".workout-list .exercise-summary")).toContainText(
    "Bench Press",
  );
});

test("switches workout history into its own tab", async ({ page }) => {
  await mockSupabase(page, {
    workouts: [
      {
        id: "seed-1",
        workout_date: "2026-07-30",
        note: "",
        exercises: [
          {
            name: "Cable Chest Press",
            weight: 160,
            unit: "lb",
            reps: "10, 10, 10",
            details: "",
          },
        ],
      },
    ],
  });

  await page.goto("/");

  await expect(page.getByText("Workout history")).not.toBeVisible();
  await page.getByRole("button", { name: "History" }).click();
  await expect(page.getByText("Workout history")).toBeVisible();
  await expect(page.locator(".workout-list .exercise-summary")).toContainText(
    "Cable Chest Press",
  );
  await page.getByRole("button", { name: "Workout" }).click();
  await expect(page.getByTestId("exercise-select")).toBeVisible();
});

test("can add an existing exercise and then create a new one", async ({
  page,
}) => {
  const { catalogRequests } = await mockSupabase(page, {
    workouts: [],
  });

  await page.goto("/");

  const select = page.getByTestId("exercise-select");
  const weight = page.getByTestId("weight-input");
  const newExerciseName = page.getByTestId("new-exercise-name");
  const set1 = page.getByTestId("set-rep-1");
  const set2 = page.getByTestId("set-rep-2");

  await selectExistingExercise(page, "Bench Press");
  await weight.fill("185");
  await set1.click();
  await set1.click();
  await page.getByTestId("add-exercise-button").click();

  await expect(page.locator(".workout-form .exercise-entry")).toContainText(
    "Bench Press",
  );
  await expect(select).toHaveValue("Bench Press");
  expect(await select.locator("option").allTextContents()).toContain(
    "Bench Press",
  );

  await select.selectOption("__add_exercise__");
  await expect(newExerciseName).toBeVisible();

  await newExerciseName.fill("Front Squat");
  await weight.fill("155");
  await set2.click();
  await set2.click();
  await page.getByTestId("add-exercise-button").click();

  await expect(
    page.locator(".workout-form .exercise-entry").last(),
  ).toContainText("Front Squat");
  expect(await select.locator("option").allTextContents()).toContain(
    "Front Squat",
  );
  await expect(catalogRequests).toContainEqual(
    expect.objectContaining({
      canonical_name: "Front Squat",
    }),
  );
});

test("keeps history sorted by date and edits replace the existing workout", async ({
  page,
}) => {
  await mockSupabase(page, {
    workouts: [
      {
        id: "seed-older",
        workout_date: "2026-07-10",
        note: "",
        exercises: [
          {
            name: "Bench Press",
            weight: 165,
            unit: "lb",
            reps: "8, 8, 8",
            details: "",
          },
        ],
      },
      {
        id: "seed-newer",
        workout_date: "2026-07-28",
        note: "",
        exercises: [
          {
            name: "Barbell Squats",
            weight: 155,
            unit: "lb",
            reps: "8, 8, 8",
            details: "",
          },
        ],
      },
    ],
  });

  await page.goto("/");
  await openHistoryTab(page);

  const cards = page.locator(".workout-list .workout-card");
  await expect(cards).toHaveCount(2);
  await expect(cards.nth(0).locator("time")).toHaveText("2026-07-28");
  await expect(cards.nth(1).locator("time")).toHaveText("2026-07-10");

  await cards.nth(1).getByRole("button", { name: "Edit" }).click();
  await page
    .locator(".exercise-entry")
    .getByRole("button", { name: "Edit" })
    .click();
  await expect(page.locator(".workout-form .exercise-entry")).toHaveCount(1);
  await expect(page.getByTestId("add-exercise-button")).toHaveText(
    "Update Exercise",
  );
  await page.getByTestId("weight-input").fill("170");
  await page.getByTestId("add-exercise-button").click();
  await expect(page.locator(".workout-form .exercise-entry")).toHaveCount(1);
  await page.getByRole("button", { name: "Log Workout" }).click();

  await openHistoryTab(page);
  await expect(page.locator(".workout-list .workout-card")).toHaveCount(2);
  await expect(
    page.locator(".workout-list .workout-card").nth(0).locator("time"),
  ).toHaveText("2026-07-28");
  await expect(
    page.locator(".workout-list .workout-card").nth(1).locator("time"),
  ).toHaveText("2026-07-10");
  await expect(
    page.locator(".workout-list .workout-card").nth(1),
  ).toContainText("170 lbs");
});

test("deletes a workout from the history and sends the delete request", async ({
  page,
}) => {
  const { deleteRequests } = await mockSupabase(page, {
    workouts: [
      {
        id: "seed-delete",
        workout_date: "2026-07-30",
        note: "",
        exercises: [
          {
            name: "Cable Chest Press",
            weight: 160,
            unit: "lb",
            reps: "10, 10, 10",
            details: "",
          },
        ],
      },
    ],
  });

  await page.goto("/");
  await openHistoryTab(page);
  page.once("dialog", (dialog) => dialog.accept());
  await page.getByRole("button", { name: "Delete" }).click();

  await expect(page.getByText("Workout deleted.")).toBeVisible();
  await expect(page.getByText("No workouts yet.")).toBeVisible();
  expect(deleteRequests).toHaveLength(1);
  expect(deleteRequests[0].search).toContain("id=eq.seed-delete");
});

test("filters exercises and repeats the last workout as a new draft", async ({
  page,
}) => {
  await mockSupabase(page, {
    workouts: [
      {
        id: "seed-repeat",
        workout_date: "2026-07-30",
        note: "Full body",
        exercises: [
          {
            name: "Bench Press",
            weight: 165,
            reps: "8, 8, 8",
            details: "Controlled",
          },
        ],
      },
    ],
  });

  await page.goto("/");
  await page.getByTestId("exercise-search").fill("squat");
  await expect(page.getByTestId("exercise-select").locator("option")).toHaveText([
    "Barbell Squats",
    "New Exercise",
  ]);

  await page.getByRole("button", { name: "Repeat last workout" }).click();
  await expect(page.locator(".workout-form .exercise-entry")).toHaveCount(1);
  await expect(page.locator(".workout-form .exercise-entry")).toContainText(
    "Bench Press",
  );
  await expect(page.getByTestId("exercise-search")).toHaveValue("");
});

test("saves and loads a workout template", async ({ page }) => {
  const { templateRequests } = await mockSupabase(page);
  await page.goto("/");
  await addExercise(page, { name: "Bench Press", weight: 185 });
  await page.getByTestId("template-name").fill("Push Day");
  await page.getByRole("button", { name: "Save template" }).click();

  await expect(page.getByText("Workout template saved.")).toBeVisible();
  expect(templateRequests).toHaveLength(1);
  await expect(page.getByTestId("template-select")).toContainText("Push Day");

  await page.getByTestId("template-select").selectOption({ label: "Push Day" });
  await expect(page.getByText("Loaded template: Push Day")).toBeVisible();
  await expect(page.locator(".workout-form .exercise-entry")).toContainText(
    "Bench Press",
  );
});

test("keeps the theme dropdown in sync with the server theme", async ({
  page,
}) => {
  const { themeRequests } = await mockSupabase(page, {
    userSettings: [{ theme: "light" }],
    initialTheme: "pink",
  });

  await page.goto("/");

  await expect(page.getByTestId("theme-select")).toHaveValue("light");
  await page.getByTestId("theme-select").selectOption("pink");
  await expect(page.getByTestId("theme-select")).toHaveValue("pink");
  expect(themeRequests).toContainEqual(
    expect.objectContaining({ theme: "pink" }),
  );
});
