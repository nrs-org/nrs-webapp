-- NOTE:
-- This completely drops and recreates the development database and user.
-- DO NOT RUN THIS ON A PRODUCTION DATABASE!

SELECT pg_terminate_backend(pid) from pg_stat_activity
WHERE usename = 'nrs_webapp_user' OR datname = 'nrs_webapp_dev';

DROP DATABASE IF EXISTS nrs_webapp_dev;
DROP USER IF EXISTS nrs_webapp_user;

CREATE USER nrs_webapp_user WITH PASSWORD 'dev_only_pwd';
CREATE DATABASE nrs_webapp_dev OWNER nrs_webapp_user;

ALTER DATABASE postgres SET log_statement = 'all';