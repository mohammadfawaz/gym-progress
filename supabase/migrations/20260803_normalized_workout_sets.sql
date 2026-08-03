-- Apply once in the Supabase SQL editor after 20260802_reliability.sql.
-- Existing `workouts.exercises` JSON stays in place for backwards compatibility.
-- The tables below are a normalized, query-friendly projection of every workout
-- exercise and individual set. They are kept in sync whenever a workout is saved.

create table if not exists public.workout_exercises (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  workout_id text not null,
  position integer not null check (position >= 1),
  name text not null,
  weight numeric,
  details text not null default '',
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  unique (user_id, workout_id, position),
  foreign key (user_id, workout_id)
    references public.workouts (user_id, id) on delete cascade
);

create table if not exists public.exercise_sets (
  id uuid primary key default gen_random_uuid(),
  workout_exercise_id uuid not null
    references public.workout_exercises(id) on delete cascade,
  set_number integer not null check (set_number >= 1),
  reps integer not null check (reps >= 0 and reps <= 100),
  weight numeric,
  created_at timestamptz not null default now(),
  unique (workout_exercise_id, set_number)
);

create table if not exists public.workout_templates (
  id text not null,
  user_id uuid not null references auth.users(id) on delete cascade,
  name text not null check (char_length(trim(name)) > 0),
  note text not null default '',
  exercises jsonb not null default '[]'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  primary key (user_id, id)
);

create index if not exists workout_exercises_user_name_idx
  on public.workout_exercises (user_id, name, workout_id);
create index if not exists exercise_sets_workout_exercise_idx
  on public.exercise_sets (workout_exercise_id, set_number);

alter table public.workout_exercises enable row level security;
alter table public.exercise_sets enable row level security;
alter table public.workout_templates enable row level security;

drop policy if exists "Users can read their own workout exercises" on public.workout_exercises;
create policy "Users can read their own workout exercises"
on public.workout_exercises for select using (auth.uid() = user_id);

drop policy if exists "Users can read their own exercise sets" on public.exercise_sets;
create policy "Users can read their own exercise sets"
on public.exercise_sets for select using (
  exists (
    select 1 from public.workout_exercises
    where workout_exercises.id = exercise_sets.workout_exercise_id
      and workout_exercises.user_id = auth.uid()
  )
);

drop policy if exists "Users can read their own workout templates" on public.workout_templates;
create policy "Users can read their own workout templates"
on public.workout_templates for select using (auth.uid() = user_id);
drop policy if exists "Users can insert their own workout templates" on public.workout_templates;
create policy "Users can insert their own workout templates"
on public.workout_templates for insert with check (auth.uid() = user_id);
drop policy if exists "Users can update their own workout templates" on public.workout_templates;
create policy "Users can update their own workout templates"
on public.workout_templates for update using (auth.uid() = user_id) with check (auth.uid() = user_id);
drop policy if exists "Users can delete their own workout templates" on public.workout_templates;
create policy "Users can delete their own workout templates"
on public.workout_templates for delete using (auth.uid() = user_id);

create or replace function public.sync_normalized_workout_sets()
returns trigger
language plpgsql
security definer
set search_path = public
as $$
declare
  exercise_row record;
  set_row record;
  exercise_id uuid;
  exercise_weight numeric;
begin
  delete from public.workout_exercises
  where user_id = new.user_id and workout_id = new.id;

  for exercise_row in
    select element, ordinal
    from jsonb_array_elements(new.exercises) with ordinality as entries(element, ordinal)
  loop
    exercise_weight := nullif(exercise_row.element->>'weight', '')::numeric;
    insert into public.workout_exercises (
      user_id, workout_id, position, name, weight, details
    ) values (
      new.user_id,
      new.id,
      exercise_row.ordinal,
      coalesce(nullif(trim(exercise_row.element->>'name'), ''), 'Unnamed Exercise'),
      exercise_weight,
      coalesce(exercise_row.element->>'details', '')
    ) returning id into exercise_id;

    for set_row in
      select value, ordinal
      from regexp_split_to_table(
        coalesce(exercise_row.element->>'reps', ''),
        '\\s*,\\s*'
      ) with ordinality as sets(value, ordinal)
    loop
      if trim(set_row.value) ~ '^\\d+$' then
        insert into public.exercise_sets (
          workout_exercise_id, set_number, reps, weight
        ) values (
          exercise_id,
          set_row.ordinal,
          trim(set_row.value)::integer,
          exercise_weight
        );
      end if;
    end loop;
  end loop;

  return new;
end;
$$;

drop trigger if exists workouts_sync_normalized_sets on public.workouts;
create trigger workouts_sync_normalized_sets
after insert or update of exercises on public.workouts
for each row execute function public.sync_normalized_workout_sets();

drop trigger if exists workout_templates_set_updated_at on public.workout_templates;
create trigger workout_templates_set_updated_at before update on public.workout_templates
for each row execute function public.set_updated_at();

-- Backfill the normalized records from current workout history.
update public.workouts set exercises = exercises;
