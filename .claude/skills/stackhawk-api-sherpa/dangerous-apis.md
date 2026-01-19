# Dangerous StackHawk APIs

This document describes API endpoints that have surprising or dangerous behavior that doesn't match their OpenAPI specification.

---

## Team API - Duplicate Application Assignment Bug

### The Danger

**Assigning the same application to multiple teams can cause 500 Internal Server Error.**

This is a known API bug. Operations that would result in an application being assigned to more than one team may return HTTP 500.

### What Fails

```
Team A: [App1, App2]
Team B: [App2, App3]  <-- App2 already in Team A, may cause 500
```

### The Safe Pattern

- Ensure each application is only assigned to one team at a time
- Before assigning an app to a team, check if it's already assigned elsewhere
- Consider removing the app from its current team before reassigning

### Status

This is a known bug in the StackHawk API. The scope is not fully determined.

---

## PUT /api/v1/org/{orgId}/team/{teamId} - Team Update

### The Danger

**This is a REPLACE-ALL API, not a PATCH API.**

Despite the OpenAPI spec marking `teamId` and `organizationId` as `readOnly`, the API requires ALL 5 fields to be present in every PUT request:

| Field | OpenAPI Says | Reality | Default if Missing |
|-------|--------------|---------|-------------------|
| `teamId` | readOnly | **REQUIRED** | `""` (breaks things) |
| `organizationId` | readOnly | **REQUIRED** | `""` (breaks things) |
| `name` | required | required | `""` (erases name) |
| `userIds` | optional | **REQUIRED** | `[]` (removes all users) |
| `applicationIds` | optional | **REQUIRED** | `[]` (removes all apps) |

### What Can Go Wrong

If you only send the fields you want to change:

```json
// WRONG - trying to just rename a team
{ "name": "New Name" }
```

The API will interpret this as:
- `teamId` = "" (error or undefined behavior)
- `organizationId` = "" (error or undefined behavior)
- `name` = "New Name" ✓
- `userIds` = [] (ALL USERS REMOVED!)
- `applicationIds` = [] (ALL APPS REMOVED!)

### The Safe Pattern

**Always follow this pattern for team updates:**

```rust
// 1. Fetch current state (use fresh read to bypass cache)
let current_team = client.get_team_fresh(&org_id, &team_id).await?;

// 2. Extract current data
let current_user_ids: Vec<String> = current_team
    .users
    .iter()
    .map(|u| u.user_id.clone())
    .collect();
let current_app_ids: Vec<String> = current_team
    .applications
    .iter()
    .map(|a| a.application_id.clone())
    .collect();

// 3. Build complete desired state
let request = UpdateTeamRequest {
    team_id: team_id.clone(),
    organization_id: org_id.clone(),
    name: Some(new_name),           // Change what you want
    user_ids: Some(current_user_ids), // Preserve what you don't
    application_ids: Some(current_app_ids),
};

// 4. PUT the complete state
client.update_team(&org_id, &team_id, request).await?;
```

### Pagination Warning

For teams with many users or apps, ensure you're fetching ALL of them before making updates:

- Users: May require pagination if team has 1000+ members
- Apps: May require pagination if team owns 1000+ applications

**Never assume a single page contains all data.**

### Cache Warning

**Always use fresh reads before mutations:**
- Use `get_team_fresh()` instead of `get_team()` before updates
- The cached team data may be stale and missing recently added users/apps

### Code Reference

See `src/client/models/user.rs` - `UpdateTeamRequest` struct documentation for implementation details.

---

## General Principles for StackHawk APIs

1. **Don't trust "readOnly" in OpenAPI** - Test actual behavior
2. **Assume PUT means REPLACE-ALL** - Always send complete state
3. **Fetch before mutate** - Get current state, modify locally, PUT back
4. **Use fresh reads** - Bypass cache before mutations
5. **Handle pagination** - Large teams may have paginated users/apps
