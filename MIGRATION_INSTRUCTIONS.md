# Database Migration Instructions

## Adding Professional Role and Company Name Columns

### Quick Migration (Recommended)

1. **Open PostgreSQL command line or your preferred database client**

2. **Run the migration script:**
   ```bash
   psql -U jobuser -d connecting_opportunities -f run_migration.sql
   ```

   Or manually execute these commands:
   ```sql
   \c connecting_opportunities;
   
   ALTER TABLE users ADD COLUMN IF NOT EXISTS professional_role VARCHAR(100);
   ALTER TABLE users ADD COLUMN IF NOT EXISTS company_name VARCHAR(100);
   
   CREATE INDEX IF NOT EXISTS idx_users_professional_role ON users(professional_role);
   CREATE INDEX IF NOT EXISTS idx_users_company_name ON users(company_name);
   
   COMMENT ON COLUMN users.professional_role IS 'Professional role/title for job_seekers';
   COMMENT ON COLUMN users.company_name IS 'Company name for employers';
   ```

3. **Verify the migration:**
   ```sql
   \d users;
   ```
   You should see the new columns: `professional_role` and `company_name`

### What This Migration Adds:

- **`professional_role`** (VARCHAR(100)) - For job_seekers/freelancers
  - Examples: "Senior Full Stack Developer", "UI/UX Designer", "Data Scientist"
  
- **`company_name`** (VARCHAR(100)) - For employers/business_owners  
  - Examples: "ChabokSoft", "Google", "Microsoft"

### Usage in Profile:

- **Job Seekers (role = 'job_seeker')**: Shows `professional_role` in profile
- **Employers (role = 'employer')**: Shows `company_name` in profile

### After Migration:

1. Restart your backend server
2. Test registration with both freelancer and business owner accounts
3. Check that the profile displays the correct information based on role

The migration is safe and uses `IF NOT EXISTS` to prevent errors on repeated runs. 