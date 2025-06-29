-- Database and user creation for job finder

-- Drop database first to remove dependencies
DROP DATABASE IF EXISTS connecting_opportunities;
-- Then drop the user
DROP USER IF EXISTS jobuser;
-- Create the user
CREATE USER jobuser WITH PASSWORD 'jobuser';
-- Create the database
CREATE DATABASE connecting_opportunities;
-- Set ownership
ALTER DATABASE connecting_opportunities OWNER TO jobuser;
