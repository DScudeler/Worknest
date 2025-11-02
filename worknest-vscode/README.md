# Worknest VSCode Extension

Seamless Worknest ticket management integration for Visual Studio Code.

## Features

### 🎯 Ticket Management
- **Ticket Tree View**: Browse projects and tickets in the sidebar
- **Quick Actions**: Create, search, and manage tickets without leaving your IDE
- **Smart Filtering**: Filter tickets by project, status, priority, and assignee
- **Ticket Details**: View and edit ticket information in a webview panel

### 🔀 Git Integration
- **Smart Commit Messages**: Auto-populate commit messages with ticket information
- **Branch Creation**: Generate branch names from tickets (e.g., `feature/TICKET-123-description`)
- **Ticket Detection**: Automatically detect current ticket from branch name
- **Code Linking**: Insert ticket references as comments in your code

### 📊 Status Bar
- **Current Ticket**: Shows current ticket when branch name contains ticket reference
- **Assigned Tickets**: Displays count of tickets assigned to you
- **Quick Access**: Click to open ticket details

### ⚡ Commands

Access via Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P`):

- `Worknest: Login` - Authenticate with Worknest server
- `Worknest: Logout` - Sign out
- `Worknest: Create Ticket` - Create a new ticket
- `Worknest: Search Tickets` - Full-text search across all tickets
- `Worknest: Refresh` - Refresh ticket data
- `Worknest: Assign Ticket to Me` - Assign selected ticket to yourself
- `Worknest: Change Ticket Status` - Update ticket status
- `Worknest: Create Branch from Ticket` - Create and checkout a new branch
- `Worknest: Insert Ticket Reference` - Add ticket comment in code

### ⌨️ Keyboard Shortcuts

- `Ctrl+Alt+W T` (Mac: `Cmd+Alt+W T`) - Create Ticket
- `Ctrl+Alt+W S` (Mac: `Cmd+Alt+W S`) - Search Tickets
- `Ctrl+Alt+W R` (Mac: `Cmd+Alt+W R`) - Refresh

## Installation

### From Source

1. Clone the Worknest repository
2. Navigate to the extension directory:
   ```bash
   cd worknest-vscode
   ```
3. Install dependencies:
   ```bash
   npm install
   ```
4. Compile TypeScript:
   ```bash
   npm run compile
   ```
5. Press `F5` in VSCode to launch Extension Development Host

## Configuration

Configure the extension via VSCode settings (`Ctrl+,` or `Cmd+,`):

### `worknest.apiUrl`
- **Type**: `string`
- **Default**: `http://localhost:3000`
- **Description**: Worknest API server URL

### `worknest.autoRefresh`
- **Type**: `boolean`
- **Default**: `true`
- **Description**: Automatically refresh tickets periodically

### `worknest.refreshInterval`
- **Type**: `number`
- **Default**: `60`
- **Description**: Auto-refresh interval in seconds

### `worknest.showStatusBar`
- **Type**: `boolean`
- **Default**: `true`
- **Description**: Show ticket information in status bar

### `worknest.defaultProject`
- **Type**: `string`
- **Default**: `""`
- **Description**: Default project ID for new tickets

## Usage

### Getting Started

1. **Start Worknest API Server**:
   ```bash
   cd crates/worknest-api
   cargo run
   ```

2. **Login**: Open Command Palette and run `Worknest: Login`

3. **Browse Tickets**: Click the Worknest icon in the Activity Bar

4. **Create Ticket**: Use `Ctrl+Alt+W T` or Command Palette

### Workflow Example

1. **Create a ticket** for your feature/bug
2. **Create a branch** from the ticket using context menu
3. **Write code** with automatic commit message pre-fill
4. **Update status** as you progress (InProgress → Review → Done)
5. **Link code** to tickets with inline comments

## Requirements

- VSCode 1.80.0 or higher
- Running Worknest API server
- Git extension (for Git integration features)

## Development

### Build
```bash
npm run compile
```

### Watch Mode
```bash
npm run watch
```

### Lint
```bash
npm run lint
```

## Architecture

```
worknest-vscode/
├── src/
│   ├── api/                  # API client and types
│   │   ├── client.ts         # HTTP client
│   │   └── types.ts          # DTOs
│   ├── views/                # UI components
│   │   ├── ticketTree.ts     # Tree view provider
│   │   └── statusBar.ts      # Status bar manager
│   ├── commands/             # Command implementations
│   │   ├── tickets.ts        # Ticket operations
│   │   └── git.ts            # Git integration
│   ├── utils/                # Utilities
│   │   ├── config.ts         # Configuration
│   │   └── storage.ts        # Secure storage
│   └── extension.ts          # Entry point
├── package.json              # Extension manifest
└── tsconfig.json             # TypeScript config
```

## Contributing

Contributions are welcome! Please see the main Worknest repository for contribution guidelines.

## License

MIT License - see LICENSE file for details

## Support

- **Issues**: Report bugs and request features on GitHub
- **Documentation**: See main Worknest documentation
- **API**: http://localhost:3000 (default)
