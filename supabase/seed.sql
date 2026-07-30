-- One-shot seed script for the Supabase SQL editor.
-- This seeds only the account with email mohammadfawaz89@gmail.com.

with target_user as (
  select id as user_id
  from auth.users
  where email = 'mohammadfawaz89@gmail.com'
  limit 1
),
seed_rows as (
  select *
  from (values
    ('seed-2026-07-28', date '2026-07-28', ''::text, '[
      {"name":"Bench Press","weight":125,"unit":"lb","reps":"9, 9, 9","details":""},
      {"name":"Barbell Squats","weight":155,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Dumbbell Overhead Press","weight":40,"unit":"lb","reps":"8, 8, 6","details":""},
      {"name":"Stiff-Leg Deadlift","weight":145,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Seated Flies","weight":150,"unit":"lb","reps":"10, 8","details":""},
      {"name":"Tricep Cable Pulldown","weight":32.5,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Dumbbell Bicep Curls","weight":30,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Leg Raises","weight":null,"unit":"bodyweight","reps":"10, 10","details":""}
    ]'::jsonb),
    ('seed-2026-07-20', date '2026-07-20', ''::text, '[
      {"name":"Barbell Overhead Press","weight":85,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Elevated Split Squats","weight":35,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Seated Rows","weight":125,"unit":"lb","reps":"6, 6","details":""},
      {"name":"Cable Chest Flies","weight":160,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Stiff-Leg Deadlift","weight":140,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Tricep Cable Pulldown","weight":30,"unit":"lb","reps":"8, 8, 8","details":""}
    ]'::jsonb),
    ('seed-2026-07-16', date '2026-07-16', ''::text, '[
      {"name":"Assisted Pull-Ups","weight":31.25,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Barbell Squats","weight":150,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Bench Press","weight":125,"unit":"lb","reps":"9, 8, 5","details":""},
      {"name":"Cable Single-Arm Row","weight":100,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Dumbbell Overhead Press","weight":40,"unit":"lb","reps":"10, 8, 7","details":""},
      {"name":"Seated Flies","weight":150,"unit":"lb","reps":"10, 8","details":""}
    ]'::jsonb),
    ('seed-2026-07-10', date '2026-07-10', ''::text, '[
      {"name":"Dumbbell Overhead Press","weight":40,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Stiff-Leg Deadlift","weight":135,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Cable Chest Flies","weight":180,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Tricep Cable Pulldown","weight":42.5,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Walking Lunges","weight":60,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Lat Pulldown","weight":120,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb),
    ('seed-2026-07-06', date '2026-07-06', ''::text, '[
      {"name":"Assisted Pull-Ups","weight":37.5,"unit":"lb","reps":"10, 10, 7","details":""},
      {"name":"Bench Press","weight":125,"unit":"lb","reps":"9, 7, 6","details":""},
      {"name":"Leg Press","weight":190,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Barbell Overhead Press","weight":80,"unit":"lb","reps":"8, 7","details":""},
      {"name":"Stiff-Leg Deadlift","weight":135,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Seated Flies","weight":145,"unit":"lb","reps":"8, 8, 8","details":""}
    ]'::jsonb),
    ('seed-2026-07-02', date '2026-07-02', ''::text, '[
      {"name":"Assisted Pull-Ups","weight":37.5,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Cable Chest Flies","weight":160,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Elevated Split Squats","weight":30,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Seated Rows","weight":125,"unit":"lb","reps":"10, 10, 7","details":""},
      {"name":"Dumbbell Overhead Press","weight":35,"unit":"lb","reps":"10, 7, 6","details":""},
      {"name":"Stiff-Leg Deadlift","weight":125,"unit":"lb","reps":"8, 8, 8","details":""}
    ]'::jsonb),
    ('seed-2026-06-27', date '2026-06-27', ''::text, '[
      {"name":"Bench Press","weight":125,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Barbell Squats","weight":145,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Barbell Overhead Press","weight":80,"unit":"lb","reps":"8, 7, 4","details":""},
      {"name":"Cable Single-Arm Row","weight":95,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Seated Flies","weight":140,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Tricep Cable Pulldown","weight":39,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb),
    ('seed-2026-06-22', date '2026-06-22', ''::text, '[
      {"name":"Assisted Pull-Ups","weight":44,"unit":"lb","reps":"10, 10, 8","details":""},
      {"name":"Barbell Squats","weight":145,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Barbell Overhead Press","weight":75,"unit":"lb","reps":"10, 10, 8","details":""},
      {"name":"Stiff-Leg Deadlift","weight":105,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Hip Abductors","weight":180,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Seated Flies","weight":135,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb),
    ('seed-2026-06-17', date '2026-06-17', ''::text, '[
      {"name":"Bench Press","weight":120,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Elevated Split Squats","weight":30,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Seated Rows","weight":125,"unit":"lb","reps":"10, 10, 8","details":""},
      {"name":"Cable Chest Flies","weight":160,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Tricep Cable Pulldown","weight":39,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb),
    ('seed-2026-06-15', date '2026-06-15', ''::text, '[
      {"name":"Assisted Pull-Ups","weight":44,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Barbell Overhead Press","weight":70,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Lat Pulldown","weight":115,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Cable Single-Arm Row","weight":90,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Incline Dumbbell Chest Press","weight":40,"unit":"lb","reps":"10, 9, 8","details":""},
      {"name":"Leg Press","weight":185,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb),
    ('seed-2026-06-12', date '2026-06-12', ''::text, '[
      {"name":"Assisted Pull-Ups","weight":50,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Bench Press","weight":120,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Barbell Squats","weight":140,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Dumbbell Overhead Press","weight":35,"unit":"lb","reps":"10, 10, 7","details":""},
      {"name":"Seated Flies","weight":130,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Seated Rows","weight":125,"unit":"lb","reps":"10, 8, 8","details":""}
    ]'::jsonb),
    ('seed-2026-06-10', date '2026-06-10', ''::text, '[
      {"name":"Assisted Pull-Ups","weight":50,"unit":"lb","reps":"10, 10, 9","details":""},
      {"name":"Barbell Overhead Press","weight":70,"unit":"lb","reps":"10, 10, 7","details":""},
      {"name":"Leg Press","weight":180,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Cable Chest Flies","weight":120,"unit":"lb","reps":"12, 12, 12","details":""},
      {"name":"Elevated Split Squats","weight":25,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Tricep Cable Pulldown","weight":37.5,"unit":"lb","reps":"10, 10, 10","details":""}
    ]'::jsonb),
    ('seed-2026-06-08', date '2026-06-08', ''::text, '[
      {"name":"Assisted Pull-Ups","weight":50,"unit":"lb","reps":"8, 8, 8","details":""},
      {"name":"Bench Press","weight":120,"unit":"lb","reps":"8, 7, 6","details":""},
      {"name":"Barbell Squats","weight":135,"unit":"lb","reps":"10, 10, 10","details":""},
      {"name":"Dumbbell Overhead Press","weight":35,"unit":"lb","reps":"10, 9, 7","details":""},
      {"name":"Seated Rows","weight":125,"unit":"lb","reps":"10, 8, 8","details":""},
      {"name":"Incline Dumbbell Chest Press","weight":40,"unit":"lb","reps":"9, 5","details":""}
    ]'::jsonb)
  ) as v(id, workout_date, note, exercises)
)
insert into public.workouts (id, user_id, workout_date, note, exercises)
select v.id, t.user_id, v.workout_date, v.note, v.exercises
from seed_rows v
cross join target_user t
on conflict (user_id, id) do update
  set workout_date = excluded.workout_date,
      note = excluded.note,
      exercises = excluded.exercises;
