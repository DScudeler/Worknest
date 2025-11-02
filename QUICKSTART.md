# Worknest VSCode Extension - Quick Start Guide

## 5-Minute Setup

### 1. Start the Backend (Terminal 1)

```bash
cd /path/to/Worknest
cargo run --package worknest-api
```

Wait for: `Starting server on 0.0.0.0:3000`

### 2. Install Extension Dependencies (Terminal 2)

```bash
cd /path/to/Worknest/worknest-vscode
npm install
npm run compile
```

### 3. Launch Extension

**Option A - Debug Mode** (Recommended for testing):
1. Open `worknest-vscode` folder in VSCode
2. Press `F5`
3. New VSCode window opens with extension loaded

**Option B - Install VSIX**:
```bash
npm install -g @vscode/vsce
vsce package
code --install-extension worknest-0.1.0.vsix
```

### 4. First Login

In the new VSCode window:

1. **Create Account** (if needed):
   ```bash
   curl -X POST http://localhost:3000/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{"username":"dev","email":"dev@example.com","password":"password123"}'
   ```

2. **Login to Extension**:
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P`)
   - Type: `Worknest: Login`
   - Username: `dev`
   - Password: `password123`

3. **Verify**: Click Worknest icon in Activity Bar (left sidebar)

## Quick Feature Demo

### Create Your First Ticket

1. Press `Ctrl+Alt+W T` (or `Cmd+Alt+W T`)
2. Select or create a project
3. Enter title: `Test VSCode Extension`
4. Select type: `Task`
5. Select priority: `Medium`
6. Add description (optional)
7. ✅ Ticket created!

### Search Tickets

1. Press `Ctrl+Alt+W S` (or `Cmd+Alt+W S`)
2. Type: `test`
3. Select ticket from results
4. View details in panel

### Git Workflow

1. **Find ticket in tree view** (click Worknest icon)
2. **Right-click ticket** → `Create Branch from Ticket`
3. **Branch created**: `feature/abcd1234-test-vscode-extension`
4. **Make changes** to any file
5. **Commit**: SCM input pre-filled with `[abcd1234] Test VSCode Extension: `
6. **Status bar** shows current ticket

### Update Ticket Status

1. Find ticket in tree view
2. Right-click → `Change Status`
3. Select: `InProgress`
4. ✅ Status updated!

## Keyboard Shortcuts Cheat Sheet

| Action | Windows/Linux | Mac |
|--------|--------------|-----|
| Create Ticket | `Ctrl+Alt+W T` | `Cmd+Alt+W T` |
| Search Tickets | `Ctrl+Alt+W S` | `Cmd+Alt+W S` |
| Refresh | `Ctrl+Alt+W R` | `Cmd+Alt+W R` |
| Command Palette | `Ctrl+Shift+P` | `Cmd+Shift+P` |

## Common Commands

Open Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P`) and type:

- `Worknest: Create Ticket`
- `Worknest: Search Tickets`
- `Worknest: Refresh`
- `Worknest: Login`
- `Worknest: Logout`

## Troubleshooting Quick Fixes

### "Cannot connect to server"
```bash
# Check if server is running
curl http://localhost:3000/health
# Should return: OK
```

### "Login failed"
```bash
# Verify credentials or create new account
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"test","email":"test@example.com","password":"test123"}'
```

### Extension not visible
1. Check Activity Bar (left sidebar) for Worknest icon
2. If missing: View → Extensions → Search "Worknest"
3. Reload window: `Ctrl+R` or `Cmd+R`

### Empty tree view
1. Verify you're logged in (check for Worknest status bar item)
2. Create a project and ticket via the API or GUI
3. Click refresh button in tree view

## Next Steps

📚 **Read Full Docs**:
- [worknest-vscode/README.md](worknest-vscode/README.md)
- [worknest-vscode/INSTALLATION.md](worknest-vscode/INSTALLATION.md)

🔧 **Configure Settings**:
- VSCode Settings (`Ctrl+,`) → Search "Worknest"
- Customize API URL, refresh interval, etc.

🚀 **Advanced Features**:
- Set up auto-refresh
- Configure default project
- Customize branch naming
- Use code linking

## Workflow Example

1. **Morning**: Login to Worknest extension
2. **Find ticket**: Search or browse tree view
3. **Start work**: Right-click → "Assign to Me" → "Change Status" to "InProgress"
4. **Create branch**: Right-click → "Create Branch from Ticket"
5. **Code**: Make changes with automatic commit message pre-fill
6. **Update**: Right-click → "Change Status" → "Review"
7. **Done**: Mark as "Done" when merged

## Development Workflow

If you want to modify the extension:

```bash
# Watch mode - auto-compile on save
cd worknest-vscode
npm run watch

# In another terminal, press F5 in VSCode to debug
# Make changes → Save → Reload extension window (Ctrl+R)
```

## Support

- **Backend Issues**: Check `crates/worknest-api/` logs
- **Extension Issues**: Help → Toggle Developer Tools → Console tab
- **Questions**: See [README.md](worknest-vscode/README.md)

Happy ticket tracking! 🎯
