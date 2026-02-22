INSERT OR IGNORE INTO breweries (id, name, notes, website, address, lat, lng, drink_menu, food_schedule)
VALUES
  ('ddbef262-8ae4-413c-9fa3-e4fbc40175b5', 'Stoup Brewing', 'Brewery', 'https://www.stoupbrewing.com/ballard/', '1108 NW 52nd St, Seattle, WA 98107', 47.66653130601593, -122.3711539291425, 'https://www.stoupbrewing.com/ballard/#whats-on-tap-ballard', 'https://www.stoupbrewing.com/ballard/#food-truck-schedule-ballard'),
  ('c8176998-6c38-4813-a9ec-1e45a710e6dc', 'Bale Breaker x Yonder Cider', 'Taste the East, out West', 'https://www.bbycballard.com/', '826 NW 49th Street, Seattle, WA 98107', 47.66454208318039, -122.36755886363453, 'https://www.bbycballard.com/current-taplist', 'https://www.bbycballard.com/food-trucks-1-1'),
  ('64b2ec8a-41ff-44c8-9b91-b0d7c3e0457b', 'Urban Family Brewing Co.', NULL, 'https://urbanfamilybrewing.com/', '4441 26th Ave W, Seattle, WA 98199', 47.660851, -122.389851, NULL, 'https://urbanfamilybrewing.com/home/calendar/'),
  ('f5a98880-fd2d-4c25-b72d-0a7f89fcd866', 'Lucky Envelope Brewing', NULL, 'https://www.luckyenvelopebrewing.com/', '907 NW 50th St, Seattle, WA 98107', 47.665631, -122.369877, NULL, 'https://www.luckyenvelopebrewing.com/events'),
  ('6373ac59-7f83-4565-8f94-9d4d6d7582ec', 'Fair Isle Brewing', NULL, 'https://fairislebrewing.com/', '936 NW 49th St, Seattle, WA 98107', 47.664811, -122.36963, NULL, 'https://fairislebrewing.com/events/'),
  ('0f4f2392-dc38-493f-aed3-9dfba21d51b0', 'Obec Brewing', NULL, 'https://obecbrewing.com/', '1144 NW 52nd St, Seattle, WA 98107', 47.66657, -122.37216, NULL, 'https://obecbrewing.com/');


INSERT OR IGNORE INTO vendors (id, name, normalized_name, notes, website, menu, needs_review, match_method)
VALUES
  ('def4c743-7ca0-444c-8294-bfc454e57461', 'El Pirata Tortas Y Burritos', 'elpiratatortasyburritos', 'Mexican-inspired dishes', 'https://elpiratatortas.com', 'https://elpiratatortas.com/menu', 0, 'seed'),
  ('c65ebe31-6b68-4373-b41d-760ba01476e2', 'Where Ya At Matt', 'whereyaatmatt', 'New Orleans soul food', 'https://www.whereyaatmatt.com', 'https://www.whereyaatmatt.com/menu', 0, 'seed'),
  ('8ba304ab-3d7c-445a-b77c-bde1768c89b2', 'The Little Pearl Oyster Bar', 'thelittlepearloysterbar', 'A mobile seafood bar featuring the finest oysters from the Salish Sea.', 'https://salishseagreens.com/pages/catering', 'https://salishseagreens.com/pages/catering', 0, 'seed'),
  ('09cfecc0-60a9-4d0a-a1ec-b18f8fbae752', 'Tacos & Beer', 'tacosbeer', 'Authentic Cocina Mexicana', 'https://www.tacosandbeerseattle.com', 'https://www.tacosandbeerseattle.com/menu', 0, 'seed'),
  ('1c89db87-e201-4295-a94a-34430d1dd2d3', 'Birrieria Pepe El Toro', 'birrieriapepeeltoro', 'Taco! Tortas! Burritos! Quesadillas!', 'https://www.birrieria-pepeeltoro.com', 'https://www.birrieria-pepeeltoro.com/menu', 0, 'seed');

INSERT OR IGNORE INTO schedule_entries (
  id,
  brewery_id,
  vendor_id,
  start_at,
  end_at,
  source
)
VALUES
  ('1f2fa18e-96e6-4ba4-aa91-f967f2ce270a', 'ddbef262-8ae4-413c-9fa3-e4fbc40175b5', 'def4c743-7ca0-444c-8294-bfc454e57461', '2026-02-01T17:00:00Z', '2026-02-02T03:00:00Z', 'seed'),
  ('9426bb02-07ba-4c2e-a436-b433a9afff96', 'ddbef262-8ae4-413c-9fa3-e4fbc40175b5', 'c65ebe31-6b68-4373-b41d-760ba01476e2', '2026-02-03T01:00:00Z', '2026-02-03T04:00:00Z', 'seed'),
  ('315ff286-d5be-48d7-978e-be385b634d4b', 'c8176998-6c38-4813-a9ec-1e45a710e6dc', '8ba304ab-3d7c-445a-b77c-bde1768c89b2', '2026-02-01T16:00:00Z', '2026-02-02T01:00:00Z', 'seed'),
  ('2130bc93-efd1-4347-8d25-631c4b96e13f', 'c8176998-6c38-4813-a9ec-1e45a710e6dc', '09cfecc0-60a9-4d0a-a1ec-b18f8fbae752', '2026-02-01T17:00:00Z', '2026-02-02T04:00:00Z', 'seed'),
  ('e06d9f6a-cd9c-40f5-8854-1b77d0a3b734', 'c8176998-6c38-4813-a9ec-1e45a710e6dc', '1c89db87-e201-4295-a94a-34430d1dd2d3', '2026-02-04T00:00:00Z', '2026-02-04T04:00:00Z', 'seed');
