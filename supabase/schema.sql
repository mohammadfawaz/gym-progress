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
