-- Apply once in the Supabase SQL editor. It preserves all existing workouts.
-- This migration makes custom exercises private to their creator and adds
-- audit timestamps required for safer sync/conflict handling.

alter table public.workouts
  add column if not exists updated_at timestamptz not null default now();

alter table public.exercise_catalog
  add column if not exists created_by uuid references auth.users(id) on delete cascade;

alter table public.exercise_catalog enable row level security;

drop policy if exists "Users can read the exercise catalog" on public.exercise_catalog;
create policy "Users can read the exercise catalog"
on public.exercise_catalog for select
using (created_by is null or created_by = auth.uid());

drop policy if exists "Users can add exercises" on public.exercise_catalog;
create policy "Users can add exercises"
on public.exercise_catalog for insert
with check (created_by = auth.uid());

drop policy if exists "Users can update exercises" on public.exercise_catalog;
create policy "Users can update exercises"
on public.exercise_catalog for update
using (created_by = auth.uid())
with check (created_by = auth.uid());

create or replace function public.set_updated_at()
returns trigger
language plpgsql
as $$
begin
  new.updated_at = now();
  return new;
end;
$$;

drop trigger if exists workouts_set_updated_at on public.workouts;
create trigger workouts_set_updated_at
before update on public.workouts
for each row execute function public.set_updated_at();

drop trigger if exists user_settings_set_updated_at on public.user_settings;
create trigger user_settings_set_updated_at
before update on public.user_settings
for each row execute function public.set_updated_at();
