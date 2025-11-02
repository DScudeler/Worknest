# Worknest VSCode Extension - Installation & Setup Guide

## Prerequisites

1. **VSCode**: Version 1.80.0 or higher
2. **Node.js**: Version 18.x or higher
3. **Worknest API**: Running instance of Worknest backend

## Installation Steps

### Option 1: Development Installation (Recommended for Testing)

1. **Navigate to Extension Directory**:
   ```bash
   cd /path/to/Worknest/worknest-vscode
   ```

2. **Install Dependencies**:
   ```bash
   npm install
   ```

3. **Compile TypeScript**:
   ```bash
   npm run compile
   ```

4. **Launch Extension Development Host**:
   - Open the `worknest-vscode` folder in VSCode
   - Press `F5` to launch a new VSCode window with the extension loaded
   - Or: Run > Start Debugging

### Option 2: Package and Install

1. **Install vsce (if not already installed)**:
   ```bash
   npm install -g @vscode/vsce
   ```

2. **Package the Extension**:
   ```bash
   cd /path/to/Worknest/worknest-vscode
   vsce package
   ```
   This creates a `.vsix` file (e.g., `worknest-0.1.0.vsix`)

3. **Install the Extension**:
   - Open VSCode
   - Go to Extensions view (`Ctrl+Shift+X` or `Cmd+Shift+X`)
   - Click the `...` menu → "Install from VSIX..."
   - Select the generated `.vsix` file

## Backend Setup

### Start Worknest API Server

1. **Navigate to API Directory**:
   ```bash
   cd /path/to/Worknest
   ```

2. **Set Environment Variables** (Optional):
   ```bash
   export WORKNEST_DB_PATH=./worknest-api.db
   export WORKNEST_SECRET_KEY=your-secret-key-here
   export PORT=3000
   ```

3. **Run the Server**:
   ```bash
   cargo run --package worknest-api
   ```

4. **Verify Server is Running**:
   ```bash
   curl http://localhost:3000/health
   # Should return: OK
   ```

## First-Time Configuration

### 1. Configure API URL

Open VSCode Settings (`Ctrl+,` or `Cmd+,`) and search for "worknest":

- **Worknest: Api Url**: Set to your Worknest server URL
  - Default: `http://localhost:3000`
  - Production: `https://your-worknest-server.com`

### 2. Create User Account

If you don't have an account yet:

```bash
# Using curl
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "your-username",
    "email": "your-email@example.com",
    "password": "your-password"
  }'
```

Or use the Worknest GUI to register.

### 3. Login to Extension

1. Open Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P`)
2. Type "Worknest: Login"
3. Enter your username and password
4. Click the Worknest icon in the Activity Bar to view tickets

## Verification

### Test the Extension

1. **View Tickets**:
   - Click the Worknest icon in the Activity Bar
   - You should see your projects and tickets in the tree view

2. **Create a Ticket**:
   - Press `Ctrl+Alt+W T` (or `Cmd+Alt+W T` on Mac)
   - Fill in ticket details
   - Verify it appears in the tree view

3. **Search Tickets**:
   - Press `Ctrl+Alt+W S` (or `Cmd+Alt+W S` on Mac)
   - Enter a search query
   - Select a ticket to view details

4. **Git Integration**:
   - Right-click a ticket in the tree view
   - Select "Create Branch from Ticket"
   - Verify branch is created and checked out
   - Make a commit - the commit message should be pre-filled

## Troubleshooting

### Cannot Connect to Server

**Error**: "Cannot connect to Worknest server"

**Solutions**:
1. Verify the API server is running:
   ```bash
   curl http://localhost:3000/health
   ```

2. Check the API URL in VSCode settings

3. If using a different port, update the `worknest.apiUrl` setting

4. Check for firewall or network issues

### Authentication Failed

**Error**: "Login failed: Invalid credentials"

**Solutions**:
1. Verify username and password are correct
2. Create a new account if needed
3. Check server logs for errors

### Tree View Empty

**Issue**: Tickets don't appear in tree view

**Solutions**:
1. Verify you're logged in (check status bar)
2. Create a project and tickets via GUI or API
3. Click the refresh button in the tree view
4. Check browser console for errors (Help > Toggle Developer Tools)

### Git Integration Not Working

**Issue**: Branch creation or commit message pre-fill doesn't work

**Solutions**:
1. Verify Git extension is installed and enabled
2. Open a folder with a Git repository
3. Check that you have a valid ticket ID in branch name

### Extension Not Loading

**Issue**: Extension doesn't appear or activate

**Solutions**:
1. Check VSCode version (must be ≥1.80.0)
2. Reload VSCode window (`Ctrl+R` or `Cmd+R`)
3. Check extension errors:
   - View > Output > Select "Extension Host"
4. Reinstall extension

## Development Workflow

### Watch Mode (for Development)

Instead of manually compiling after each change:

```bash
npm run watch
```

This will automatically recompile TypeScript files when you save changes.

### Debugging

1. Open `worknest-vscode` folder in VSCode
2. Press `F5` to start debugging
3. Set breakpoints in TypeScript files
4. Use Debug Console to inspect variables

### Testing Changes

After making code changes:

1. Reload the Extension Development Host window:
   - `Ctrl+R` or `Cmd+R` in the Extension Development Host
2. Or restart debugging (`F5`)

## Advanced Configuration

### Custom Refresh Interval

Set how often tickets auto-refresh (in seconds):

```json
{
  "worknest.refreshInterval": 30
}
```

### Disable Auto-Refresh

```json
{
  "worknest.autoRefresh": false
}
```

### Default Project

Set a default project for new tickets:

```json
{
  "worknest.defaultProject": "project-uuid-here"
}
```

### Hide Status Bar

```json
{
  "worknest.showStatusBar": false
}
```

## Uninstallation

### Via VSCode UI

1. Go to Extensions view (`Ctrl+Shift+X`)
2. Find "Worknest" extension
3. Click the gear icon → "Uninstall"

### Via Command Line

```bash
code --uninstall-extension worknest
```

### Clean Up Data

The extension stores authentication tokens in VSCode's secure storage. To clear:

1. Logout using `Worknest: Logout` command
2. Or delete VSCode workspace storage manually

## Next Steps

- **Create Projects**: Use the Worknest GUI or API
- **Customize Shortcuts**: VSCode Settings > Keyboard Shortcuts
- **Explore Commands**: `Ctrl+Shift+P` > Type "Worknest"
- **Integrate with Workflow**: Create branches, link commits to tickets

## Support

For issues or questions:
- **GitHub Issues**: https://github.com/your-org/worknest
- **Documentation**: See main Worknest README
- **API Docs**: http://localhost:3000/health

Happy ticket tracking! 🎯
