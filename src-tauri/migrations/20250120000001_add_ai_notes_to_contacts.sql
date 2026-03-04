-- Add ai_notes column to contacts table for storing AI context notes per contact
ALTER TABLE contacts ADD COLUMN ai_notes TEXT;
