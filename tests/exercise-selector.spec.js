const { test, expect } = require('@playwright/test');

const STORAGE_KEY = 'lift-log-v1';
const AUTH_TOKEN_KEY = 'lift-log-auth-token';
const AUTH_UID_KEY = 'lift-log-auth-uid';
const THEME_KEY = 'lift-log-theme-v2';

const exerciseCatalog = [
  { canonical_name: 'Bench Press', aliases: ['Bench press'], sort_order: 1 },
  { canonical_name: 'Barbell Squats', aliases: ['Squats', 'Barbell squats'], sort_order: 2 },
  { canonical_name: 'Dumbbell Overhead Press', aliases: ['Seated dumbbell overhead press'], sort_order: 3 },
];

function jsonResponse(payload, status = 200) {
  return {
    status,
    contentType: 'application/json',
    body: JSON.stringify(payload),
  };
}

async function mockSupabase(page, options = {}) {
  const catalogRequests = [];
  const workoutRequests = [];

  await page.route('**/rest/v1/**', async route => {
    const url = new URL(route.request().url());
    const { pathname } = url;
    const method = route.request().method();

    if (pathname.endsWith('/exercise_catalog')) {
      if (method === 'GET') {
        await route.fulfill(jsonResponse(options.catalog ?? exerciseCatalog));
        return;
      }
      if (method === 'POST') {
        catalogRequests.push(JSON.parse(route.request().postData() || '{}'));
        await route.fulfill(jsonResponse([], 201));
        return;
      }
    }

    if (pathname.endsWith('/workouts')) {
      if (method === 'GET') {
        await route.fulfill(jsonResponse(options.workouts ?? []));
        return;
      }
      if (method === 'POST') {
        workoutRequests.push(JSON.parse(route.request().postData() || '{}'));
        await route.fulfill(jsonResponse([], 201));
        return;
      }
      if (method === 'DELETE') {
        await route.fulfill(jsonResponse([], 200));
        return;
      }
    }

    await route.fulfill(jsonResponse({ error: `Unhandled route: ${method} ${pathname}` }, 500));
  });

  await page.addInitScript(({ storageKey, tokenKey, uidKey, themeKey }) => {
    localStorage.removeItem(storageKey);
    localStorage.setItem(tokenKey, 'test-token');
    localStorage.setItem(uidKey, '00000000-0000-0000-0000-000000000001');
    localStorage.removeItem(themeKey);
  }, {
    storageKey: STORAGE_KEY,
    tokenKey: AUTH_TOKEN_KEY,
    uidKey: AUTH_UID_KEY,
    themeKey: THEME_KEY,
  });

  return { catalogRequests, workoutRequests };
}

test('loads the exercise catalog and defaults to a real exercise', async ({ page }) => {
  await mockSupabase(page);

  await page.goto('/');

  const select = page.getByTestId('exercise-select');
  await expect(select).toBeVisible();
  await expect(select).toHaveValue('Barbell Squats');
  await expect(page.getByTestId('theme-select')).toHaveValue('dark');

  const options = await select.locator('option').allTextContents();
  expect(options).toEqual([
    'Barbell Squats',
    'Bench Press',
    'Dumbbell Overhead Press',
    'New Exercise',
  ]);
  expect(options).not.toContain('Choose an exercise');
  await expect(page.getByRole('button', { name: 'History' })).toBeVisible();
  await expect(page.getByText('Workout history')).not.toBeVisible();
});

test('switches workout history into its own tab', async ({ page }) => {
  await mockSupabase(page, {
    workouts: [
      {
        id: 'seed-1',
        date: '2026-07-30',
        note: '',
        exercises: [
          {
            name: 'Cable Chest Press',
            weight: 160,
            unit: 'lb',
            reps: '10, 10, 10',
            details: '',
          },
        ],
      },
    ],
  });

  await page.goto('/');

  await expect(page.getByText('Workout history')).not.toBeVisible();
  await page.getByRole('button', { name: 'History' }).click();
  await expect(page.getByText('Workout history')).toBeVisible();
  await expect(page.getByText('Cable Chest Press')).toBeVisible();
  await page.getByRole('button', { name: 'Workout' }).click();
  await expect(page.getByTestId('exercise-select')).toBeVisible();
});

test('can add an existing exercise and a new exercise', async ({ page }) => {
  const { catalogRequests } = await mockSupabase(page, {
    workouts: [
      {
        id: 'seed-1',
        date: '2026-07-30',
        note: '',
        exercises: [
          {
            name: 'Cable Chest Press',
            weight: 160,
            unit: 'lb',
            reps: '10, 10, 10',
            details: '',
          },
        ],
      },
    ],
  });

  await page.goto('/');

  const select = page.getByTestId('exercise-select');
  const weight = page.getByTestId('weight-input');
  const set1 = page.getByTestId('set-rep-1');
  const set2 = page.getByTestId('set-rep-2');
  const addButton = page.getByTestId('add-exercise-button');

  await select.selectOption('Cable Chest Press');
  await weight.fill('160');
  await expect(set1).toContainText('10');
  await set1.click();
  await set1.click();
  await expect(set1).toContainText('8');
  await addButton.click();

  await expect(page.locator('.workout-form .exercise-entry')).toContainText('Cable Chest Press');
  await expect(page.locator('.workout-form .exercise-entry')).toContainText('8, 10, 10');
  await expect(set1).toContainText('10');

  await select.selectOption('__add_exercise__');
  await expect(page.getByTestId('new-exercise-name')).toBeVisible();
  await page.getByTestId('new-exercise-name').fill('Romanian Deadlift');
  await weight.fill('135');
  await expect(set2).toContainText('10');
  await set2.click();
  await set2.click();
  await set2.click();
  await set2.click();
  await set2.click();
  await set2.click();
  await set2.click();
  await set2.click();
  await set2.click();
  await set2.click();
  await expect(set2).toContainText('10');
  await addButton.click();

  await expect(page.locator('.workout-form .exercise-entry')).toContainText('Romanian Deadlift');
  await expect(catalogRequests).toContainEqual(expect.objectContaining({
    canonical_name: 'Romanian Deadlift',
  }));
});

test('keeps custom exercise mode working after adding a new exercise', async ({ page }) => {
  const { catalogRequests } = await mockSupabase(page, {
    workouts: [],
  });

  await page.goto('/');

  const select = page.getByTestId('exercise-select');
  const weight = page.getByTestId('weight-input');
  const addButton = page.getByTestId('add-exercise-button');
  const newExerciseName = page.getByTestId('new-exercise-name');
  const set1 = page.getByTestId('set-rep-1');

  await select.selectOption('__add_exercise__');
  await expect(newExerciseName).toBeVisible();

  await newExerciseName.fill('Front Squat');
  await weight.fill('155');
  await set1.click();
  await set1.click();
  await addButton.click();

  await expect(page.locator('.workout-form .exercise-entry')).toContainText('Front Squat');
  await expect(newExerciseName).toBeVisible();

  await newExerciseName.fill('Chest Supported Row');
  await weight.fill('110');
  await addButton.click();

  await expect(page.locator('.workout-form .exercise-entry')).toContainText('Chest Supported Row');
  await expect(catalogRequests).toContainEqual(expect.objectContaining({
    canonical_name: 'Front Squat',
  }));
  await expect(catalogRequests).toContainEqual(expect.objectContaining({
    canonical_name: 'Chest Supported Row',
  }));
});
