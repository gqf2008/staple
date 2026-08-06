-- Roll back attention inbox dismissals (also covered by migration 0026's
-- down migration).

DROP TABLE IF EXISTS inbox_dismissals;
