# Web GUI - API Integration TODOs

The desktop native support has been removed from Worknest. The GUI now requires API client implementation to connect to the backend server.

## Screens Needing API Integration

All screens have been updated to remove direct database access. The following screens need API client methods implemented:

### Authentication
- [ ] `login.rs` - Implement `api_client.login()` 
- [ ] `register.rs` - Implement `api_client.register()`

### Projects
- [ ] `dashboard.rs` - Implement `api_client.get_projects()`
- [ ] `project_list.rs` - Implement:
  - `api_client.get_projects()`
  - `api_client.create_project()`
  - `api_client.archive_project()`
  - `api_client.unarchive_project()`
- [ ] `project_detail.rs` - Implement:
  - `api_client.get_project_by_id()`
  - `api_client.update_project()`
  - `api_client.get_project_tickets()`

### Tickets
- [ ] `ticket_list.rs` - Implement:
  - `api_client.get_tickets()`
  - `api_client.create_ticket()`
- [ ] `ticket_board.rs` - Implement `api_client.get_project_tickets()`
- [ ] `ticket_detail.rs` - Implement:
  - `api_client.get_ticket_by_id()`
  - `api_client.update_ticket()`
  - `api_client.delete_ticket()`
  - `api_client.update_ticket_status()`

## API Client Implementation

The `api_client.rs` file needs to be expanded to include all the methods above. Each method should:
1. Make HTTP requests to the backend API
2. Handle authentication tokens
3. Parse responses into domain models
4. Handle errors appropriately

## Testing

Once API methods are implemented, test the following:
1. User can log in and register
2. Projects can be created, viewed, and managed
3. Tickets can be created, viewed, and managed
4. Responsive UI works on desktop and mobile browsers

## Build Instructions

```bash
# Build web application  
trunk build --release

# Serve for development
trunk serve
```
