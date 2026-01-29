CREATE TABLE breweries (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  notes TEXT,
  website TEXT,
  address TEXT,
  lat DOUBLE PRECISION,
  lng DOUBLE PRECISION,
  drink_menu TEXT,
  food_schedule TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE food_vendors (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  notes TEXT,
  website TEXT,
  menu TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE schedule_entries (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  brewery_id UUID NOT NULL REFERENCES breweries(id),
  brewery_name TEXT NOT NULL,
  food_vendor_id UUID NOT NULL REFERENCES food_vendors(id),
  food_vendor_name TEXT NOT NULL,
  open_hours TSTZRANGE NOT NULL,
  source TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX schedule_entries_open_hours ON schedule_entries USING GIST (open_hours);
CREATE INDEX schedule_entries_brewery_idx ON schedule_entries(brewery_id);
CREATE INDEX schedule_entries_vendor_idx ON schedule_entries(food_vendor_id);
