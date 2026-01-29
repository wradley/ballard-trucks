INSERT INTO breweries (id, name, notes, website, address, lat, lng, drink_menu, food_schedule)
VALUES
  ('ddbef262-8ae4-413c-9fa3-e4fbc40175b5', 'Stoup Brewing', 'Brewery', 'https://www.stoupbrewing.com/ballard/', '1108 NW 52nd St, Seattle, WA 98107', 47.66653130601593, -122.3711539291425, 'https://www.stoupbrewing.com/ballard/#whats-on-tap-ballard', 'https://www.stoupbrewing.com/ballard/#food-truck-schedule-ballard'),
  ('c8176998-6c38-4813-a9ec-1e45a710e6dc', 'Bale Breaker x Yonder Cider', 'Taste the East, out West', 'https://www.bbycballard.com/', '826 NW 49th Street, Seattle, WA 98107', 47.66454208318039, -122.36755886363453, 'https://www.bbycballard.com/current-taplist', 'https://www.bbycballard.com/food-trucks-1-1');


INSERT INTO food_vendors (id, name, notes, website, menu)
VALUES
  ('def4c743-7ca0-444c-8294-bfc454e57461', 'El Pirata Tortas Y Burritos', 'Mexican-inspired dishes', 'https://elpiratatortas.com', 'https://elpiratatortas.com/menu'),
  ('c65ebe31-6b68-4373-b41d-760ba01476e2', 'Where Ya At Matt', 'New Orleans soul food', 'https://www.whereyaatmatt.com', 'https://www.whereyaatmatt.com/menu'),
  ('8ba304ab-3d7c-445a-b77c-bde1768c89b2', 'The Little Pearl Oyster Bar', 'A mobile seafood bar featuring the finest oysters from the Salish Sea.', 'https://salishseagreens.com/pages/catering', 'https://salishseagreens.com/pages/catering'),
  ('09cfecc0-60a9-4d0a-a1ec-b18f8fbae752', 'Tacos & Beer', 'Authentic Cocina Mexicana', 'https://www.tacosandbeerseattle.com', 'https://www.tacosandbeerseattle.com/menu'),
  ('1c89db87-e201-4295-a94a-34430d1dd2d3', 'Birrieria Pepe El Toro', 'Taco! Tortas! Burritos! Quesadillas!', 'https://www.birrieria-pepeeltoro.com', 'https://www.birrieria-pepeeltoro.com/menu');

INSERT INTO schedule_entries (
  id,
  brewery_id,
  brewery_name,
  food_vendor_id,
  food_vendor_name,
  open_hours,
  source
)
VALUES
  ('1f2fa18e-96e6-4ba4-aa91-f967f2ce270a', 'ddbef262-8ae4-413c-9fa3-e4fbc40175b5', 'Stoup Brewing', 'def4c743-7ca0-444c-8294-bfc454e57461', 'El Pirata Tortas Y Burritos', '[2026-02-01 17:00:00+00, 2026-02-02 03:00:00+00]', 'seed'),
  ('9426bb02-07ba-4c2e-a436-b433a9afff96', 'ddbef262-8ae4-413c-9fa3-e4fbc40175b5', 'Stoup Brewing', 'c65ebe31-6b68-4373-b41d-760ba01476e2', 'Where Ya At Matt', '[2026-02-03 01:00:00+00, 2026-02-03 04:00:00+00]', 'seed'),
  ('315ff286-d5be-48d7-978e-be385b634d4b', 'c8176998-6c38-4813-a9ec-1e45a710e6dc', 'Bale Breaker x Yonder Cider', '8ba304ab-3d7c-445a-b77c-bde1768c89b2', 'The Little Pearl Oyster Bar', '[2026-02-01 16:00:00+00, 2026-02-02 01:00:00+00]', 'seed'),
  ('2130bc93-efd1-4347-8d25-631c4b96e13f', 'c8176998-6c38-4813-a9ec-1e45a710e6dc', 'Bale Breaker x Yonder Cider', '09cfecc0-60a9-4d0a-a1ec-b18f8fbae752', 'Tacos & Beer', '[2026-02-01 17:00:00+00, 2026-02-02 04:00:00+00]', 'seed'),
  ('e06d9f6a-cd9c-40f5-8854-1b77d0a3b734', 'c8176998-6c38-4813-a9ec-1e45a710e6dc', 'Bale Breaker x Yonder Cider', '1c89db87-e201-4295-a94a-34430d1dd2d3', 'Birrieria Pepe El Toro', '[2026-02-04 00:00:00+00, 2026-02-04 04:00:00+00]', 'seed');
