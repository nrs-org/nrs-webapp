-- The tester user should have no password by default
INSERT INTO "appuser" (username, password_hash) VALUES ('tester', '');
