-- Run this script to add the new columns to your existing database
-- Connect to your PostgreSQL database and run these commands:

\c connecting_opportunities;

-- Add the new columns
ALTER TABLE users ADD COLUMN IF NOT EXISTS professional_role VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS company_name VARCHAR(100);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_users_professional_role ON users(professional_role);
CREATE INDEX IF NOT EXISTS idx_users_company_name ON users(company_name);

-- Add comments for documentation
COMMENT ON COLUMN users.professional_role IS 'Professional role/title for job_seekers (e.g., "Senior Full Stack Developer", "UI/UX Designer")';
COMMENT ON COLUMN users.company_name IS 'Company name for employers (e.g., "ChabokSoft", "Google", "Microsoft")';

-- Verify the changes
\d users;

SELECT 'Migration completed successfully!' as status; 