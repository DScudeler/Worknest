-- Project source-repo location.
--
-- Required so the agents subsystem can bootstrap a per-deployment git
-- worktree (BootstrapWorktree activation step). The column accepts either
-- an absolute filesystem path to an existing repo or a clone URL
-- (https://, git@, ssh://). When NULL the deployment's BootstrapWorktree
-- step is a no-op — the agent gets a config-only workspace, no code.

ALTER TABLE projects ADD COLUMN repo_path TEXT;
