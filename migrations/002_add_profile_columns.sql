-- Migration: Add professional_role and company_name columns to users table
-- Date: 2025-01-29
-- Description: Add columns to store role-specific profile information

-- Add professional_role column for job_seekers/freelancers
ALTER TABLE users ADD COLUMN IF NOT EXISTS professional_role VARCHAR(100);

-- Add company_name column for employers/business_owners  
ALTER TABLE users ADD COLUMN IF NOT EXISTS company_name VARCHAR(100);

-- Create indexes for the new columns for better query performance
CREATE INDEX IF NOT EXISTS idx_users_professional_role ON users(professional_role);
CREATE INDEX IF NOT EXISTS idx_users_company_name ON users(company_name);

-- Add comments to document the purpose of these columns
COMMENT ON COLUMN users.professional_role IS 'Professional role/title for job_seekers (e.g., "Senior Full Stack Developer", "UI/UX Designer")';
COMMENT ON COLUMN users.company_name IS 'Company name for employers (e.g., "ChabokSoft", "Google", "Microsoft")'; 