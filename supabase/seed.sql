-- One-shot seed script for the Supabase SQL editor.
-- This seeds only the account with email mohammadfawaz89@gmail.com.

with target_user as (
  select id as user_id
  from auth.users
  where email = 'mohammadfawaz89@gmail.com'
  limit 1
),
catalog_rows as (
  select *
  from (values
    ('Assisted Pull-Ups', array['Assisted pull-ups']::text[], 10),
    ('Barbell Overhead Press', array['Barbell overhead press']::text[], 20),
    ('Barbell Squats', array['Squats', 'Barbell squats']::text[], 30),
    ('Bench Press', array['Bench press']::text[], 40),
    ('Cable Chest Flies', array['Cable chest flies', 'Cable chest press', 'Cable flies']::text[], 50),
    ('Cable Single-Arm Row', array['Cable single-arm row', 'Cable single hand row pulls']::text[], 60),
    ('Dumbbell Bicep Curls', array['Dumbbell bicep curls']::text[], 70),
    ('Dumbbell Overhead Press', array['Seated dumbbell overhead press', 'Dumbell overhead press', 'Dumbel overhead press']::text[], 80),
    ('Elevated Split Squats', array['Elevated split squats', 'Split squats']::text[], 90),
    ('Hip Abductors', array['Hip abductors']::text[], 100),
    ('Incline Dumbbell Chest Press', array['Inclined dumbell chest press', 'Inclined dumbell chess press']::text[], 110),
    ('Lat Pulldown', array['Lat cable pull down']::text[], 120),
    ('Leg Press', array['Leg press']::text[], 130),
    ('Leg Raises', array['Knee raises', 'Bosu leg raises']::text[], 140),
    ('Seated Flies', array['Seated flies']::text[], 150),
    ('Seated Rows', array['Seated rows']::text[], 160),
    ('Stiff-Leg Deadlift', array['Stiff deadlift', 'Stiff deadlifts', 'Stiff legs deadlift']::text[], 170),
    ('Tricep Cable Pulldown', array['Tricep pulldown', 'Tricep cable pulldown', 'Triceps pulldown', 'Triceps overhead cable pull']::text[], 180),
    ('Walking Lunges', array['Lunges walk']::text[], 190)
  ) as v(canonical_name, aliases, sort_order)
)
insert into public.exercise_catalog (canonical_name, aliases, sort_order)
select canonical_name, aliases, sort_order
from catalog_rows
on conflict (canonical_name) do update
  set aliases = excluded.aliases,
      sort_order = excluded.sort_order;

insert into public.workouts (id, user_id, workout_date, note, exercises)
values
  (
    'seed-2026-07-28',
    (select user_id from target_user),
    '2026-07-28',
    '',
    '[
      {"name":"Bench Press","weight":125,"unit":"lb","reps":"9, 9, 9","details":""},
      {"name":"Barbell Squats","weight":155,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Dumbbell Overhead Press","weight":40,"unit":"lb","reps":"8, 8, 6","details":""},
      {"name":"Stiff-Leg Deadlift","weight":145,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Seated Flies","weight":150,"unit":"lb","reps":"10, 8","details":""},
      {"name":"Tricep Cable Pulldown","weight":32.5,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Dumbbell Bicep Curls","weight":30,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Leg Raises","weight":null,"unit":"bodyweight","reps":"10, 10","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-07-20',
    (select user_id from target_user),
    '2026-07-20',
    '',
    '[
      {"name":"Barbell Overhead Press","weight":85,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Elevated Split Squats","weight":35,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Seated Rows","weight":125,"unit":"lb","reps":"6, 6","details":""},
      {"name":"Cable Chest Flies","weight":160,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Stiff-Leg Deadlift","weight":140,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Tricep Cable Pulldown","weight":30,"unit":"lb","reps":"8, 8, 8","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-07-16',
    (select user_id from target_user),
    '2026-07-16',
    '',
    '[
      {"name":"Assisted Pull-Ups","weight":31.25,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Barbell Squats","weight":150,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Bench Press","weight":125,"unit":"lb","reps":"9, 8, 5","details":""},
      {"name":"Cable Single-Arm Row","weight":100,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Dumbbell Overhead Press","weight":40,"unit":"lb","reps":"10, 8, 7","details":""},
      {"name":"Seated Flies","weight":150,"unit":"lb","reps":"10, 8","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-07-10',
    (select user_id from target_user),
    '2026-07-10',
    '',
    '[
      {"name":"Dumbbell Overhead Press","weight":40,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Stiff-Leg Deadlift","weight":135,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Cable Chest Flies","weight":180,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Tricep Cable Pulldown","weight":42.5,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Walking Lunges","weight":60,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Lat Pulldown","weight":120,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-07-06',
    (select user_id from target_user),
    '2026-07-06',
    '',
    '[
      {"name":"Assisted Pull-Ups","weight":37.5,"unit":"lb","reps":"10, 10, 7","details":""},
      {"name":"Bench Press","weight":125,"unit":"lb","reps":"9, 7, 6","details":""},
      {"name":"Leg Press","weight":190,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Barbell Overhead Press","weight":80,"unit":"lb","reps":"8, 7","details":""},
      {"name":"Stiff-Leg Deadlift","weight":135,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Seated Flies","weight":145,"unit":"lb","reps":"8, 8, 8","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-07-02',
    (select user_id from target_user),
    '2026-07-02',
    '',
    '[
      {"name":"Assisted Pull-Ups","weight":37.5,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Cable Chest Flies","weight":160,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Elevated Split Squats","weight":30,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Seated Rows","weight":125,"unit":"lb","reps":"10, 10, 7","details":""},
      {"name":"Dumbbell Overhead Press","weight":35,"unit":"lb","reps":"10, 7, 6","details":""},
      {"name":"Stiff-Leg Deadlift","weight":125,"unit":"lb","reps":"8, 8, 8","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-06-27',
    (select user_id from target_user),
    '2026-06-27',
    '',
    '[
      {"name":"Bench Press","weight":125,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Barbell Squats","weight":145,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Barbell Overhead Press","weight":80,"unit":"lb","reps":"8, 7, 4","details":""},
      {"name":"Cable Single-Arm Row","weight":95,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Seated Flies","weight":140,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Tricep Cable Pulldown","weight":39,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-06-22',
    (select user_id from target_user),
    '2026-06-22',
    '',
    '[
      {"name":"Assisted Pull-Ups","weight":44,"unit":"lb","reps":"10, 10, 8","details":""},
      {"name":"Barbell Squats","weight":145,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Barbell Overhead Press","weight":75,"unit":"lb","reps":"10, 10, 8","details":""},
      {"name":"Stiff-Leg Deadlift","weight":105,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Hip Abductors","weight":180,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Seated Flies","weight":135,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-06-17',
    (select user_id from target_user),
    '2026-06-17',
    '',
    '[
      {"name":"Bench Press","weight":120,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Elevated Split Squats","weight":30,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Seated Rows","weight":125,"unit":"lb","reps":"10, 10, 8","details":""},
      {"name":"Cable Chest Flies","weight":160,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Tricep Cable Pulldown","weight":39,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-06-15',
    (select user_id from target_user),
    '2026-06-15',
    '',
    '[
      {"name":"Assisted Pull-Ups","weight":44,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Barbell Overhead Press","weight":70,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Lat Pulldown","weight":115,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Cable Single-Arm Row","weight":90,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Incline Dumbbell Chest Press","weight":40,"unit":"lb","reps":"10, 9, 8","details":""},
      {"name":"Leg Press","weight":185,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-06-12',
    (select user_id from target_user),
    '2026-06-12',
    '',
    '[
      {"name":"Assisted Pull-Ups","weight":50,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Bench Press","weight":120,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Barbell Squats","weight":140,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Dumbbell Overhead Press","weight":35,"unit":"lb","reps":"10, 10, 7","details":""},
      {"name":"Seated Flies","weight":130,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Seated Rows","weight":125,"unit":"lb","reps":"10, 8, 8","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-06-10',
    (select user_id from target_user),
    '2026-06-10',
    '',
    '[
      {"name":"Assisted Pull-Ups","weight":50,"unit":"lb","reps":"10, 10, 9","details":""},
      {"name":"Barbell Overhead Press","weight":70,"unit":"lb","reps":"10, 10, 7","details":""},
      {"name":"Leg Press","weight":180,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Cable Chest Flies","weight":120,"unit":"lb","reps":"12, 12, 12","details":""},
      {"name":"Elevated Split Squats","weight":25,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Tricep Cable Pulldown","weight":37.5,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb
  ),
  (
    'seed-2026-06-08',
    (select user_id from target_user),
    '2026-06-08',
    '',
    '[
      {"name":"Assisted Pull-Ups","weight":50,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Bench Press","weight":120,"unit":"lb","reps":"8, 7, 6","details":""},
      {"name":"Barbell Squats","weight":135,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Dumbbell Overhead Press","weight":35,"unit":"lb","reps":"10, 9, 7","details":""},
      {"name":"Seated Rows","weight":125,"unit":"lb","reps":"10, 8, 8","details":""},
      {"name":"Incline Dumbbell Chest Press","weight":40,"unit":"lb","reps":"9, 5","details":""}
    ]'::jsonb
  )
on conflict (user_id, id) do update
  set workout_date = excluded.workout_date,
      note = excluded.note,
      exercises = excluded.exercises;

update public.workouts
set exercises = (
  select coalesce(
    jsonb_agg(
      jsonb_set(elem, '{name}', to_jsonb(public.canonical_exercise_name(elem->>'name')), true)
      order by ord
    ),
    '[]'::jsonb
  )
  from jsonb_array_elements(public.workouts.exercises) with ordinality as parts(elem, ord)
)
where user_id = (select user_id from target_user);
