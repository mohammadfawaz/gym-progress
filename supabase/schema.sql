create table if not exists public.workouts (
  id text not null,
  user_id uuid not null references auth.users(id) on delete cascade,
  workout_date date not null,
  note text not null default '',
  exercises jsonb not null default '[]'::jsonb,
  created_at timestamptz not null default now(),
  primary key (user_id, id)
);

alter table public.workouts enable row level security;

drop policy if exists "Users can read their own workouts" on public.workouts;
create policy "Users can read their own workouts" on public.workouts for select using (auth.uid() = user_id);
drop policy if exists "Users can insert their own workouts" on public.workouts;
create policy "Users can insert their own workouts" on public.workouts for insert with check (auth.uid() = user_id);
drop policy if exists "Users can update their own workouts" on public.workouts;
create policy "Users can update their own workouts" on public.workouts for update using (auth.uid() = user_id) with check (auth.uid() = user_id);
drop policy if exists "Users can delete their own workouts" on public.workouts;
create policy "Users can delete their own workouts" on public.workouts for delete using (auth.uid() = user_id);

create table if not exists public.exercise_catalog (
  canonical_name text primary key,
  aliases text[] not null default '{}'::text[],
  sort_order integer not null default 0,
  created_at timestamptz not null default now()
);

alter table public.exercise_catalog enable row level security;

drop policy if exists "Users can read the exercise catalog" on public.exercise_catalog;
create policy "Users can read the exercise catalog" on public.exercise_catalog for select using (true);
drop policy if exists "Users can add exercises" on public.exercise_catalog;
create policy "Users can add exercises" on public.exercise_catalog for insert with check (auth.role() = 'authenticated');
drop policy if exists "Users can update exercises" on public.exercise_catalog;
create policy "Users can update exercises" on public.exercise_catalog for update using (auth.role() = 'authenticated') with check (auth.role() = 'authenticated');

create table if not exists public.user_settings (
  user_id uuid primary key references auth.users(id) on delete cascade,
  theme text not null default 'dark',
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

alter table public.user_settings enable row level security;

drop policy if exists "Users can read their own settings" on public.user_settings;
create policy "Users can read their own settings" on public.user_settings for select using (auth.uid() = user_id);
drop policy if exists "Users can insert their own settings" on public.user_settings;
create policy "Users can insert their own settings" on public.user_settings for insert with check (auth.uid() = user_id);
drop policy if exists "Users can update their own settings" on public.user_settings;
create policy "Users can update their own settings" on public.user_settings for update using (auth.uid() = user_id) with check (auth.uid() = user_id);

create or replace function public.canonical_exercise_name(input_name text)
returns text
language sql
stable
as $$
  select coalesce(
    (
      select canonical_name
      from public.exercise_catalog
      where lower(canonical_name) = lower(input_name)
         or lower(input_name) = any (
           select lower(alias)
           from unnest(aliases) as alias
         )
      limit 1
    ),
    input_name
  );
$$;
